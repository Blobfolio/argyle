/*!
# Argyle: Argue
*/

use crate::{
	ArgyleError,
	ArgsOsStr,
	KeyKind,
};
use std::{
	cell::Cell,
	ffi::OsStr,
	ops::{
		BitOr,
		Deref,
	},
	os::unix::ffi::{
		OsStrExt,
		OsStringExt,
	},
};



/// # Key/Value Iterator Item.
///
/// The bool indicates whether or not this was a miscellaneous argument (i.e.
/// not a key).
///
/// The middle value is either said argument or the key.
///
/// The third item is the attached value, if any.
type KvIterItem = (bool, Vec<u8>, Option<Vec<u8>>);



/// # Flag: Argument(s) Required.
///
/// If a program is called with zero arguments — no flags, options, trailing
/// args —, an error will be printed and the thread will exit with status code
/// `1`.
pub const FLAG_REQUIRED: u8 =     0b0000_0001;

/// # Flag: Expect Subcommand.
///
/// Set this flag to treat the first value as a subcommand rather than a
/// trailing argument. (This fixes the edge case where the command has zero
/// dash-prefixed keys.)
pub const FLAG_SUBCOMMAND: u8 =   0b0000_0010;

#[cfg(feature = "dynamic-help")]
#[cfg_attr(feature = "docsrs", doc(cfg(feature = "dynamic-help")))]
/// # Flag: Check For Help Flag.
///
/// When set, [`Argue`] will return [`ArgyleError::WantsDynamicHelp`] if help args
/// are present. The subcommand, if any, is included, allowing the caller to
/// dynamically handle output.
pub const FLAG_DYNAMIC_HELP: u8 = 0b0000_0100;

/// # Flag: Check For Help Flag.
///
/// When set, [`Argue`] will return [`ArgyleError::WantsHelp`] if help args are
/// present.
pub const FLAG_HELP: u8 =         0b0000_1000;

/// # Flag: Check For Version Flag.
///
/// When set, [`Argue`] will return [`ArgyleError::WantsVersion`] if version
/// args are present.
pub const FLAG_VERSION: u8 =      0b0001_0000;

/// # Flag: Has Help.
///
/// This flag is set if either `-h` or `--help` switches are present. It has
/// no effect unless [`Argue::FLAG_HELP`] is set.
const FLAG_HAS_HELP: u8 =         0b0010_0000;

/// # Flag: Has Version.
///
/// This flag is set if either `-V` or `--version` switches are present. It has
/// no effect unless [`Argue::FLAG_VERSION`] is set.
const FLAG_HAS_VERSION: u8 =      0b0100_0000;

/// # Flag: Do Version.
///
/// When both `FLAG_VERSION` and `FLAG_HAS_VERSION` are set.
const FLAG_DO_VERSION: u8 =       FLAG_VERSION | FLAG_HAS_VERSION;

#[cfg(feature = "dynamic-help")]
/// # Flag: Any Help.
///
/// When either `FLAG_HELP` or `FLAG_DYNAMIC_HELP` are set.
const FLAG_ANY_HELP: u8 =         FLAG_HELP | FLAG_DYNAMIC_HELP;

#[cfg(not(feature = "dynamic-help"))]
/// # Flag: Any Help.
///
/// When either `FLAG_HELP` or `FLAG_DYNAMIC_HELP` are set.
const FLAG_ANY_HELP: u8 =         FLAG_HELP;



/// # The size of our keys array.
const KEY_SIZE: usize = 16;
/// # The index noting total key length.
const KEY_LEN: usize = 15;



#[derive(Debug, Clone)]
/// `Argue` is an agnostic CLI argument parser. Unlike more robust libraries
/// like [clap](https://crates.io/crates/clap), `Argue` does not hold
/// information about expected or required arguments; it merely parses the raw
/// arguments into a consistent state so the implementor can query them as
/// needed.
///
/// Post-processing is an exercise largely left to the implementing library to
/// do in its own way, in its own time. `Argue` exposes several methods for
/// quickly querying the individual pieces of the set, but it can also be
/// dereferenced to a slice or consumed into an owned vector for fully manual
/// processing if desired.
///
/// Arguments are processed and held as owned bytes rather than (os)strings,
/// again leaving the choice of later conversion entirely up to the
/// implementor.
///
/// For simple applications, this agnostic approach can significantly reduce
/// the overhead of processing CLI arguments, but because handling is left to
/// the implementing library, it might be too tedious or limiting for more
/// complex use cases.
///
/// ## Assumptions
///
/// `Argue` is built for speed and simplicity, and as such, contains a number
/// of assumptions and limitations that might make it unsuitable for use.
///
/// ### Keys
///
/// A "key" is an argument entry beginning with one or two dashes `-` and an
/// ASCII letter (`A..=Z` or `a..=z`). Entries with one dash are "short", and
/// can only consist of two bytes. Entries with two dashes are "long" and can
/// be however long they want to be.
///
/// If a short key entry is longer than two bytes, everything in range `2..` is
/// assumed to be a value and is split off into its own entry. For example,
/// `-kVal` is equivalent to `-k Val`.
///
/// If a long key contains an `=`, it is likewise assumed to be a key/value
/// pair, and will be split into two at that index. For example, `--key=Val` is
/// equivalent to `--key Val`.
///
/// A key without a value is called a "switch". It is `true` if present,
/// `false` if not.
///
/// A key with one value is called an "option". Multi-value options are *not*
/// supported.
///
/// ### Trailing Arguments
///
/// All values beginning after the last known switch or option value are
/// considered to be trailing arguments. Any number (including zero) of
/// trailing arguments can be provided.
///
/// ### Restrictions
///
/// 1. Keys are not checked for uniqueness, but only the first occurrence of a given key will ever match.
/// 2. A given argument set may only include up to **15** keys. If that number is exceeded, `Argue` will print an error and terminate the thread with a status code of `1`.
/// 3. The total number of keys, values, and arguments may not exceed `u16::MAX`.
/// 4. A glued `--key=Val` expression cannot be longer than `u16::MAX` bytes.
/// 5. Argument parsing stops if a passthrough separator `--` is found. Anything up to that point is parsed as usual; everything after is discarded.
///
/// ## Examples
///
/// `Argue` follows a builder pattern for construction, with a few odds and
/// ends tucked away as flags.
///
/// ```no_run
/// use argyle::{Argue, FLAG_REQUIRED};
///
/// // Parse the env arguments, aborting if the set is empty.
/// let args = Argue::new(FLAG_REQUIRED).unwrap();
///
/// // Check to see what's there.
/// let switch: bool = args.switch(b"-s");
/// let option: Option<&[u8]> = args.option(b"--my-opt");
/// let extras: &[Vec<u8>] = args.args();
/// ```
///
/// If you just want a clean set to iterate over, `Argue` can be dereferenced
/// to a slice:
///
/// ```ignore
/// let arg_slice: &[Vec<u8>] = *args;
/// ```
///
/// Or it can be converted into an owned Vector:
/// ```ignore
/// let args: Vec<Vec<u8>> = args.take();
/// ```
pub struct Argue {
	/// Parsed arguments.
	args: Vec<Vec<u8>>,
	/// Keys.
	///
	/// This array holds the key indexes (from `self.args`) so checks can avoid
	/// re-evaluation, etc.
	///
	/// The last slot holds the number of keys, hence only 15 total keys are
	/// supported.
	keys: [u16; KEY_SIZE],
	/// Highest non-arg index.
	///
	/// This is used to divide the arguments between named and trailing values.
	/// This is inferred during instantiation from the last-found dash-prefixed
	/// key, but could be updated `+1` if that key turns out to be an option
	/// (its value would then be the last non-trailing argument).
	///
	/// The only way `Argue` knows switches from options is by the method
	/// invoked by the implementing library. Be sure to request all options
	/// before asking for trailing arguments.
	last: Cell<u16>,
	/// Flags.
	flags: u8,
}

impl Default for Argue {
	#[inline]
	fn default() -> Self {
		Self {
			args: Vec::with_capacity(KEY_SIZE),
			keys: [0_u16; KEY_SIZE],
			last: Cell::new(0),
			flags: 0,
		}
	}
}

impl Deref for Argue {
	type Target = [Vec<u8>];
	#[inline]
	fn deref(&self) -> &Self::Target { &self.args }
}

/// ## Instantiation and Builder Patterns.
impl Argue {
	/// # New Instance.
	///
	/// This simply parses the owned output of [`std::env::args_os`].
	///
	/// ## Errors
	///
	/// This method will bubble any processing errors or aborts (like the
	/// discovery of version or help flags).
	pub fn new(chk_flags: u8) -> Result<Self, ArgyleError> {
		let mut args: Vec<Vec<u8>> = Vec::with_capacity(KEY_SIZE);
		let mut keys = [0_u16; KEY_SIZE];
		let mut last = 0_u16;
		let mut idx = 0_u16;
		let mut flags = 0_u8;

		// Run through all the raw arguments, except the first one, which holds
		// the program path.
		for x in std::env::args_os().skip(1) {
			let (standalone, key, value) = kv_adapter(x.into_vec());

			// Skip leading empties.
			if 0 == idx && (key.is_empty() || key.iter().all(u8::is_ascii_whitespace)) {
				continue;
			}

			// How many spots will this take up?
			let inc: u16 =
				if value.is_some() { 2 }
				else if key == b"--" { break; }
				else { 1 };

			// Make sure we fit.
			if u16::MAX - inc < idx { return Err(ArgyleError::TooManyArgs); }

			// Just an arg?
			if standalone { idx += inc; }
			// Do key stuff!
			else {
				if value.is_none() {
					match key.as_slice() {
						b"-V" | b"--version" => { flags |= FLAG_HAS_VERSION; },
						b"-h" | b"--help" => { flags |= FLAG_HAS_HELP; },
						_ => {},
					}
				}

				let num_keys = keys[KEY_LEN] as usize;
				if num_keys == KEY_LEN { return Err(ArgyleError::TooManyKeys); }

				keys[num_keys] = idx;
				keys[KEY_LEN] += 1;

				idx += inc;
				last = idx - 1;
			}

			// Push the things.
			args.push(key);
			if let Some(v) = value { args.push(v); }
		}

		// Turn it into an object!
		let mut out = Self {
			args,
			keys,
			last: Cell::new(last),
			flags,
		};

		// Run any flag checks.
		out.check_flags(chk_flags)?;

		// Done!
		Ok(out)
	}

	/// # Set/Check Flags.
	///
	/// This is run after [`Argue::new`] to see what's what.
	fn check_flags(&mut self, flags: u8) -> Result<(), ArgyleError> {
		if 0 < flags {
			self.flags |= flags;

			// There are no arguments.
			if self.args.is_empty() {
				// Required?
				if 0 != self.flags & FLAG_REQUIRED {
					return Err(ArgyleError::Empty);
				}
			}
			// Print version.
			else if FLAG_DO_VERSION == self.flags & FLAG_DO_VERSION {
				return Err(ArgyleError::WantsVersion);
			}
			// Check for help.
			else if let Some(e) = self.help_flag() {
				return Err(e);
			}
		}

		Ok(())
	}

	#[cfg(feature = "dynamic-help")]
	/// # Handle Help.
	fn help_flag(&self) -> Option<ArgyleError> {
		if 0 != self.flags & FLAG_ANY_HELP {
			// Help is requested!
			if 0 != self.flags & FLAG_HAS_HELP || self.args[0] == b"help" {
				// Static help.
				if 0 != self.flags & FLAG_HELP {
					return Some(ArgyleError::WantsHelp);
				}

				// Dynamic help.
				return Some(ArgyleError::WantsDynamicHelp(
					if ! self.args[0].is_empty() && self.args[0][0] != b'-' && self.args[0] != b"help" {
						Some(self.args[0].clone().into_boxed_slice())
					}
					else { None }
				));
			}
		}

		None
	}

	#[cfg(not(feature = "dynamic-help"))]
	#[inline]
	/// # Handle Help.
	fn help_flag(&self) -> Option<ArgyleError> {
		if
			0 != self.flags & FLAG_ANY_HELP &&
			(0 != self.flags & FLAG_HAS_HELP || self.args[0] == b"help")
		{
				return Some(ArgyleError::WantsHelp);
		}

		None
	}

	#[must_use]
	/// # Add Arguments From a Text File.
	///
	/// When chained to `new()`, if either "-l" or "--list" options are found,
	/// the subsequent value (if any) is read as a text file, and each non-
	/// empty line within is appended to the set as additional arguments,
	/// exactly as if they were provided directly.
	///
	/// No judgments are passed on the contents of the file. If a line has
	/// length, it is appended.
	///
	/// Note: if using this approach to seed a command with file paths, make
	/// sure those paths are absolute as their relativity will likely be lost
	/// in translation.
	///
	/// This method always transparently returns `self`.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use argyle::Argue;
	///
	/// let mut args = Argue::new(0).unwrap().with_list();
	/// ```
	pub fn with_list(mut self) -> Self {
		if let Some(raw) = self.option2_os(b"-l", b"--list").and_then(|p| std::fs::read_to_string(p).ok()) {
			for line in raw.lines() {
				let bytes = line.trim().as_bytes();
				if ! bytes.is_empty() {
					self.args.push(bytes.to_vec());
				}
			}
		}

		self
	}
}

/// ## Casting.
///
/// These methods convert `Argue` into different data structures.
impl Argue {
	#[allow(clippy::missing_const_for_fn)] // Doesn't work!
	#[must_use]
	#[inline]
	/// # Into Owned Vec.
	///
	/// Use this method to consume the struct and return the parsed arguments
	/// as a `Vec<Vec<u8>>`.
	///
	/// If you merely want something to iterate over, you can alternatively
	/// dereference the struct to a string slice.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use argyle::Argue;
	///
	/// let args: Vec<Vec<u8>> = Argue::new(0).unwrap().take();
	/// ```
	pub fn take(self) -> Vec<Vec<u8>> { self.args }
}

/// ## Queries.
///
/// These methods allow data to be questioned and extracted.
impl Argue {
	#[must_use]
	#[inline]
	/// # First Entry.
	///
	/// Borrow the first entry, if any.
	///
	/// ## Examples
	///
	/// ```ignore
	/// use argyle::Argue;
	///
	/// let mut args = Argue::new(0);
	///
	/// if let Some("happy") = args.peek() { ... }
	/// ```
	pub fn peek(&self) -> Option<&[u8]> { self.args.get(0).map(Vec::as_slice) }

	#[allow(unsafe_code)]
	#[must_use]
	#[inline]
	/// # First Entry.
	///
	/// Borrow the first entry without first checking for its existence.
	///
	/// ## Safety
	///
	/// This assumes a first argument exists; it will panic if the set is
	/// empty.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use argyle::{Argue, FLAG_REQUIRED};
	///
	/// let args = Argue::new(FLAG_REQUIRED).unwrap();
	///
	/// // This is actually safe because FLAG_REQUIRED would have errored out
	/// // if nothing were present.
	/// let first: &[u8] = unsafe { args.peek_unchecked() };
	/// ```
	pub unsafe fn peek_unchecked(&self) -> &[u8] { &self.args[0] }

	#[must_use]
	#[inline]
	/// # Switch.
	///
	/// Returns `true` if the switch is present, `false` if not.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use argyle::Argue;
	///
	/// let mut args = Argue::new(0).unwrap();
	/// let switch: bool = args.switch(b"--my-switch");
	/// ```
	pub fn switch(&self, key: &[u8]) -> bool {
		self.keys.iter()
			.take(self.num_keys())
			.map(|x| &self.args[*x as usize])
			.any(|x| x == key)
	}

	#[must_use]
	#[inline]
	/// # Switch x2.
	///
	/// This is a convenience method that checks for the existence of two
	/// switches at once, returning `true` if either are present. Generally
	/// you would use this for a flag that has both a long and short version.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use argyle::Argue;
	///
	/// let mut args = Argue::new(0).unwrap();
	/// let switch: bool = args.switch2(b"-s", b"--my-switch");
	/// ```
	pub fn switch2(&self, short: &[u8], long: &[u8]) -> bool {
		self.keys.iter()
			.take(self.num_keys())
			.map(|x| &self.args[*x as usize])
			.any(|x| {
				x == short || x == long
			})
	}

	#[must_use]
	/// # Switches As Bitflags.
	///
	/// If you have a lot of switches that directly correspond to bitflags, you
	/// can pass them all to this method and receive the appropriate combined
	/// flag value back.
	///
	/// This does not conflict with [`Argue::switch`]; if some of your flags
	/// require special handling you can mix-and-match calls.
	///
	/// Note: the default value of `N` is used as a starting point. For `u8`,
	/// `u16`, etc., that's just `0`, but if using a custom type, make sure its
	/// default state is the equivalent of "no flags".
	///
	/// ## Examples
	///
	/// ```no_run
	/// use argyle::Argue;
	///
	/// let mut args = Argue::new(0).unwrap();
	/// let flags: u8 = args.bitflags([
	///     (&b"-o"[..], 0b0000_0001),
	///     (&b"-t"[..], 0b0000_0010),
	/// ]);
	/// ```
	pub fn bitflags<'a, N, I>(&self, pairs: I) -> N
	where
		N: BitOr<Output = N> + Default,
		I: IntoIterator<Item=(&'a [u8], N)>
	{
		pairs.into_iter()
			.fold(N::default(), |flags, (switch, flag)|
				if self.switch(switch) { flags | flag }
				else { flags }
			)
	}

	/// # Option.
	///
	/// Return the value corresponding to `key`, if present. "Value" in this
	/// case means the entry immediately following the key. Multi-value
	/// options are not supported.
	///
	/// Note: this method is the only way `Argue` knows whether or not a key
	/// is an option (with a value) or a switch. Be sure to request all
	/// possible options *before* requesting the trailing arguments to ensure
	/// the division between named and trailing is properly set.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use argyle::Argue;
	///
	/// let mut args = Argue::new(0).unwrap();
	/// let opt: Option<&[u8]> = args.option(b"--my-opt");
	/// ```
	pub fn option(&self, key: &[u8]) -> Option<&[u8]> {
		self.keys.iter()
			.take(self.num_keys())
			.position(|&x| self.args.get(x as usize).map_or(false, |x| x == key))
			.and_then(|idx| {
				let idx = self.keys[idx] + 1;
				self.args.get(idx as usize).map(|x| {
					if idx > self.last.get() { self.last.set(idx); }
					x.as_slice()
				})
			})
	}

	/// # Option x2.
	///
	/// This is a convenience method that checks for the existence of two
	/// options at once, returning the first found value, if any. Generally
	/// you would use this for a flag that has both a long and short version.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use argyle::Argue;
	///
	/// let mut args = Argue::new(0).unwrap();
	/// let opt: Option<&[u8]> = args.option2(b"-o", b"--my-opt");
	/// ```
	pub fn option2(&self, short: &[u8], long: &[u8]) -> Option<&[u8]> {
		self.keys.iter()
			.take(self.num_keys())
			.position(|&x| self.args.get(x as usize).map_or(false, |x| {
				x == short || x == long
			}))
			.and_then(|idx| {
				let idx = self.keys[idx] + 1;
				self.args.get(idx as usize).map(|x| {
					if idx > self.last.get() { self.last.set(idx); }
					x.as_slice()
				})
			})
	}

	#[must_use]
	/// # Trailing Arguments.
	///
	/// This returns a slice from the end of the result set assumed to
	/// represent unnamed arguments. The boundary for the split is determined
	/// by the position of the last known key (or key value).
	///
	/// It is important to query any expected options prior to calling this
	/// method, as the existence of those options might shift the boundary.
	///
	/// If there are no arguments, an empty slice is returned.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use argyle::Argue;
	///
	/// let mut args = Argue::new(0).unwrap();
	/// let extras: &[Vec<u8>] = args.args();
	/// ```
	pub fn args(&self) -> &[Vec<u8>] {
		let idx = self.arg_idx();
		if idx < self.args.len() {
			&self.args[self.arg_idx()..]
		}
		else { &[] }
	}

	#[must_use]
	/// # Arg at Index.
	///
	/// Pluck the nth trailing argument by index (starting from zero).
	///
	/// Note, this is different than dereferencing the whole `Argue` struct
	/// and requesting its zero index; that would refer to the first CLI
	/// argument of any kind, which could be a subcommand or key.
	pub fn arg(&self, idx: usize) -> Option<&[u8]> {
		let start_idx = self.arg_idx();
		if start_idx + idx < self.args.len() {
			Some(&self.args[start_idx + idx])
		}
		else { None }
	}

	/// # First Trailing Argument.
	///
	/// Return the first trailing argument, or print an error and exit the
	/// thread if there isn't one.
	///
	/// As with other arg-related methods, it is important to query all options
	/// first, as that helps the struct determine the boundary between named
	/// and unnamed values.
	///
	/// ## Errors
	///
	/// This method will return an error if there is no first argument.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use argyle::Argue;
	///
	/// let mut args = Argue::new(0).unwrap();
	/// let opt: &[u8] = args.first_arg().unwrap();
	/// ```
	pub fn first_arg(&self) -> Result<&[u8], ArgyleError> {
		let idx = self.arg_idx();
		if idx >= self.args.len() { Err(ArgyleError::NoArg) }
		else { Ok(&self.args[idx]) }
	}
}

/// # `OsStr` Methods.
impl Argue {
	#[must_use]
	/// # Option as `OsStr`.
	///
	/// This works just like [`Argue::option`], except it returns the value as
	/// an [`OsStr`](std::ffi::OsStr) instead of bytes.
	pub fn option_os(&self, key: &[u8]) -> Option<&OsStr> {
		self.option(key).map(OsStr::from_bytes)
	}

	#[must_use]
	/// # Option x2 as `OsStr`.
	///
	/// This works just like [`Argue::option2`], except it returns the value as
	/// an [`OsStr`](std::ffi::OsStr) instead of bytes.
	pub fn option2_os(&self, short: &[u8], long: &[u8]) -> Option<&OsStr> {
		self.option2(short, long).map(OsStr::from_bytes)
	}

	#[must_use]
	/// # Trailing Arguments as `OsStr`.
	///
	/// This works just like [`Argue::args`], except it returns an iterator
	/// that yields [`OsStr`](std::ffi::OsStr) instead of bytes.
	pub fn args_os(&self) -> ArgsOsStr {
		ArgsOsStr::new(self.args())
	}

	#[must_use]
	/// # Arg at Index as `OsStr`.
	///
	/// This works just like [`Argue::arg`], except it returns the value as an
	/// [`OsStr`](std::ffi::OsStr) instead of bytes.
	pub fn arg_os(&self, idx: usize) -> Option<&OsStr> {
		self.arg(idx).map(OsStr::from_bytes)
	}

	/// # First Trailing Argument as `OsStr`
	///
	/// This works just like [`Argue::first_arg`] except it returns the value
	/// as an [`OsStr`](std::ffi::OsStr) instead of bytes.
	///
	/// ## Errors
	///
	/// This method will return an error if there is no first argument.
	pub fn first_arg_os(&self) -> Result<&OsStr, ArgyleError> {
		self.first_arg().map(OsStr::from_bytes)
	}
}

/// ## Internal Helpers.
impl Argue {
	#[inline]
	/// # Arg Index.
	///
	/// This is an internal method that returns the index at which the first
	/// unnamed argument may be found.
	///
	/// Note: the index may be out of range, but won't be used in that case.
	fn arg_idx(&self) -> usize {
		if self.keys[KEY_LEN] == 0 && 0 == self.flags & FLAG_SUBCOMMAND { 0 }
		else { self.last.get() as usize + 1 }
	}

	#[inline]
	/// # Num Keys.
	const fn num_keys(&self) -> usize { self.keys[KEY_LEN] as usize }
}



/// # Key/Value Iterator Adapter.
///
/// This parses arguments in preparation for `kv_argue` to build an actual
/// [`Argue`] instance.
fn kv_adapter(mut src: Vec<u8>) -> KvIterItem {
	match KeyKind::from(src.as_slice()) {
		KeyKind::None => (true, src, None),
		KeyKind::Short | KeyKind::Long => (false, src, None),
		KeyKind::ShortV => {
			let b = src.split_off(2);
			(false, src, Some(b))
		},
		KeyKind::LongV(x) => {
			let end = x.get() as usize;
			if end + 1 < src.len() {
				let b = src.split_off(end + 1);
				src.truncate(end);
				(false, src, Some(b))
			}
			else {
				src.truncate(end);
				(false, src, Some(Vec::new()))
			}
		}
	}
}



#[cfg(test)]
mod tests {
	use super::*;
	use brunch as _;

	/// # Argue From Key/Value Iterator.
	///
	/// This mimics [`Argue::new`], allowing us to populate an object with
	/// arbitrary arguments for testing.
	///
	/// It would be nice to not have to duplicate this, but it hurts runtime
	/// performance, so whatever. Haha.
	fn kv_argue<T>(src: T) -> Result<Argue, ArgyleError>
	where T: Iterator<Item = KvIterItem> {
		let mut args: Vec<Vec<u8>> = Vec::with_capacity(KEY_SIZE);
		let mut keys = [0_u16; KEY_SIZE];
		let mut last = 0_u16;
		let mut idx = 0_u16;
		let mut flags = 0_u8;

		for (standalone, key, value) in src {
			// Skip leading empties.
			if 0 == idx && (key.is_empty() || key.iter().all(u8::is_ascii_whitespace)) {
				continue;
			}

			// How many spots will this take up?
			let inc: u16 =
				if value.is_some() { 2 }
				else if key == b"--" { break; }
				else { 1 };

			// Make sure we fit.
			if u16::MAX - inc < idx { return Err(ArgyleError::TooManyArgs); }

			// Just an arg?
			if standalone { idx += inc; }
			// Do key stuff!
			else {
				if value.is_none() {
					match key.as_slice() {
						b"-V" | b"--version" => { flags |= FLAG_HAS_VERSION; },
						b"-h" | b"--help" => { flags |= FLAG_HAS_HELP; },
						_ => {},
					}
				}

				let num_keys = keys[KEY_LEN] as usize;
				if num_keys == KEY_LEN { return Err(ArgyleError::TooManyKeys); }

				keys[num_keys] = idx;
				keys[KEY_LEN] += 1;

				idx += inc;
				last = idx - 1;
			}

			// Push the things.
			args.push(key);
			if let Some(v) = value { args.push(v); }
		}

		Ok(Argue {
			args,
			keys,
			last: Cell::new(last),
			flags,
		})
	}

	/// # Key/Value Iterator Adapter.
	fn kv_ref_adapter(src: &'static [u8]) -> KvIterItem {
		kv_adapter(src.to_vec())
	}

	#[test]
	fn t_parse_args() {
		let mut base: Vec<&[u8]> = vec![
			b"hey",
			b"-kVal",
			b"--empty=",
			b"--key=Val",
		];

		let mut args = kv_argue(base.iter().copied().map(kv_ref_adapter)).unwrap();

		// Check the overall structure.
		assert_eq!(
			*args,
			[
				b"hey"[..].to_vec(),
				b"-k"[..].to_vec(),
				b"Val"[..].to_vec(),
				b"--empty"[..].to_vec(),
				vec![],
				b"--key"[..].to_vec(),
				b"Val"[..].to_vec(),
			]
		);

		// Test the finders.
		assert_eq!(args.peek(), Some(&b"hey"[..]));
		assert!(args.switch(b"-k"));
		assert!(args.switch(b"--key"));
		assert!(args.switch2(b"-k", b"--key"));
		assert_eq!(args.option(b"--key"), Some(&b"Val"[..]));
		assert_eq!(args.option2(b"-k", b"--key"), Some(&b"Val"[..]));
		assert!(args.args().is_empty());

		// Let's test a first-position key.
		base.insert(0, b"--prefix");
		args = kv_argue(base.iter().copied().map(kv_ref_adapter)).unwrap();

		// The whole thing again.
		assert_eq!(
			*args,
			[
				b"--prefix"[..].to_vec(),
				b"hey"[..].to_vec(),
				b"-k"[..].to_vec(),
				b"Val"[..].to_vec(),
				b"--empty"[..].to_vec(),
				vec![],
				b"--key"[..].to_vec(),
				b"Val"[..].to_vec(),
			]
		);

		assert_eq!(args.peek(), Some(&b"--prefix"[..]));
		assert!(args.switch(b"--prefix"));
		assert_eq!(args.option(b"--prefix"), Some(&b"hey"[..]));

		// Something that doesn't exist.
		assert_eq!(args.option(b"foo"), None);

		// Let's see what trailing args look like when there are none.
		assert!(args.first_arg().is_err());
		assert_eq!(args.arg(0), None);

		// Let's also make sure the trailing arguments work too.
		let trailing: &[&[u8]] = &[b"Hello", b"World"];
		base.extend_from_slice(trailing);
		args = kv_argue(base.iter().copied().map(kv_ref_adapter)).unwrap();
		assert_eq!(args.first_arg(), Ok(&b"Hello"[..]));
		assert_eq!(args.arg(0), Some(&b"Hello"[..]));
		assert_eq!(args.arg(1), Some(&b"World"[..]));
		assert_eq!(args.arg(2), None);
		assert_eq!(args.args(), trailing);
	}

	#[test]
	fn t_version() {
		let mut base: Vec<&[u8]> = vec![
			b"hey",
			b"-V",
		];

		// We should be wanting a version.
		let mut args = kv_argue(base.iter().copied().map(kv_ref_adapter)).unwrap();

		assert!(matches!(
			args.check_flags(FLAG_VERSION),
			Err(ArgyleError::WantsVersion)
		));

		// Same thing without the version flag.
		args = kv_argue(base.iter().copied().map(kv_ref_adapter)).unwrap();
		assert!(args.check_flags(FLAG_HELP).is_ok());

		// Repeat with the long flag.
		base[1] = b"--version";

		// We should be wanting a version.
		args = kv_argue(base.iter().copied().map(kv_ref_adapter)).unwrap();
		assert!(matches!(
			args.check_flags(FLAG_VERSION),
			Err(ArgyleError::WantsVersion)
		));

		// Same thing without the version flag.
		args = kv_argue(base.iter().copied().map(kv_ref_adapter)).unwrap();
		assert!(args.check_flags(FLAG_HELP).is_ok());

		// One last time without a version arg present.
		base[1] = b"--ok";

		// We should be wanting a version.
		args = kv_argue(base.iter().copied().map(kv_ref_adapter)).unwrap();
		assert!(args.check_flags(FLAG_VERSION).is_ok());

		// Same thing without the version flag.
		args = kv_argue(base.iter().copied().map(kv_ref_adapter)).unwrap();
		assert!(args.check_flags(FLAG_HELP).is_ok());
	}

	#[test]
	fn t_help() {
		let mut base: Vec<&[u8]> = vec![
			b"hey",
			b"-h",
		];

		// We should be wanting a static help.
		let mut args = kv_argue(base.iter().copied().map(kv_ref_adapter)).unwrap();
		assert!(matches!(
			args.check_flags(FLAG_HELP),
			Err(ArgyleError::WantsHelp)
		));

		#[cfg(feature = "dynamic-help")]
		{
			// Dynamic help this time.
			args = kv_argue(base.iter().copied().map(kv_ref_adapter)).unwrap();
			match args.check_flags(FLAG_DYNAMIC_HELP) {
				Err(ArgyleError::WantsDynamicHelp(e)) => {
					let expected: Option<Box<[u8]>> = Some(Box::from(&b"hey"[..]));
					assert_eq!(e, expected);
				},
				_ => panic!("Test should have produced an error with Some(Box(hey))."),
			}
		}

		// Same thing without wanting help.
		args = kv_argue(base.iter().copied().map(kv_ref_adapter)).unwrap();
		assert!(args.check_flags(FLAG_VERSION).is_ok());

		// Again with help flag first.
		base[0] = b"--help";

		// We should be wanting a static help.
		args = kv_argue(base.iter().copied().map(kv_ref_adapter)).unwrap();
		assert!(matches!(
			args.check_flags(FLAG_HELP),
			Err(ArgyleError::WantsHelp)
		));

		#[cfg(feature = "dynamic-help")]
		{
			args = kv_argue(base.iter().copied().map(kv_ref_adapter)).unwrap();
			// Dynamic help this time.
			assert!(matches!(
				args.check_flags(FLAG_DYNAMIC_HELP),
				Err(ArgyleError::WantsDynamicHelp(None))
			));
		}

		// Same thing without wanting help.
		args = kv_argue(base.iter().copied().map(kv_ref_adapter)).unwrap();
		assert!(args.check_flags(FLAG_VERSION).is_ok());

		// Same thing without wanting help.
		base[0] = b"help";
		base[1] = b"--foo";

		// We should be wanting a static help.
		args = kv_argue(base.iter().copied().map(kv_ref_adapter)).unwrap();
		assert!(matches!(
			args.check_flags(FLAG_HELP),
			Err(ArgyleError::WantsHelp)
		));

		#[cfg(feature = "dynamic-help")]
		{
			args = kv_argue(base.iter().copied().map(kv_ref_adapter)).unwrap();
			// Dynamic help this time.
			assert!(matches!(
				args.check_flags(FLAG_DYNAMIC_HELP),
				Err(ArgyleError::WantsDynamicHelp(None))
			));
		}

		// Same thing without wanting help.
		args = kv_argue(base.iter().copied().map(kv_ref_adapter)).unwrap();
		assert!(args.check_flags(FLAG_VERSION).is_ok());

		// One last time with no helpish things.
		base[0] = b"hey";

		// We should be wanting a static help.
		args = kv_argue(base.iter().copied().map(kv_ref_adapter)).unwrap();
		assert!(args.check_flags(FLAG_HELP).is_ok());

		#[cfg(feature = "dynamic-help")]
		{
			// Dynamic help this time.
			args = kv_argue(base.iter().copied().map(kv_ref_adapter)).unwrap();
			assert!(args.check_flags(FLAG_DYNAMIC_HELP).is_ok());
		}

		// Same thing without wanting help.
		args = kv_argue(base.iter().copied().map(kv_ref_adapter)).unwrap();
		assert!(args.check_flags(FLAG_VERSION).is_ok());
	}

	#[test]
	fn t_bitflags() {
		const FLAG_EMPTY: u8 =    0b0000_0001;
		const FLAG_HELLO: u8 =    0b0000_0010;
		const FLAG_K: u8 =        0b0000_0100;
		const FLAG_ONE_MORE: u8 = 0b0000_1000;
		const FLAG_OTHER: u8 =    0b0001_0000;

		let base: Vec<&[u8]> = vec![
			b"hey",
			b"-k",
			b"--empty",
			b"--key=Val",
			b"--hello",
			b"--one-more",
		];

		let args = kv_argue(base.iter().copied().map(kv_ref_adapter)).unwrap();
		let flags: u8 = args.bitflags([
			(&b"-k"[..], FLAG_K),
			(&b"--empty"[..], FLAG_EMPTY),
			(&b"--hello"[..], FLAG_HELLO),
			(&b"--one-more"[..], FLAG_ONE_MORE),
			(&b"--other"[..], FLAG_OTHER),
		]);

		assert_eq!(flags & FLAG_K, FLAG_K);
		assert_eq!(flags & FLAG_EMPTY, FLAG_EMPTY);
		assert_eq!(flags & FLAG_HELLO, FLAG_HELLO);
		assert_eq!(flags & FLAG_ONE_MORE, FLAG_ONE_MORE);
		assert_eq!(flags & FLAG_OTHER, 0);
	}

	#[test]
	#[cfg_attr(miri, ignore)]
	fn t_overflow() {
		// Let's start with one-too-many elements.
		let mut nope: Vec<&[u8]> = (0..65536_u32).into_iter()
			.map(|_x| b"hi".as_slice())
			.collect();

		// We can't exceed u16::MAX elements.
		assert!(kv_argue(nope.iter().copied().map(kv_ref_adapter)).is_err());

		// This is an awful lot of arguments, but should fit now!
		nope.pop();
		assert_eq!(nope.len(), 65535);
		assert!(kv_argue(nope.iter().copied().map(kv_ref_adapter)).is_ok());

		// We also can't have more than 15 keys.
		nope.truncate(0);
		for _ in 0..16 {
			nope.push(b"-h");
		}
		assert!(kv_argue(nope.iter().copied().map(kv_ref_adapter)).is_err());

		// But if we remove one it should work.
		nope.pop();
		assert_eq!(nope.len(), 15);
		assert!(kv_argue(nope.iter().copied().map(kv_ref_adapter)).is_ok());
	}
}
