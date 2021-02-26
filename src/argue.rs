/*!
# Argyle: Argue
*/

use crate::{
	ArgyleError,
	KeyKind,
	utility,
};
use std::{
	borrow::Cow,
	cell::Cell,
	ops::Deref,
};



/// # Flag: Argument(s) Required.
///
/// If a program is called with zero arguments — no flags, options, trailing
/// args —, an error will be printed and the thread will exit with status code
/// `1`.
pub const FLAG_REQUIRED: u8 =     0b0000_0001;

/// # Flag: Merge Separator Args.
///
/// If the program is called with `--` followed by additional arguments, those
/// arguments are glued back together into a single entry, replacing that `--`.
pub const FLAG_SEPARATOR: u8 =    0b0000_0010;

/// # Flag: Expect Subcommand.
///
/// Set this flag to treat the first value as a subcommand rather than a
/// trailing argument. (This fixes the edge case where the command has zero
/// dash-prefixed keys.)
pub const FLAG_SUBCOMMAND: u8 =   0b0000_0100;

/// # Flag: Check For Help Flag.
///
/// When set, [`Argue`] will return [`ArgyleError::WantsDynamicHelp`] if help args
/// are present. The subcommand, if any, is included, allowing the caller to
/// dynamically handle output.
pub const FLAG_DYNAMIC_HELP: u8 = 0b0000_1000;

/// # Flag: Check For Help Flag.
///
/// When set, [`Argue`] will return [`ArgyleError::WantsHelp`] if help args are
/// present.
pub const FLAG_HELP: u8 =         0b0001_0000;

/// # Flag: Check For Version Flag.
///
/// When set, [`Argue`] will return [`ArgyleError::WantsVersion`] if version
/// args are present.
pub const FLAG_VERSION: u8 =      0b0010_0000;

/// # Flag: Has Help.
///
/// This flag is set if either `-h` or `--help` switches are present. It has
/// no effect unless [`Argue::FLAG_HELP`] is set.
const FLAG_HAS_HELP: u8 =         0b0100_0000;

/// # Flag: Has Version.
///
/// This flag is set if either `-V` or `--version` switches are present. It has
/// no effect unless [`Argue::FLAG_VERSION`] is set.
const FLAG_HAS_VERSION: u8 =      0b1000_0000;

/// # Flag: Do Version.
///
/// When both `FLAG_VERSION` and `FLAG_HAS_VERSION` are set.
const FLAG_DO_VERSION: u8 =       FLAG_VERSION | FLAG_HAS_VERSION;

/// # Flag: Any Help.
///
/// When either `FLAG_HELP` or `FLAG_DYNAMIC_HELP` are set.
const FLAG_ANY_HELP: u8 =         FLAG_HELP | FLAG_DYNAMIC_HELP;



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
/// Arguments are processed and held as bytes — `Cow<'static, [u8]>` — rather
/// than (os)strings, again leaving the choice of later conversion entirely up
/// to the implementor. For non-Musl Linux systems, this is almost entirely
/// non-allocating as CLI arguments map directly back to the `CStr` pointers.
/// For other systems, `Argue` falls back to [`std::env::args_os`], so requires
///  a bit more allocation.
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
///
/// ## Examples
///
/// `Argue` follows a builder pattern for construction, with a few odds and
/// ends tucked away as flags.
///
/// ```no_run
/// use std::borrow::Cow;
/// use argyle::{Argue, FLAG_REQUIRED};
///
/// // Parse the env arguments, aborting if the set is empty.
/// let args = Argue::new(FLAG_REQUIRED)?;
///
/// // Check to see what's there.
/// let switch: bool = args.switch(b"-s");
/// let option: Option<&[u8]> = args.option(b"--my-opt");
/// let extras: &[Cow<'static, [u8]>] = args.args();
/// ```
///
/// If you just want a clean set to iterate over, `Argue` can be dereferenced
/// to a slice:
///
/// ```ignore
/// let arg_slice: &[Cow<'static, [u8]>] = *args;
/// ```
///
/// Or it can be converted into an owned Vector:
/// ```ignore
/// let args: Vec<Cow<'static, [u8]>> = args.take();
/// ```
pub struct Argue {
	/// Parsed arguments.
	args: Vec<Cow<'static, [u8]>>,
	/// Keys.
	///
	/// This array holds the key indexes (from `self.args`) so checks can avoid
	/// re-evaluation, etc.
	///
	/// The last slot holds the number of keys, hence only 15 total keys are
	/// supported.
	keys: [usize; KEY_SIZE],
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

impl Default for Argue {
	#[inline]
	fn default() -> Self {
		Self {
			args: Vec::new(),
			keys: [0_usize; KEY_SIZE],
			last: Cell::new(0),
			flags: 0,
		}
	}
}

impl Deref for Argue {
	type Target = [Cow<'static, [u8]>];
	#[inline]
	fn deref(&self) -> &Self::Target { &self.args }
}

/// ## Instantiation and Builder Patterns.
impl Argue {
	#[cfg(all(target_os = "linux", not(target_env = "musl")))]
	#[inline]
	/// # New Instance.
	///
	/// This populates arguments from the environment using a specialized
	/// implementation that requires slightly less overhead than using the
	/// stock [`std::env::args`] iterator. The first (command path) part is
	/// automatically excluded.
	///
	/// To construct an `Argue` from arbitrary raw values, use the
	/// `Argue::from_iter()` method (via the [`std::iter::FromIterator`] trait).
	///
	/// ## Errors
	///
	/// This method will bubble any processing errors or aborts (like the
	/// discover of version or help flags).
	///
	/// ## Examples
	///
	/// ```no_run
	/// use argyle::Argue;
	///
	/// let args = Argue::new(0);
	/// ```
	pub fn new(flags: u8) -> Result<Self, ArgyleError> {
		let iter = argv::Args::default();
		let len = iter.len();
		iter
			.skip_while(|b| b.is_empty() || b.iter().all(u8::is_ascii_whitespace))
			.try_fold(
				Self {
					args: Vec::with_capacity(len),
					..Self::default()
				},
				Self::push
			)?
			.with_flags(flags)
	}

	#[cfg(any(not(target_os = "linux"), target_env = "musl"))]
	#[inline]
	/// # New Instance.
	///
	/// Populate arguments from `std::env::args()`. The first (command path)
	/// part is automatically excluded.
	///
	/// To construct an `Argue` from arbitrary raw values, use the
	/// `Argue::from_iter()` method (via the [`std::iter::FromIterator`] trait).
	///
	/// ## Errors
	///
	/// This method will bubble any processing errors or aborts (like the
	/// discover of version or help flags).
	///
	/// ## Examples
	///
	/// ```no_run
	/// use argyle::Argue;
	///
	/// let args = Argue::new(0);
	/// ```
	pub fn new(flags: u8) -> Result<Self, ArgyleError> {
		use std::os::unix::ffi::OsStrExt;

		std::env::args_os()
			.skip(1)
			.map(|b| b.as_bytes().to_vec())
			.skip_while(|x|
				x.is_empty() ||
				x.iter().all(u8::is_ascii_whitespace)
			)
			.try_fold(Self::default(), Self::push)?
			.with_flags(flags)
	}

	/// # With Flags.
	///
	/// This method can be used to set additional parsing options in cases
	/// where the struct was initialized without calling [`Argue::new`].
	///
	/// This will only ever enable flags; it will not disable existing flags.
	///
	/// ## Errors
	///
	/// This method will bubble any processing errors or aborts (like the
	/// discover of version or help flags).
	///
	/// ## Examples
	///
	/// ```no_run
	/// use argyle::{Argue, FLAG_REQUIRED};
	///
	/// let args = Argue::new(0).with_flags(FLAG_REQUIRED);
	/// ```
	pub fn with_flags(mut self, flags: u8) -> Result<Self, ArgyleError> {
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
		else if 0 != self.flags & FLAG_ANY_HELP {
			let cmd = self.args[0].as_ref();
			if 0 != self.flags & FLAG_HAS_HELP || cmd == b"help" {
				// Static help.
				if 0 != self.flags & FLAG_HELP {
					return Err(ArgyleError::WantsHelp);
				}

				// Dynamic help.
				return Err(ArgyleError::WantsDynamicHelp(
					if ! cmd.is_empty() && cmd[0] != b'-' && cmd != b"help" {
						Some(Box::from(cmd))
					}
					else { None }
				));
			}
		}

		// Handle separator.
		if 0 != self.flags & FLAG_SEPARATOR {
			self.parse_separator();
		}

		Ok(self)
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
	/// let mut args = Argue::new(0).with_list();
	/// ```
	pub fn with_list(mut self) -> Self {
		use std::{
			ffi::OsStr,
			fs::File,
			io::{
				BufRead,
				BufReader,
			},
			os::unix::ffi::OsStrExt,
		};

		if let Some(file) = self.option2(b"-l", b"--list").and_then(|p| File::open(OsStr::from_bytes(p)).ok()) {
			BufReader::new(file).lines()
				.filter_map(std::result::Result::ok)
				.for_each(|line| {
					let bytes = line.trim().as_bytes();
					if ! bytes.is_empty() {
						self.args.push(Cow::Owned(bytes.to_vec()))
					}
				});
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
	/// as a `Vec<Cow<[u8]>>`.
	///
	/// If you merely want something to iterate over, you can alternatively
	/// dereference the struct to a string slice.
	///
	/// ## Examples
	///
	/// ```no_run
	/// use argyle::Argue;
	///
	/// let args: Vec<Cow<[u8]>> = Argue::new(0).take();
	/// ```
	pub fn take(self) -> Vec<Cow<'static, [u8]>> { self.args }
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
	pub fn peek(&self) -> Option<&[u8]> { self.args.get(0).map(Cow::as_ref) }

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
	/// let args = Argue::new(FLAG_REQUIRED);
	///
	/// // This is actually safe because FLAG_REQUIRED would have errored out
	/// // if nothing were present.
	/// let first: &[u8] = unsafe { args.peek_unchecked() };
	/// ```
	pub unsafe fn peek_unchecked(&self) -> &[u8] { self.args[0].as_ref() }

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
	/// let mut args = Argue::new(0);
	/// let switch: bool = args.switch("--my-switch");
	/// ```
	pub fn switch(&self, key: &[u8]) -> bool {
		self.keys.iter()
			.take(self.keys[KEY_LEN])
			.map(|x| &self.args[*x])
			.any(|x| x.as_ref().eq(key))
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
	/// let mut args = Argue::new(0);
	/// let switch: bool = args.switch2("-s", "--my-switch");
	/// ```
	pub fn switch2(&self, short: &[u8], long: &[u8]) -> bool {
		self.keys.iter()
			.take(self.keys[KEY_LEN])
			.map(|x| &self.args[*x])
			.any(|x| {
				let xr = x.as_ref();
				xr.eq(short) || xr.eq(long)
			})
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
	/// let mut args = Argue::new(0);
	/// let opt: Option<&[u8]> = args.option("--my-opt");
	/// ```
	pub fn option(&self, key: &[u8]) -> Option<&[u8]> {
		self.keys.iter()
			.take(self.keys[KEY_LEN])
			.position(|&x| self.args.get(x).map_or(false, |x| x.as_ref().eq(key)))
			.and_then(|idx| {
				let idx = self.keys[idx] + 1;
				self.args.get(idx).map(|x| {
					if idx > self.last.get() { self.last.set(idx); }
					x.as_ref()
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
	/// let mut args = Argue::new(0);
	/// let opt: Option<&[u8]> = args.option2("-o", "--my-opt");
	/// ```
	pub fn option2(&self, short: &[u8], long: &[u8]) -> Option<&[u8]> {
		self.keys.iter()
			.take(self.keys[KEY_LEN])
			.position(|&x| self.args.get(x).map_or(false, |x| {
				let xr = x.as_ref();
				xr.eq(short) || xr.eq(long)
			}))
			.and_then(|idx| {
				let idx = self.keys[idx] + 1;
				self.args.get(idx).map(|x| {
					if idx > self.last.get() { self.last.set(idx); }
					x.as_ref()
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
	/// let mut args = Argue::new(0);
	/// let extras: &[Cow<[u8]>] = args.args();
	/// ```
	pub fn args(&self) -> &[Cow<'static, [u8]>] {
		let idx = self.arg_idx();
		if idx < self.args.len() {
			&self.args[self.arg_idx()..]
		}
		else { &[] }
	}

	#[must_use]
	/// # Arg at Index
	///
	/// Pluck the nth trailing argument by index (starting from zero).
	///
	/// Note, this is different than dereferencing the whole `Argue` struct
	/// and requesting its zero index; that would refer to the first CLI
	/// argument of any kind, which could be a subcommand or key.
	pub fn arg(&self, idx: usize) -> Option<&[u8]> {
		let start_idx = self.arg_idx();
		if start_idx + idx < self.args.len() {
			Some(self.args[start_idx + idx].as_ref())
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
	/// let mut args = Argue::new(0);
	/// let opt: &[u8] = args.first_arg();
	/// ```
	pub fn first_arg(&self) -> Result<&[u8], ArgyleError> {
		let idx = self.arg_idx();
		if idx >= self.args.len() {
			Err(ArgyleError::NoArg)
		}
		else {
			Ok(self.args[idx].as_ref())
		}
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
		if self.keys[KEY_LEN] == 0 && 0 == self.flags & FLAG_SUBCOMMAND { 0 }
		else { self.last.get() + 1 }
	}

	/// # Insert Key.
	///
	/// This will record the key index, unless the maximum number of keys
	/// has been reached, in which case it will print an error and exit with a
	/// status code of `1` instead.
	fn insert_key(&mut self, idx: usize) -> Result<(), ArgyleError> {
		if self.keys[KEY_LEN] == KEY_LEN {
			return Err(ArgyleError::TooManyKeys);
		}

		self.keys[self.keys[KEY_LEN]] = idx;
		self.keys[KEY_LEN] += 1;

		Ok(())
	}

	#[cfg(all(target_os = "linux", not(target_env = "musl")))]
	/// # Parse Keys (Bytes).
	fn push(mut self, bytes: &'static [u8]) -> Result<Self, ArgyleError> {
		// Find out what we've got!
		match KeyKind::from(bytes) {
			// Passthrough.
			KeyKind::None => {
				self.args.push(Cow::Borrowed(bytes));
			},
			// Record the key and passthrough.
			KeyKind::Short => {
				if bytes[1] == b'V' { self.flags |= FLAG_HAS_VERSION; }
				else if bytes[1] == b'h' { self.flags |= FLAG_HAS_HELP; }

				let idx = self.args.len();
				self.args.push(Cow::Borrowed(bytes));
				self.insert_key(idx)?;
				self.last.set(idx);
			},
			// Record the key and passthrough.
			KeyKind::Long => {
				if bytes == b"--version" { self.flags |= FLAG_HAS_VERSION; }
				else if bytes == b"--help" { self.flags |= FLAG_HAS_HELP; }

				let idx = self.args.len();
				self.args.push(Cow::Borrowed(bytes));
				self.insert_key(idx)?;
				self.last.set(idx);
			},
			// Split a short key/value pair.
			KeyKind::ShortV => {
				let idx = self.args.len();

				self.args.push(Cow::Borrowed(&bytes[0..2]));
				self.args.push(Cow::Borrowed(&bytes[2..]));

				self.insert_key(idx)?;
				self.last.set(idx + 1);
			},
			// Split a long key/value pair.
			KeyKind::LongV(x) => {
				let idx = self.args.len();

				self.args.push(Cow::Borrowed(&bytes[0..x]));

				if x + 1 < bytes.len() {
					self.args.push(Cow::Borrowed(&bytes[x + 1..]));
				}
				else {
					self.args.push(Cow::Owned(Vec::new()));
				}

				self.insert_key(idx)?;
				self.last.set(idx + 1);
			},
		}

		Ok(self)
	}

	#[cfg(any(not(target_os = "linux"), target_env = "musl"))]
	/// # Parse Keys (Owned Bytes).
	fn push(mut self, mut bytes: Vec<u8>) -> Result<Self, ArgyleError> {
		// Find out what we've got!
		match KeyKind::from(&bytes[..]) {
			// Passthrough.
			KeyKind::None => {
				self.args.push(Cow::Owned(bytes));
			},
			// Record the key and passthrough.
			KeyKind::Short => {
				if bytes[1] == b'V' { self.flags |= FLAG_HAS_VERSION; }
				else if bytes[1] == b'h' { self.flags |= FLAG_HAS_HELP; }

				let idx = self.args.len();
				self.args.push(Cow::Owned(bytes));
				self.insert_key(idx)?;
				self.last.set(idx);
			},
			// Record the key and passthrough.
			KeyKind::Long => {
				if bytes == b"--version" { self.flags |= FLAG_HAS_VERSION; }
				else if bytes == b"--help" { self.flags |= FLAG_HAS_HELP; }

				let idx = self.args.len();
				self.args.push(Cow::Owned(bytes));
				self.insert_key(idx)?;
				self.last.set(idx);
			},
			// Split a short key/value pair.
			KeyKind::ShortV => {
				let idx = self.args.len();

				let v2 = bytes.split_off(2);
				self.args.push(Cow::Owned(bytes));
				self.args.push(Cow::Owned(v2));

				self.insert_key(idx)?;
				self.last.set(idx + 1);
			},
			// Split a long key/value pair.
			KeyKind::LongV(x) => {
				let idx = self.args.len();


				if x + 1 < bytes.len() {
					let v2 = bytes.split_off(x + 1);
					bytes.truncate(x);
					self.args.push(Cow::Owned(bytes));
					self.args.push(Cow::Owned(v2));
				}
				else {
					bytes.truncate(x);
					self.args.push(Cow::Owned(bytes));
					self.args.push(Cow::Owned(Vec::new()));
				}

				self.insert_key(idx)?;
				self.last.set(idx + 1);
			},
		}

		Ok(self)
	}

	/// # Parse Separator.
	///
	/// This concatenates all arguments trailing a "--" entry into a single
	/// value, replacing the "--".
	///
	/// It is not recursive; if a separator has its own separator, it will
	/// merely be included in the re-glued string.
	fn parse_separator(&mut self) {
		if let Some(idx) = self.args.iter().position(|x| x.as_ref().eq(b"--")) {
			if idx + 1 < self.args.len() {
				let mut joined: Vec<u8> = self.args.drain(idx + 1..)
					.flat_map(|x| {
						let mut v: Vec<u8> = x.into_owned();
						utility::esc_arg_b(&mut v);
						v.push(b' ');
						v
					})
					.collect::<Vec<u8>>();
				if let Some(b' ') = joined.last() {
					joined.truncate(joined.len() - 1);
				}
				self.args[idx] = Cow::Owned(joined);
			}
			else {
				self.args.truncate(idx);
			}
		}
	}
}



#[cfg(all(target_os = "linux", not(target_env = "musl")))]
#[allow(clippy::similar_names)] // Follow convention.
/// # Linux Specialized Args
///
/// This is a non-allocating version of [`std::env::args_os`] for non-Musl Linux
/// systems inspired by [`argv`](https://crates.io/crates/argv).
///
/// Other targets just use the normal [`std::env::args_os`].
mod argv {
	use std::{
		ffi::CStr,
		os::raw::{
			c_char,
			c_int,
		},
	};

	static mut ARGC: c_int = 0;
	static mut ARGV: *const *const c_char = std::ptr::null();

	#[cfg(target_os = "linux")]
	#[link_section = ".init_array"]
	#[used]
	static CAPTURE: unsafe extern "C" fn(c_int, *const *const c_char) = capture;

	#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
	#[allow(dead_code)]
	unsafe extern "C" fn capture(argc: c_int, argv: *const *const c_char) {
		ARGC = argc;
		ARGV = argv;
	}

	/// # Raw Arguments.
	///
	/// This will skip the first (path) argument and return a pointer and
	/// length if there's anything worth returning.
	///
	/// The actual iterables are byte slices in this case, rather than
	/// (os)strings.
	pub(super) struct Args {
		next: *const *const c_char,
		end: *const *const c_char,
	}

	impl Default for Args {
		#[allow(clippy::cast_sign_loss)] // ARGC is non-negative.
		/// # Raw Arguments.
		///
		/// ## Safety
		///
		/// This accesses mutable statics — `ARGC` and `ARGV` — but because
		/// they are only mutated prior to the execution of `main()`, it's
		/// A-OK.
		///
		/// Also worth noting, the operating system is responsible for ensuring
		/// `ARGV + ARGC` does not overflow, so no worries there either.
		fn default() -> Self {
			// We'll only return arguments if there are at least 2 of them.
			let len: usize = unsafe { ARGC } as usize;
			if len > 1 {
				Self {
					next: unsafe { ARGV.add(1) },
					end: unsafe { ARGV.add(len) },
				}
			}
			else {
				let end = unsafe { ARGV.add(len) };
				Self {
					next: end,
					end
				}
			}
		}
	}

	impl Iterator for Args {
		type Item = &'static [u8];

		fn next(&mut self) -> Option<Self::Item> {
			if self.next >= self.end { None }
			else {
				let out = unsafe { CStr::from_ptr(*self.next).to_bytes() };
				self.next = unsafe { self.next.add(1) };
				Some(out)
			}
		}

		fn size_hint(&self) -> (usize, Option<usize>) {
			let len = self.len();
			(len, Some(len))
		}
	}

	impl ExactSizeIterator for Args {
		#[allow(clippy::cast_sign_loss)] // Distance is always >= 0.
		fn len(&self) -> usize {
			unsafe { self.end.offset_from(self.next) as usize }
		}
	}
}



#[cfg(all(target_os = "linux", not(target_env = "musl")))]
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
			b"--",
			b"stuff",
			b"and things",
		];

		let mut args = base.iter()
			.try_fold(Argue::default(), |a, &b| a.push(b))
			.expect("Failed to build Argue.")
			.with_flags(FLAG_SEPARATOR)
			.expect("Failed to build Argue.");

		// Check the overall structure.
		assert_eq!(
			*args,
			[
				Cow::from(&b"hey"[..]),
				Cow::from(&b"-k"[..]),
				Cow::from(&b"Val"[..]),
				Cow::from(&b"--empty"[..]),
				Cow::from(vec![]),
				Cow::from(&b"--key"[..]),
				Cow::from(&b"Val"[..]),
				Cow::from(&b"stuff 'and things'"[..]),
			]
		);

		// Test the finders.
		assert_eq!(args.peek(), Some(&b"hey"[..]));
		assert!(args.switch(b"-k"));
		assert!(args.switch(b"--key"));
		assert!(args.switch2(b"-k", b"--key"));
		assert_eq!(args.option(b"--key"), Some(&b"Val"[..]));
		assert_eq!(args.option2(b"-k", b"--key"), Some(&b"Val"[..]));

		{
			let a = args.args();
			assert_eq!(a.len(), 1);
			assert_eq!(a[0].as_ref(), b"stuff 'and things'");
		}

		// Let's test a first-position key, and also not doing separator bits.
		base.insert(0, b"--prefix");
		args = base.iter()
			.try_fold(Argue::default(), |a, &b| a.push(b))
			.expect("Failed to build Argue.");

		// The whole thing again.
		assert_eq!(
			*args,
			[
				Cow::from(&b"--prefix"[..]),
				Cow::from(&b"hey"[..]),
				Cow::from(&b"-k"[..]),
				Cow::from(&b"Val"[..]),
				Cow::from(&b"--empty"[..]),
				Cow::from(vec![]),
				Cow::from(&b"--key"[..]),
				Cow::from(&b"Val"[..]),
				Cow::from(&b"--"[..]),
				Cow::from(&b"stuff"[..]),
				Cow::from(&b"and things"[..]),
			]
		);

		assert_eq!(args.peek(), Some(&b"--prefix"[..]));
		assert!(args.switch(b"--prefix"));
		assert_eq!(args.option(b"--prefix"), Some(&b"hey"[..]));

		// Something that doesn't exist.
		assert_eq!(args.option(b"foo"), None);
	}

	#[test]
	fn t_version() {
		let mut base: Vec<&[u8]> = vec![
			b"hey",
			b"-V",
		];

		// We should be wanting a version.
		assert!(matches!(
			base.iter()
				.try_fold(Argue::default(), |a, &b| a.push(b))
				.expect("Failed to build Argue.")
				.with_flags(FLAG_VERSION),
			Err(ArgyleError::WantsVersion)
		));

		// Same thing without the version flag.
		assert!(
			base.iter()
				.try_fold(Argue::default(), |a, &b| a.push(b))
				.expect("Failed to build Argue.")
				.with_flags(FLAG_HELP)
				.is_ok()
		);

		// Repeat with the long flag.
		base[1] = b"--version";

		// We should be wanting a version.
		assert!(matches!(
			base.iter()
				.try_fold(Argue::default(), |a, &b| a.push(b))
				.expect("Failed to build Argue.")
				.with_flags(FLAG_VERSION),
			Err(ArgyleError::WantsVersion)
		));

		// Same thing without the version flag.
		assert!(
			base.iter()
				.try_fold(Argue::default(), |a, &b| a.push(b))
				.expect("Failed to build Argue.")
				.with_flags(FLAG_HELP)
				.is_ok()
		);

		// One last time without a version arg present.
		base[1] = b"--ok";

		// We should be wanting a version.
		assert!(
			base.iter()
				.try_fold(Argue::default(), |a, &b| a.push(b))
				.expect("Failed to build Argue.")
				.with_flags(FLAG_VERSION)
				.is_ok()
		);

		// Same thing without the version flag.
		assert!(
			base.iter()
				.try_fold(Argue::default(), |a, &b| a.push(b))
				.expect("Failed to build Argue.")
				.with_flags(FLAG_HELP)
				.is_ok()
		);
	}

	#[test]
	fn t_help() {
		let mut base: Vec<&[u8]> = vec![
			b"hey",
			b"-h",
		];

		// We should be wanting a static help.
		assert!(matches!(
			base.iter()
				.try_fold(Argue::default(), |a, &b| a.push(b))
				.expect("Failed to build Argue.")
				.with_flags(FLAG_HELP),
			Err(ArgyleError::WantsHelp)
		));

		// Dynamic help this time.
		if let Err(ArgyleError::WantsDynamicHelp(e)) = base.iter()
				.try_fold(Argue::default(), |a, &b| a.push(b))
				.expect("Failed to build Argue.")
				.with_flags(FLAG_DYNAMIC_HELP) {
			let expected: Option<Box<[u8]>> = Some(Box::from(&b"hey"[..]));
			assert_eq!(e, expected);
		}
		else {
			panic!("Test should have produced an error with Some(Box(hey)).");
		}

		// Same thing without wanting help.
		assert!(
			base.iter()
				.try_fold(Argue::default(), |a, &b| a.push(b))
				.expect("Failed to build Argue.")
				.with_flags(FLAG_VERSION)
				.is_ok()
		);

		// Again with help flag first.
		base[0] = b"--help";

		// We should be wanting a static help.
		assert!(matches!(
			base.iter()
				.try_fold(Argue::default(), |a, &b| a.push(b))
				.expect("Failed to build Argue.")
				.with_flags(FLAG_HELP),
			Err(ArgyleError::WantsHelp)
		));

		// Dynamic help this time.
		assert!(matches!(
			base.iter()
				.try_fold(Argue::default(), |a, &b| a.push(b))
				.expect("Failed to build Argue.")
				.with_flags(FLAG_DYNAMIC_HELP),
			Err(ArgyleError::WantsDynamicHelp(None))
		));

		// Same thing without wanting help.
		assert!(
			base.iter()
				.try_fold(Argue::default(), |a, &b| a.push(b))
				.expect("Failed to build Argue.")
				.with_flags(FLAG_VERSION)
				.is_ok()
		);

		// Same thing without wanting help.
		base[0] = b"help";
		base[1] = b"--foo";

		// We should be wanting a static help.
		assert!(matches!(
			base.iter()
				.try_fold(Argue::default(), |a, &b| a.push(b))
				.expect("Failed to build Argue.")
				.with_flags(FLAG_HELP),
			Err(ArgyleError::WantsHelp)
		));

		// Dynamic help this time.
		assert!(matches!(
			base.iter()
				.try_fold(Argue::default(), |a, &b| a.push(b))
				.expect("Failed to build Argue.")
				.with_flags(FLAG_DYNAMIC_HELP),
			Err(ArgyleError::WantsDynamicHelp(None))
		));

		// Same thing without wanting help.
		assert!(
			base.iter()
				.try_fold(Argue::default(), |a, &b| a.push(b))
				.expect("Failed to build Argue.")
				.with_flags(FLAG_VERSION)
				.is_ok()
		);

		// One last time with no helpish things.
		base[0] = b"hey";

		// We should be wanting a static help.
		assert!(
			base.iter()
				.try_fold(Argue::default(), |a, &b| a.push(b))
				.expect("Failed to build Argue.")
				.with_flags(FLAG_HELP)
				.is_ok()
		);

		// Dynamic help this time.
		assert!(
			base.iter()
				.try_fold(Argue::default(), |a, &b| a.push(b))
				.expect("Failed to build Argue.")
				.with_flags(FLAG_DYNAMIC_HELP)
				.is_ok()
		);

		// Same thing without wanting help.
		assert!(
			base.iter()
				.try_fold(Argue::default(), |a, &b| a.push(b))
				.expect("Failed to build Argue.")
				.with_flags(FLAG_VERSION)
				.is_ok()
		);
	}
}
