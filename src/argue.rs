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
	ffi::{
		OsStr,
		OsString,
	},
	ops::{
		BitOr,
		Deref,
		Index,
	},
	os::unix::ffi::{
		OsStrExt,
		OsStringExt,
	},
};



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



#[derive(Debug, Clone, Default)]
/// `Argue` is an agnostic CLI argument parser. Unlike more robust libraries
/// like [clap](https://crates.io/crates/clap), `Argue` does not hold
/// information about expected or required arguments; it merely parses the raw
/// arguments into a consistent state so the implementor can query them as
/// needed.
///
/// (It is effectively a wrapper around [`std::env::args_os`].)
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
/// 2. Argument parsing stops if a passthrough separator `--` is found. Anything up to that point is parsed as usual; everything after is discarded.
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
/// let arg_slice: &[Vec<u8>] = &args;
/// ```
///
/// Or it can be converted into an owned Vector:
/// ```ignore
/// let args: Vec<Vec<u8>> = args.take();
/// ```
pub struct Argue {
	/// Parsed arguments.
	args: Vec<Vec<u8>>,

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
	last: Cell<usize>,

	/// Flags.
	flags: u8,
}

impl Deref for Argue {
	type Target = [Vec<u8>];
	#[inline]
	fn deref(&self) -> &Self::Target { &self.args }
}

impl<'a> FromIterator<&'a [u8]> for Argue {
	fn from_iter<I: IntoIterator<Item = &'a [u8]>>(src: I) -> Self {
		src.into_iter().map(<[u8]>::to_vec).collect()
	}
}

impl FromIterator<Vec<u8>> for Argue {
	fn from_iter<I: IntoIterator<Item = Vec<u8>>>(src: I) -> Self {
		src.into_iter().map(OsString::from_vec).collect()
	}
}

impl FromIterator<OsString> for Argue {
	fn from_iter<I: IntoIterator<Item = OsString>>(src: I) -> Self {
		let mut args: Vec<Vec<u8>> = Vec::with_capacity(16);
		let mut last = 0_usize;
		let mut flags = 0_u8;
		let mut idx = 0_usize;

		for a in src {
			let mut a = a.into_vec();
			let key: &[u8] = a.as_slice();

			// Skip leading empties.
			if 0 == idx && (key.is_empty() || key.iter().all(u8::is_ascii_whitespace)) {
				continue;
			}

			match KeyKind::from(key) {
				KeyKind::None => {
					if key == b"--" { break; } // Stop on separator.

					args.push(a);
					idx += 1;
				},
				KeyKind::Short => {
					if key == b"-V" { flags |= FLAG_HAS_VERSION; }
					else if key == b"-h" { flags |= FLAG_HAS_HELP; }

					args.push(a);
					last = idx;
					idx += 1;
				},
				KeyKind::Long => {
					if key == b"--version" { flags |= FLAG_HAS_VERSION; }
					else if key == b"--help" { flags |= FLAG_HAS_HELP; }

					args.push(a);
					last = idx;
					idx += 1;
				},
				KeyKind::ShortV => {
					let b = a.split_off(2);
					args.push(a);
					args.push(b);
					last = idx + 1;
					idx += 2;
				},
				KeyKind::LongV(end) => {
					let b =
						if end + 1 < key.len() { a.split_off(end + 1) }
						else { Vec::new() };
					a.truncate(end); // Chop off the equal sign.
					args.push(a);
					args.push(b);
					last = idx + 1;
					idx += 2;
				},
			}
		}

		// Turn it into an object!
		Self {
			args,
			last: Cell::new(last),
			flags,
		}
	}
}

impl Index<usize> for Argue {
	type Output = [u8];

	#[inline]
	/// # Argument by Index.
	///
	/// This returns the nth CLI argument, which could be a subcommand, key,
	/// value, or trailing argument.
	///
	/// If you're only interested in trailing arguments, use [`Argue::arg`]
	/// instead.
	///
	/// If you want everything, you can alternatively dereference [`Argue`]
	/// into a slice.
	///
	/// ## Panics
	///
	/// This will panic if the index is out of range. Use [`Argue::len`] to
	/// confirm the length ahead of time, or [`Argue::get`], which wraps the
	/// answer in an `Option` instead of panicking.
	fn index(&self, idx: usize) -> &Self::Output { &self.args[idx] }
}

/// ## Instantiation and Builder Patterns.
impl Argue {
	#[inline]
	/// # New Instance.
	///
	/// This simply parses the owned output of [`std::env::args_os`].
	///
	/// ## Examples
	///
	/// ```
	/// use argyle::{Argue, ArgyleError, FLAG_VERSION};
	///
	/// // Parse, but abort if -V/--version is present.
	/// let args = match Argue::new(FLAG_VERSION) {
	///     Ok(a) => a, // No abort.
	///     // The version flags were present.
	///     Err(ArgyleError::WantsVersion) => {
	///         println!("MyApp v{}", env!("CARGO_PKG_VERSION"));
	///         return;
	///     },
	///     // This probably won't happen with only FLAG_VERSION set, but just
	///     // in case…
	///     Err(e) => {
	///         println!("Error: {}", e);
	///         return;
	///     },
	/// };
	///
	/// // If we're here, check whatever random args your program needs.
	/// let quiet: bool = args.switch(b"-q");
	/// let foo: Option<&[u8]> = args.option2(b"-f", b"--foo");
	/// ```
	///
	/// ## Errors
	///
	/// This method's result may represent an actual error, or some form of
	/// abort, such as the presence of `-V`/`--version` when `FLAG_VERSION`
	/// was passed to the constructor.
	///
	/// Generally you'd want to match the specific [`ArgyleError`] variant to
	/// make sure you're taking the appropriate action.
	pub fn new(chk_flags: u8) -> Result<Self, ArgyleError> {
		let mut out: Self = std::env::args_os().skip(1).collect();
		out.check_flags(chk_flags)?;
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
				if FLAG_REQUIRED == self.flags & FLAG_REQUIRED {
					return Err(ArgyleError::Empty);
				}
			}
			// Print version.
			else if FLAG_DO_VERSION == self.flags & FLAG_DO_VERSION {
				return Err(ArgyleError::WantsVersion);
			}
			// Help.
			else if
				0 != self.flags & FLAG_ANY_HELP &&
				(FLAG_HAS_HELP == self.flags & FLAG_HAS_HELP || self.args[0] == b"help")
			{
				#[cfg(feature = "dynamic-help")]
				if FLAG_DYNAMIC_HELP == self.flags & FLAG_DYNAMIC_HELP {
					return Err(ArgyleError::WantsDynamicHelp(
						if self.args[0][0] != b'-' && self.args[0] != b"help" {
							Some(Box::from(self.args[0].as_slice()))
						}
						else { None }
					));
				}

				return Err(ArgyleError::WantsHelp);
			}
		}

		Ok(())
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
	pub fn switch(&self, key: &[u8]) -> bool { self.args.iter().any(|x| x == key) }

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
		self.args.iter().any(|x| x == short || x == long)
	}

	#[must_use]
	/// # Switch By Prefix.
	///
	/// If you have multiple, mutually exclusive switches that all begin with
	/// the same prefix, this method can be used to quickly return the first
	/// match (stripped of the common prefix).
	///
	/// If no match is found, or an _exact_ match is found — i.e. leaving the
	/// key empty — `None` is returned.
	///
	/// Do not use this if you have options sharing this prefix; `Argue`
	/// doesn't know the difference so will simply return whatever it finds
	/// first.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use argyle::Argue;
	///
	/// let mut args = Argue::new(0).unwrap();
	/// match args.switch_by_prefix(b"--dump-") {
	///     Some(b"addresses") => {}, // --dump-addresses
	///     Some(b"names") => {},     // --dump-names
	///     _ => {},                  // No matches.
	/// }
	/// ```
	pub fn switch_by_prefix(&self, prefix: &[u8]) -> Option<&[u8]> {
		if prefix.is_empty() { None }
		else {
			self.args.iter().find_map(|x| {
				let key = x.strip_prefix(prefix)?;
				if key.is_empty() { None }
				else { Some(key) }
			})
		}
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
		let idx = self.args.iter().position(|x| x == key)? + 1;
		self._option(idx)
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
		let idx = self.args.iter().position(|x| x == short || x == long)? + 1;
		self._option(idx)
	}

	#[must_use]
	/// # Option By Prefix.
	///
	/// If you have multiple, mutually exclusive options that all begin with
	/// the same prefix, this method can be used to quickly return the first
	/// matching key (stripped of the common prefix) and value.
	///
	/// If no match is found, an _exact_ match is found — i.e. leaving the
	/// key empty — or no value follows, `None` is returned.
	///
	/// Do not use this if you have switches sharing this prefix; `Argue`
	/// doesn't know the difference so will simply return whatever it finds
	/// first.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use argyle::Argue;
	///
	/// let mut args = Argue::new(0).unwrap();
	/// match args.option_by_prefix(b"--color-") {
	///     Some((b"solid", val)) => {},  // --color-solid, green
	///     Some((b"dashed", val)) => {}, // --color-dashed, blue
	///     _ => {},                      // No matches.
	/// }
	/// ```
	pub fn option_by_prefix(&self, prefix: &[u8]) -> Option<(&[u8], &[u8])> {
		if prefix.is_empty() { None }
		else {
			let (idx, key) = self.args.iter()
				.enumerate()
				.find_map(|(idx, x)| {
					let key = x.strip_prefix(prefix)?;
					if key.is_empty() { None }
					else { Some((idx, key)) }
				})?;

			let val = self._option(idx + 1)?;
			Some((key, val))
		}
	}

	/// # Return Option at Index.
	///
	/// This method holds the common code for [`Argue::option`] and
	/// [`Argue::option2`]. It returns the argument at the index they find,
	/// nudging the options/args boundary upward if needed.
	///
	/// This will return `None` if the index is out of range.
	fn _option(&self, idx: usize) -> Option<&[u8]> {
		let arg = self.args.get(idx)?;
		if self.last.get() < idx { self.last.set(idx); }
		Some(arg.as_slice())
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
		if idx < self.args.len() { &self.args[idx..] }
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
		self.args.get(start_idx + idx).map(Vec::as_slice)
	}
}

/// ## Misc Indexing.
impl Argue {
	#[must_use]
	/// # Get Argument.
	///
	/// This is the non-panicking way to index into a specific subcommand, key,
	/// value, etc. It will be returned if it exists, otherwise you'll get `None`
	/// if the index is out of range.
	///
	/// If you _know_ the index is valid, you can leverage the `std::ops::Index`
	/// trait to fetch the value directly.
	pub fn get(&self, idx: usize) -> Option<&[u8]> {
		self.args.get(idx).map(Vec::as_slice)
	}

	#[inline]
	#[must_use]
	/// # Is Empty?
	pub fn is_empty(&self) -> bool { self.args.is_empty() }

	#[inline]
	#[must_use]
	/// # Length.
	///
	/// Return the length of all the arguments (keys, values, etc.) held by
	/// the instance.
	pub fn len(&self) -> usize { self.args.len() }
}

/// # `OsStr` Methods.
impl Argue {
	#[must_use]
	/// # Switch Starting With… as `OsStr`.
	///
	/// This works just like [`Argue::switch_by_prefix`], except it returns the
	/// value as an [`OsStr`](std::ffi::OsStr) instead of bytes.
	pub fn switch_by_prefix_os(&self, prefix: &[u8]) -> Option<&OsStr> {
		self.switch_by_prefix(prefix).map(OsStr::from_bytes)
	}

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
	/// # Option Starting With… as `OsStr`.
	///
	/// This works just like [`Argue::option_by_prefix`], except it returns the
	/// key/value as [`OsStr`](std::ffi::OsStr) instead of bytes.
	pub fn option_by_prefix_os(&self, prefix: &[u8]) -> Option<(&OsStr, &OsStr)> {
		self.option_by_prefix(prefix)
			.map(|(k, v)| (OsStr::from_bytes(k), OsStr::from_bytes(v)))
	}

	#[must_use]
	/// # Trailing Arguments as `OsStr`.
	///
	/// This works just like [`Argue::args`], except it returns an iterator
	/// that yields [`OsStr`](std::ffi::OsStr) instead of bytes.
	pub fn args_os(&self) -> ArgsOsStr { ArgsOsStr::new(self.args()) }

	#[must_use]
	/// # Arg at Index as `OsStr`.
	///
	/// This works just like [`Argue::arg`], except it returns the value as an
	/// [`OsStr`](std::ffi::OsStr) instead of bytes.
	pub fn arg_os(&self, idx: usize) -> Option<&OsStr> {
		self.arg(idx).map(OsStr::from_bytes)
	}
}

/// ## Internal Helpers.
impl Argue {
	/// # Arg Index.
	///
	/// This is an internal method that returns the index at which the first
	/// unnamed argument may be found.
	///
	/// Note: the index may be out of range, but won't be used in that case.
	fn arg_idx(&self) -> usize {
		let last = self.last.get();
		if 0 == last && 0 == self.flags & FLAG_SUBCOMMAND { 0 }
		else { last + 1 }
	}
}



#[cfg(test)]
mod tests {
	use super::*;
	use brunch as _;

	#[test]
	fn t_parse_args() {
		let mut base: Vec<&[u8]> = vec![
			b"hey",
			b"-kVal",
			b"--empty=",
			b"--key=Val",
		];

		let mut args: Argue = base.iter().copied().collect();

		// Check the overall structure.
		assert_eq!(
			*args,
			[
				b"hey".to_vec(),
				b"-k".to_vec(),
				b"Val".to_vec(),
				b"--empty".to_vec(),
				vec![],
				b"--key".to_vec(),
				b"Val".to_vec(),
			]
		);

		// Test the finders.
		assert_eq!(args.get(0), Some(&b"hey"[..]));

		assert_eq!(&args[1], b"-k");
		assert!(args.switch(b"-k"));
		assert!(args.switch(b"--key"));
		assert!(args.switch2(b"-k", b"--key"));

		assert_eq!(args.option(b"--key"), Some(&b"Val"[..]));
		assert_eq!(args.option2(b"-k", b"--key"), Some(&b"Val"[..]));
		assert!(args.args().is_empty());

		// These shouldn't exist.
		assert!(! args.switch(b"-c"));
		assert!(! args.switch2(b"-c", b"--copy"));
		assert!(args.option(b"-c").is_none());
		assert!(args.option2(b"-c", b"--copy").is_none());
		assert!(args.get(100).is_none());

		// Let's test a first-position key.
		base.insert(0, b"--prefix");
		args = base.iter().copied().collect();

		// The whole thing again.
		assert_eq!(
			*args,
			[
				b"--prefix".to_vec(),
				b"hey".to_vec(),
				b"-k".to_vec(),
				b"Val".to_vec(),
				b"--empty".to_vec(),
				vec![],
				b"--key".to_vec(),
				b"Val".to_vec(),
			]
		);

		assert_eq!(args.get(0), Some(&b"--prefix"[..]));
		assert!(args.switch(b"--prefix"));
		assert_eq!(args.option(b"--prefix"), Some(&b"hey"[..]));

		// This is as good a place as any to double-check the _os version links
		// up correctly.
		let hey = OsStr::new("hey");
		assert_eq!(args.option_os(b"--prefix"), Some(hey));

		// Let's see what trailing args look like when there are none.
		assert_eq!(args.arg(0), None);

		// Let's also make sure the trailing arguments work too.
		let trailing: &[&[u8]] = &[b"Hello", b"World"];
		base.extend_from_slice(trailing);
		args = base.iter().copied().collect();
		assert_eq!(args.arg(0), Some(&b"Hello"[..]));
		assert_eq!(args.arg(1), Some(&b"World"[..]));
		assert_eq!(args.arg(2), None);
		assert_eq!(args.args(), trailing);

		// If there are no keys, the first entry should also be the first
		// argument.
		args = [b"hello".to_vec()].into_iter().collect();
		assert_eq!(args.arg(0), Some(&b"hello"[..]));

		// Unless we're expecting a subcommand...
		args.flags |= FLAG_SUBCOMMAND;
		assert!(args.arg(0).is_none());
	}

	#[test]
	fn t_version() {
		let mut base: Vec<&[u8]> = vec![
			b"hey",
			b"-V",
		];

		// We should be wanting a version.
		let mut args: Argue = base.iter().copied().collect();

		assert!(matches!(
			args.check_flags(FLAG_VERSION),
			Err(ArgyleError::WantsVersion)
		));

		// Same thing without the version flag.
		args = base.iter().copied().collect();
		assert!(args.check_flags(FLAG_HELP).is_ok());

		// Repeat with the long flag.
		base[1] = b"--version";

		// We should be wanting a version.
		args = base.iter().copied().collect();
		assert!(matches!(
			args.check_flags(FLAG_VERSION),
			Err(ArgyleError::WantsVersion)
		));

		// Same thing without the version flag.
		args = base.iter().copied().collect();
		assert!(args.check_flags(FLAG_HELP).is_ok());

		// One last time without a version arg present.
		base[1] = b"--ok";

		// We should be wanting a version.
		args = base.iter().copied().collect();
		assert!(args.check_flags(FLAG_VERSION).is_ok());

		// Same thing without the version flag.
		args = base.iter().copied().collect();
		assert!(args.check_flags(FLAG_HELP).is_ok());
	}

	#[test]
	fn t_help() {
		let mut base: Vec<&[u8]> = vec![
			b"hey",
			b"-h",
		];

		// We should be wanting a static help.
		let mut args: Argue = base.iter().copied().collect();
		assert!(matches!(
			args.check_flags(FLAG_HELP),
			Err(ArgyleError::WantsHelp)
		));

		#[cfg(feature = "dynamic-help")]
		{
			// Dynamic help this time.
			args = base.iter().copied().collect();
			match args.check_flags(FLAG_DYNAMIC_HELP) {
				Err(ArgyleError::WantsDynamicHelp(e)) => {
					let expected: Option<Box<[u8]>> = Some(Box::from(&b"hey"[..]));
					assert_eq!(e, expected);
				},
				_ => panic!("Test should have produced an error with Some(Box(hey))."),
			}
		}

		// Same thing without wanting help.
		args = base.iter().copied().collect();
		assert!(args.check_flags(FLAG_VERSION).is_ok());

		// Again with help flag first.
		base[0] = b"--help";

		// We should be wanting a static help.
		args = base.iter().copied().collect();
		assert!(matches!(
			args.check_flags(FLAG_HELP),
			Err(ArgyleError::WantsHelp)
		));

		#[cfg(feature = "dynamic-help")]
		{
			args = base.iter().copied().collect();
			// Dynamic help this time.
			assert!(matches!(
				args.check_flags(FLAG_DYNAMIC_HELP),
				Err(ArgyleError::WantsDynamicHelp(None))
			));
		}

		// Same thing without wanting help.
		args = base.iter().copied().collect();
		assert!(args.check_flags(FLAG_VERSION).is_ok());

		// Same thing without wanting help.
		base[0] = b"help";
		base[1] = b"--foo";

		// We should be wanting a static help.
		args = base.iter().copied().collect();
		assert!(matches!(
			args.check_flags(FLAG_HELP),
			Err(ArgyleError::WantsHelp)
		));

		#[cfg(feature = "dynamic-help")]
		{
			args = base.iter().copied().collect();
			// Dynamic help this time.
			assert!(matches!(
				args.check_flags(FLAG_DYNAMIC_HELP),
				Err(ArgyleError::WantsDynamicHelp(None))
			));
		}

		// Same thing without wanting help.
		args = base.iter().copied().collect();
		assert!(args.check_flags(FLAG_VERSION).is_ok());

		// One last time with no helpish things.
		base[0] = b"hey";

		// We should be wanting a static help.
		args = base.iter().copied().collect();
		assert!(args.check_flags(FLAG_HELP).is_ok());

		#[cfg(feature = "dynamic-help")]
		{
			// Dynamic help this time.
			args = base.iter().copied().collect();
			assert!(args.check_flags(FLAG_DYNAMIC_HELP).is_ok());
		}

		// Same thing without wanting help.
		args = base.iter().copied().collect();
		assert!(args.check_flags(FLAG_VERSION).is_ok());
	}

	#[test]
	fn t_with_list() {
		let list = std::path::Path::new("skel/list.txt");
		assert!(list.exists(), "Missing list.txt");

		let mut base: Vec<Vec<u8>> = vec![
			b"print".to_vec(),
			b"-l".to_vec(),
			b"skel/list.txt".to_vec(),
		];

		let mut args = base.iter().cloned().collect::<Argue>().with_list();
		assert_eq!(
			*args,
			[
				b"print".to_vec(),
				b"-l".to_vec(),
				b"skel/list.txt".to_vec(),
				b"/foo/bar/one".to_vec(),
				b"/foo/bar/two".to_vec(),
				b"/foo/bar/three".to_vec(),
			]
		);

		// These should be trailing args.
		assert_eq!(args.arg(0), Some(&b"/foo/bar/one"[..]));
		assert_eq!(args.arg(1), Some(&b"/foo/bar/two"[..]));
		assert_eq!(args.arg(2), Some(&b"/foo/bar/three"[..]));

		// Now try it with a bad file.
		base[2] = b"skel/not-list.txt".to_vec();
		args = base.iter().cloned().collect::<Argue>().with_list();
		assert_eq!(
			*args,
			[
				b"print".to_vec(),
				b"-l".to_vec(),
				b"skel/not-list.txt".to_vec(),
			]
		);
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

		let args: Argue = base.iter().copied().collect();
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
	fn t_by_prefix() {
		let base: Vec<&[u8]> = vec![
			b"hey",
			b"-k",
			b"--dump-three",
			b"--key-1=Val",
			b"--dump-four",
			b"--one-more",
		];

		let args = base.iter().cloned().collect::<Argue>();
		assert_eq!(args.switch_by_prefix(b"--dump"), Some(&b"-three"[..]));
		assert_eq!(args.switch_by_prefix(b"--dump-"), Some(&b"three"[..]));
		assert_eq!(args.switch_by_prefix(b"--with"), None);
		assert_eq!(args.switch_by_prefix(b"-k"), None); // Full matches suppressed.

		assert_eq!(
			args.option_by_prefix(b"--key-"),
			Some((&b"1"[..], &b"Val"[..]))
		);
		assert_eq!(args.option_by_prefix(b"--foo"), None);
		assert_eq!(args.option_by_prefix(b"--key-1"), None); // Full matches suppressed.
	}
}
