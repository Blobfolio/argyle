/*!
# Argyle: Streaming Argument Iterator.

This module contains a streaming alternative to the crate's original (and
deprecated) [`Argue`](crate::Argue) structure that avoids the overhead associated
with argument collection and searching.

This [`Argue`] is simpler and cleaner than the original, but less agnostic as
it requires declaring reserved keywords — subcommands, switches, and options —
upfront (via builder-style methods) to remove the guesswork during iteration.
*/

mod error;

pub use error::ArgyleError;
use std::{
	borrow::Borrow,
	cmp::Ordering,
	collections::BTreeSet,
	env::ArgsOs,
	ffi::{
		OsStr,
		OsString,
	},
	iter::Skip,
};



/// # Alias for Env Args.
///
/// This is the return type for [`args`]. It is kinda clunky so downstream
/// users may want to just use this shorthand instead.
pub type ArgueEnv = Argue<Skip<ArgsOs>>;



/// # Streaming Argument Iterator.
///
/// `Argue` occupies the middle ground between [`std::env::args`] and full-service
/// crates like `clap`.
///
/// It performs some basic normalization — and won't crash if an [`OsString`]
/// contains invalid UTF-8! — and can help identify any reserved subcommands
/// and keys expected by your app, but leaves any subsequent validation- and
/// handling-related particulars _to you_.
///
/// That said, it does have a few _opinions_ that are worth noting:
/// * Keywords may only contain ASCII alphanumeric characters, `-`, and `_`;
/// * Subcommands must start with an ASCII alphanumeric character;
/// * Short keys can only be two bytes: a dash followed by an ASCII alphanumeric character;
/// * Long keys can be any length provided they start with two dashes and an ASCII alphanumeric character;
///
/// `Argue` supports both combined and consecutive key/value association. For
/// example, the following are equivalent:
/// * `-kval`; `-k=val`; `-k` then `val`;
/// * `--key=val`; `--key` then `val`;
///
/// Arguments following an end-of-command separator (`--`) are not parsed, but
/// instead collected and returned as-are in case you want to do anything with
/// them. See [`Argument::End`] for more details.
///
/// ## Examples
///
/// ```
/// use argyle::stream::Argument;
///
/// // Most of the time you'll probably want to parse env args, which the
/// // helper method `args` sets up. Add switches, options, etc, then loop to
/// // see what all comes in!
/// for arg in argyle::stream::args().with_key("--help", false).unwrap() {
///     match arg {
///         // The user wants help.
///         Argument::Key("--help") => println!("Help! Help!"),
///
///         // The user passed something else.
///         Argument::Other(s) => println!("Found: {s}"),
///
///         // The user passed something that can't be stringified, but
///         // maybe you can make use of the raw value anyway?
///         Argument::InvalidUtf8(s) => println!("WTF: {s:?}"),
///
///         // Other apps might have keys with values (options),
///         // subcommands, end-of-command arguments, etc.
///         _ => todo!(),
///     }
/// }
/// ```
pub struct Argue<I> {
	/// # Raw Iterator.
	iter: I,

	/// # Keys to Look For.
	special: BTreeSet<KeyKind>,
}

impl<I: IntoIterator<Item=OsString>> From<I> for Argue<I::IntoIter> {
	fn from(src: I) -> Self {
		Self {
			iter: src.into_iter(),
			special: BTreeSet::new(),
		}
	}
}

impl<I> Argue<I> {
	/// # With (Sub)Command.
	///
	/// Add a (sub)command to the watchlist.
	///
	/// ## Examples
	///
	/// ```
	/// let args = argyle::stream::args()
	///     .with_command("make").unwrap();
	///
	/// for arg in args {
	///     // Do stuff!
	/// }
	/// ```
	///
	/// ## Errors
	///
	/// This will return an error if the command was previously specified (as
	/// any type of key) or contains invalid characters.
	pub fn with_command(mut self, key: &'static str) -> Result<Self, ArgyleError> {
		let key = key.trim();

		// Ignore empties.
		if key.is_empty() { Ok(self) }
		// Call out invalid characters specially.
		else if ! valid_command(key.as_bytes()) { Err(ArgyleError::InvalidKey(key)) }
		// Add it if unique!
		else if self.special.insert(KeyKind::Command(key)) { Ok(self) }
		// It wasn't unique…
		else { Err(ArgyleError::DuplicateKey(key)) }
	}

	/// # With Key.
	///
	/// Add a key to the watchlist, optionally requiring a value.
	///
	/// ## Examples
	///
	/// ```
	/// let args = argyle::stream::args()
	///     .with_key("--verbose", false).unwrap() // Boolean flag.
	///     .with_key("--output", true).unwrap();  // Expects a value.
	///
	/// for arg in args {
	///     // Do stuff!
	/// }
	/// ```
	///
	/// ## Errors
	///
	/// This will return an error if the key was previously specified
	/// or contains invalid characters.
	pub fn with_key(mut self, key: &'static str, value: bool)
	-> Result<Self, ArgyleError> {
		let key = key.trim();

		// Ignore empties.
		if key.is_empty() { Ok(self) }
		// Call out invalid characters specially.
		else if ! valid_key(key.as_bytes()) { Err(ArgyleError::InvalidKey(key)) }
		// Add it if unique!
		else {
			let k = if value { KeyKind::KeyWithValue(key) } else { KeyKind::Key(key) };
			if self.special.insert(k) { Ok(self) }
			else { Err(ArgyleError::DuplicateKey(key)) }
		}
	}
}

impl<I> Argue<I> {
	/// # With (Sub)Commands.
	///
	/// Add one or more (sub)commands to the watchlist.
	///
	/// ## Examples
	///
	/// ```
	/// let args = argyle::stream::args()
	///     .with_commands(["help", "verify"]).unwrap();
	///
	/// for arg in args {
	///     // Do stuff!
	/// }
	/// ```
	///
	/// ## Errors
	///
	/// This will return an error if any of the commands were previously
	/// specified (as any type of key) or contain invalid characters.
	pub fn with_commands<I2: IntoIterator<Item=&'static str>>(self, keys: I2)
	-> Result<Self, ArgyleError> {
		keys.into_iter().try_fold(self, Self::with_command)
	}

	/// # With Keys.
	///
	/// Add one or more keys to the watchlist.
	///
	/// If you find the tuples messy, you can use [`Argue::with_switches`] and
	/// [`Argue::with_options`] to declare your keys instead.
	///
	/// ## Examples
	///
	/// ```
	/// let args = argyle::stream::args()
	///     .with_keys([
	///         ("--verbose", false), // Boolean flag.
	///         ("--output", true),   // Expects a value.
	///      ]).unwrap();
	///
	/// for arg in args {
	///     // Do stuff!
	/// }
	/// ```
	///
	/// ## Errors
	///
	/// This will return an error if any of the keys were previously specified
	/// or contain invalid characters.
	pub fn with_keys<I2: IntoIterator<Item=(&'static str, bool)>>(self, keys: I2)
	-> Result<Self, ArgyleError> {
		keys.into_iter().try_fold(self, |acc, (k, v)| acc.with_key(k, v))
	}

	/// # With Options.
	///
	/// Add one or more keys to the watchlist that require values.
	///
	/// ## Examples
	///
	/// ```
	/// let args = argyle::stream::args()
	///     .with_options(["--input", "--output"])
	///     .unwrap();
	///
	/// for arg in args {
	///     // Do stuff!
	/// }
	/// ```
	///
	/// ## Errors
	///
	/// This will return an error if any of the keys were previously specified
	/// or contain invalid characters.
	pub fn with_options<I2: IntoIterator<Item=&'static str>>(self, keys: I2)
	-> Result<Self, ArgyleError> {
		keys.into_iter().try_fold(self, |acc, k| acc.with_key(k, true))
	}

	/// # With Switches.
	///
	/// Add one or more boolean keys to the watchlist.
	///
	/// This can be used instead of [`Argue::with_keys`] if you have a bunch
	/// of keys that do not require values. (It saves you the trouble of
	/// tuple-izing everything.)
	///
	/// ## Examples
	///
	/// ```
	/// let args = argyle::stream::args()
	///     .with_switches(["--verbose", "--strict"])
	///     .unwrap();
	///
	/// for arg in args {
	///     // Do stuff!
	/// }
	/// ```
	///
	/// ## Errors
	///
	/// This will return an error if any of the keys were previously specified
	/// or contain invalid characters.
	pub fn with_switches<I2: IntoIterator<Item=&'static str>>(self, keys: I2)
	-> Result<Self, ArgyleError> {
		keys.into_iter().try_fold(self, |acc, k| acc.with_key(k, false))
	}
}

impl<I> Argue<I> {
	/// # Find Key.
	///
	/// Find and return the key associated with `raw`, if any.
	fn find_keyword(&self, raw: &str) -> Option<KeyKind> {
		// Short circuit; keywords must start with a dash or alphanumeric.
		let bytes = raw.as_bytes();
		if bytes.is_empty() || ! (bytes[0] == b'-' || bytes[0].is_ascii_alphanumeric()) {
			return None;
		}

		// Direct hit!
		if let Some(key) = self.special.get(raw) { return Some(*key); }

		// Keylike strings could have a value gumming up the works; separate
		// and try again if that is the case.
		if 3 <= bytes.len() && bytes[0] == b'-' {
			let needle: &str =
				// Short keys can only be two bytes.
				if bytes[1].is_ascii_alphanumeric() { raw.get(..2) }
				// Long keys can only have values if there's an = sign
				// in there somewhere.
				else if bytes[1] == b'-' && bytes[2].is_ascii_alphanumeric() {
					raw.split_once('=').map(|(k, _)| k)
				}
				// No dice.
				else { None }?;
			self.special.get(needle).copied()
		}
		else { None }
	}
}

impl<I: Iterator<Item=OsString>> Iterator for Argue<I> {
	type Item = Argument;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			// Pull the next value and try to stringify it.
			let mut next = match self.iter.next()?.into_string() {
				Ok(next) => next,
				// We can't do anything with OsString; return as is.
				Err(e) => return Some(Argument::InvalidUtf8(e)),
			};

			// Empty values that aren't associated with a key are pointless.
			if next.is_empty() { continue; }

			// If we've hit a separator, just gobble up the remaining bits and
			// return them without further effort.
			if next == "--" {
				let next = self.iter.by_ref().collect::<Vec<_>>();
				if next.is_empty() { return None; }
				return Some(Argument::End(next));
			}

			// Is this a key?
			if let Some(key) = self.find_keyword(&next) {
				// Tease out the matched key.
				let k = key.as_str();

				// Return whatever we're meant to based on the match type.
				return Some(match key {
					KeyKind::Command(_) => Argument::Command(k),
					KeyKind::Key(_) => Argument::Key(k),
					KeyKind::KeyWithValue(_) => {
						// We need a value for this one!
						let v: String =
							// Pull it from the next argument.
							if next == k {
								match self.iter.next()?.into_string() {
									Ok(v) => v,
									// This is awkward! Let's merge the key and
									// value into a single OsString that can be
									// returned instead.
									Err(e) => {
										let mut boo = OsStr::new(k).to_owned();
										boo.push("=");
										boo.push(e);
										return Some(Argument::InvalidUtf8(boo));
									},
								}
							}
							// Split it off from the current argument.
							else {
								let mut v = next.split_off(k.len());
								if v.starts_with('=') { v.drain(..1); }
								v
							};

						Argument::KeyWithValue(k, v)
					},
				});
			}

			// Whatever it was, it was something else!
			return Some(Argument::Other(next));
		}
	}
}




#[derive(Debug, Clone, Eq, PartialEq)]
/// # Parsed Argument.
///
/// This is the return type for the [`Argue`] iterator. In practice, you'll
/// probably want to use a `match` and take the appropriate action given the
/// classification.
pub enum Argument {
	/// # (Sub)command.
	///
	/// This is for arguments matching keywords declared via [`Argue::with_command`]
	/// or [`Argue::with_commands`].
	Command(&'static str),

	/// # Boolean Key.
	///
	/// This is for arguments matching keywords declared via [`Argue::with_key`]
	/// or [`Argue::with_keys`] that do not require a value. (The key is a
	/// boolean flag.)
	Key(&'static str),

	/// # Key and Value.
	///
	/// This is for arguments matching keywords declared via [`Argue::with_key`]
	/// or [`Argue::with_keys`] that require a value.
	///
	/// Note that values are simply "whatever follows" so might represent the
	/// wrong thing if the user mistypes the command.
	KeyWithValue(&'static str, String),

	/// # Everything Else.
	///
	/// This is for arguments that don't meet the criteria for a more specific
	/// [`Argument`] variant.
	Other(String),

	/// # Invalid UTF-8.
	///
	/// This is for arguments that could not be converted to a String because
	/// of invalid UTF-8. The original [`OsString`] representation is maintained
	/// in case you want to dig deeper.
	InvalidUtf8(OsString),

	/// # Everything after "--".
	///
	/// This holds all remaining arguments after the end-of-command terminator.
	/// (The terminator itself is stripped out.)
	///
	/// Note that these arguments are collected as-are with no normalization or
	/// scrutiny of any kind. If you want them parsed, they can be fed directly
	/// into a new [`Argue`] instance.
	///
	/// ## Example
	///
	/// ```
	/// use argyle::stream::{Argue, Argument};
	///
	/// let mut args = argyle::stream::args();
	/// if let Some(Argument::End(extra)) = args.next() {
	///     for arg in Argue::from(extra.into_iter()) {
	///         // Do more stuff!
	///     }
	/// }
	/// ```
	End(Vec<OsString>),
}



#[must_use]
/// # CLI Argument Iterator.
///
/// Return an [`Argue`] iterator seeded with [`ArgsOs`], skipping the first
/// (command path) entry.
///
/// If you'd rather not skip that first entry, create your instance with
/// `Argue::from(std::env::args_os())` instead.
pub fn args() -> Argue<Skip<ArgsOs>> {
	Argue {
		iter: std::env::args_os().skip(1),
		special: BTreeSet::new(),
	}
}



#[derive(Debug, Clone, Copy)]
/// # Key Kinds.
///
/// This enum is used for the internal keyword collection, enabling
/// type-independent matching with the option of subsequently giving a shit
/// about said types. Haha.
enum KeyKind {
	/// # (Sub)command.
	Command(&'static str),

	/// # Boolean key.
	Key(&'static str),

	/// # Key with Value.
	KeyWithValue(&'static str),
}

impl Borrow<str> for KeyKind {
	fn borrow(&self) -> &str { self.as_str() }
}

impl Eq for KeyKind {}

impl Ord for KeyKind {
	#[inline]
	fn cmp(&self, other: &Self) -> Ordering { self.as_str().cmp(other.as_str()) }
}

impl PartialEq for KeyKind {
	#[inline]
	fn eq(&self, other: &Self) -> bool { self.as_str() == other.as_str() }
}

impl PartialOrd for KeyKind {
	#[inline]
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl KeyKind {
	/// # As String Slice.
	///
	/// Return the inner value of the key.
	const fn as_str(&self) -> &'static str {
		match self { Self::Command(s) | Self::Key(s) | Self::KeyWithValue(s) => s }
	}
}



/// # Valid Command?
const fn valid_command(mut key: &[u8]) -> bool {
	// The first character must be alphanumeric.
	let [b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9', rest @ ..] = key else { return false; };
	key = rest;

	// The remaining characters must be alphanumeric, dashes, or underscores.
	while let [a, rest @ ..] = key {
		if ! valid_key_byte(*a) { return false; }
		key = rest;
	}

	true
}

/// # Valid Key?
const fn valid_key(mut key: &[u8]) -> bool {
	// Strip leading dashes.
	let len = key.len();
	while let [b'-', rest @ ..] = key { key = rest; }
	let dashes = len - key.len();

	// A short key must be exactly two bytes with an alphanumeric second.
	if dashes == 1 {
		return key.len() == 1 && key[0].is_ascii_alphanumeric();
	}

	// Long keys must be at least three bytes with the third alphanumeric. If
	// longer, everything else must be alphanumeric or a dash or underscore.
	if dashes == 2 {
		let [b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9', rest @ ..] = key else { return false; };
		key = rest;
		while let [a, rest @ ..] = key {
			if ! valid_key_byte(*a) { return false; }
			key = rest;
		}
		return true;
	}

	false
}

/// # Valid Key Char?
const fn valid_key_byte(b: u8) -> bool {
	matches!(b, b'-' | b'_' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9')
}



#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn t_argue() {
		use std::ffi::OsStr;
		let mut cli = vec![
			OsStr::new("subcommand").to_owned(),
			OsStr::new("").to_owned(),
			OsStr::new("-s").to_owned(),
			OsStr::new("--long").to_owned(),
			OsStr::new("-t2").to_owned(),
			OsStr::new("--m=yar").to_owned(),
			OsStr::new("--n").to_owned(),
			OsStr::new("yar").to_owned(),
			OsStr::new("-u").to_owned(),
			OsStr::new("2").to_owned(),
			OsStr::new("/foo/bar").to_owned(),
			OsStr::new("--").to_owned(),
		];
		let mut args = Argue::from(cli.iter().cloned())
			.with_command("subcommand").unwrap()
			.with_keys([
				("--long", false),
				("--m", true),
				("--n", true),
				("-s", false),
				("-t", true),
				("-u", true),
			]).unwrap();

		assert_eq!(args.next(), Some(Argument::Command("subcommand")));
		assert_eq!(args.next(), Some(Argument::Key("-s")));
		assert_eq!(args.next(), Some(Argument::Key("--long")));
		assert_eq!(args.next(), Some(Argument::KeyWithValue("-t", "2".to_owned())));
		assert_eq!(args.next(), Some(Argument::KeyWithValue("--m", "yar".to_owned())));
		assert_eq!(args.next(), Some(Argument::KeyWithValue("--n", "yar".to_owned())));
		assert_eq!(args.next(), Some(Argument::KeyWithValue("-u", "2".to_owned())));
		assert_eq!(args.next(), Some(Argument::Other("/foo/bar".to_owned())));
		assert!(args.next().is_none());

		// One more time around with a couple end-of-command args.
		cli.push(OsStr::new("--end").to_owned());
		cli.push(OsStr::new("--m=yar").to_owned());

		let mut args = Argue::from(cli.into_iter())
			.with_command("subcommand").unwrap()
			.with_keys([
				("--long", false),
				("--m", true),
				("--n", true),
				("-s", false),
				("-t", true),
				("-u", true),
			]).unwrap();

		assert_eq!(args.next(), Some(Argument::Command("subcommand")));
		assert_eq!(args.next(), Some(Argument::Key("-s")));
		assert_eq!(args.next(), Some(Argument::Key("--long")));
		assert_eq!(args.next(), Some(Argument::KeyWithValue("-t", "2".to_owned())));
		assert_eq!(args.next(), Some(Argument::KeyWithValue("--m", "yar".to_owned())));
		assert_eq!(args.next(), Some(Argument::KeyWithValue("--n", "yar".to_owned())));
		assert_eq!(args.next(), Some(Argument::KeyWithValue("-u", "2".to_owned())));
		assert_eq!(args.next(), Some(Argument::Other("/foo/bar".to_owned())));
		assert_eq!(
			args.next(),
			Some(Argument::End(vec![
				OsStr::new("--end").to_owned(),
				OsStr::new("--m=yar").to_owned(),
			]))
		);
		assert!(args.next().is_none());
	}

	#[test]
	fn t_argue_with_keys() {
		// Define keys all together.
		let arg1 = Argue::from(std::iter::once(OsString::new()))
			.with_keys([
				("--switch1", false),
				("--switch2", false),
				("--opt1", true),
				("--opt2", true),
			])
			.expect("Argue::with_keys failed.");

		// Define them separately.
		let arg2 = Argue::from(std::iter::once(OsString::new()))
			.with_switches(["--switch1", "--switch2"])
				.expect("Argue::with_switches failed.")
			.with_options(["--opt1", "--opt2"])
				.expect("Argue::with_options failed.");

		// The special list should be the same either way.
		assert_eq!(arg1.special, arg2.special);

		// While we're here, let's make sure we can't repeat a key.
		assert!(arg2.with_key("--switch1", false).is_err());
	}

	#[test]
	fn t_valid_key() {
		// Happy first letters.
		let first: BTreeSet<char> = ('0'..='9').into_iter()
			.chain('a'..='z')
			.chain('A'..='Z')
			.collect();

		// Make sure we got everything!
		assert_eq!(first.len(), 26 * 2 + 10);

		// Any short/long key beginning with one or two dashes and one of these
		// characters should be good.
		let mut key = String::new();
		for i in first.iter().copied() {
			"-".clone_into(&mut key);
			key.push(i);
			assert!(valid_key(key.as_bytes()), "Bug: -{i} should be a valid key.");

			key.insert(0, '-');
			assert!(valid_key(key.as_bytes()), "Bug: --{i} should be a valid key.");

			// Chuck a few extras on there for good measure.
			key.push_str("a-Z_0123");
			assert!(valid_key(key.as_bytes()), "Bug: {key:?} should be a valid key.");
		}

		// These should all be bad.
		for k in [
			"",        // Empty.
			"-",       // No alphanumeric.
			"--",
			"---",
			"--_",
			"--Björk", // The ö is invalid.
			"-abc",    // Too long.
			"-Björk",  // Too long and ö.
			"0",       // No leading dash(es).
			"0bc",
			"_abc",
			"a",
			"A",
			"abc",
		] {
			assert!(! valid_key(k.as_bytes()), "Bug: Key {k:?} shouldn't be valid.");
		}
	}

	#[test]
	fn t_valid_key_byte() {
		for i in 0..u8::MAX {
			// Our validation method uses matches!() instead, so let's make
			// sure there's agreement with this approach.
			let expected: bool =
				i == b'-' ||
				i == b'_' ||
				i.is_ascii_alphanumeric();

			assert_eq!(
				expected,
				valid_key_byte(i),
				"Key byte validation mismatch {i}",
			);
		}
	}
}
