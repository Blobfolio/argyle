/*!
# Argyle: Streaming Argument Iterator.
*/

use crate::KeyWord;
use std::{
	collections::BTreeSet,
	env::ArgsOs,
	ffi::OsString,
	iter::Skip,
};



/// # Alias for Env Args.
///
/// This is the return type for [`args`]. It is kinda clunky so downstream
/// users may want to just use this shorthand instead.
pub type ArgueEnv = Argue<Skip<ArgsOs>>;



/// # Streaming Argument Iterator.
///
/// `Argue` occupies the middle ground between the standard library's barebones
/// [`std::env::args_os`] helper and full-service crates like [clap](https://crates.io/crates/clap).
///
/// It performs some basic normalization — it handles string conversion in a
/// non-panicking way, recognizes shorthand value assignments like `-kval`,
/// `-k=val`, `--key=val`, and handles end-of-command (`--`) arguments — and
/// will help identify any special keys/values expected by your app.
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
/// In general, `Argue` tries hard not to get in your way or make too many
/// assumptions — handling and validation are left up to you! — but it does
/// require that commands and keys follow certain basic formatting rules. Check
/// out the [`KeyWord`] documentation for more details.
///
/// ## Examples
///
/// ```
/// use argyle::{Argument, KeyWord};
///
/// // To parse arguments from env, just use the `args` helper method.
/// let args = argyle::args()
///     .with_keywords([
///         KeyWord::key("--version").unwrap(),
///         KeyWord::key("--help").unwrap(),
///     ]);
///
/// for arg in args {
///     match arg {
///         Argument::Key("--help") => println!("Help! Help!"),
///         Argument::Key("--version") => println!("v1.2.3"),
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

	/// # Keywords to Look For.
	keys: BTreeSet<KeyWord>,
}

impl<I: IntoIterator<Item=OsString>> From<I> for Argue<I::IntoIter> {
	#[inline]
	fn from(src: I) -> Self {
		Self {
			iter: src.into_iter(),
			keys: BTreeSet::new(),
		}
	}
}

impl<I> Argue<I> {
	#[must_use]
	/// # With Keywords.
	///
	/// Specify the various keywords you'd like [`Argue`] to keep an eye out
	/// for during parsing. It'll call them out specially if/when they appear.
	///
	/// ## Examples
	///
	/// ```
	/// use argyle::{Argument, KeyWord};
	///
	/// let args = argyle::args()
	///     .with_keywords([
	///         // Boolean keys:
	///         KeyWord::key("--help").unwrap(),
	///         KeyWord::key("-h").unwrap(),
	///
	///         // Keys that expect a value:
	///         KeyWord::key_with_value("-o").unwrap(),
	///         KeyWord::key_with_value("--output").unwrap(),
	///      ]);
	///
	/// for arg in args {
	///     match arg {
	///         Argument::Key("-h" | "--help") => {},
	///         Argument::KeyWithValue("-o" | "--output", value) => {},
	///         _ => {}, // Other stuff.
	///     }
	/// }
	/// ```
	pub fn with_keywords<I2: IntoIterator<Item=KeyWord>>(mut self, keys: I2) -> Self {
		for key in keys {
			// Note: we're using `replace` instead of `insert` to keep the
			// variants synced.
			let _res = self.keys.replace(key);
		}

		self
	}
}

impl<I> Argue<I> {
	/// # Find Key.
	///
	/// Find and return the key associated with `raw`, if any.
	fn find_keyword(&self, raw: &str) -> Option<KeyWord> {
		// Short circuit; keywords must start with a dash or alphanumeric.
		let bytes = raw.as_bytes();
		if bytes.is_empty() || ! (bytes[0] == b'-' || bytes[0].is_ascii_alphanumeric()) {
			return None;
		}

		// Direct hit!
		if let Some(key) = self.keys.get(raw) { return Some(*key); }

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
			self.keys.get(needle).copied()
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
				Err(e) => {
					#[cfg(feature = "try_paths")]
					// Well, not _nothing_; maybe it's a path?
					if matches!(std::fs::exists(&e), Ok(true)) {
						return Some(Argument::Path(e));
					}
					return Some(Argument::InvalidUtf8(e));
				},
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
					KeyWord::Command(_) => Argument::Command(k),
					KeyWord::Key(_) => Argument::Key(k),
					KeyWord::KeyWithValue(_) => {
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
										let mut boo = OsString::from(k);
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

			#[cfg(feature = "try_paths")]
			// Maybe it's a path?
			if matches!(std::fs::exists(&next), Ok(true)) {
				return Some(Argument::Path(OsString::from(next)));
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
	/// This is for arguments matching a [`KeyWord::Command`].
	Command(&'static str),

	/// # Boolean Key.
	///
	/// This is for arguments matching a [`KeyWord::Key`].
	Key(&'static str),

	/// # Key and Value.
	///
	/// This is for arguments matching [`KeyWord::KeyWithValue`], along with
	/// the associated value.
	///
	/// Note: values are simply "the next entry" — unless split off from combo
	/// args like `--key=val` — so may or may not be _logically_ correct, but
	/// that's CLI arguments in a nutshell. Haha.
	KeyWithValue(&'static str, String),

	#[cfg(feature = "try_paths")]
	#[cfg_attr(docsrs, doc(cfg(feature = "try_paths")))]
	/// # Path.
	///
	/// This is for unassociated-and-unrecognized arguments for which
	/// [`std::fs::exists`] return `Ok(true)`.
	///
	/// All other such arguments will be yielded as [`Argument::Other`]
	/// or [`Argument::InvalidUtf8`] instead.
	Path(OsString),

	/// # Everything Else.
	///
	/// This is for arguments that don't meet the criteria for a more specific
	/// [`Argument`] variant.
	Other(String),

	/// # Invalid UTF-8.
	///
	/// This is for arguments that could not be converted to a String because
	/// of invalid UTF-8. The original [`OsString`] representation is passed
	/// through for your consideration.
	InvalidUtf8(OsString),

	/// # Everything after "--".
	///
	/// This holds all remaining arguments after an end-of-command terminator
	/// is encountered. (The terminator itself is stripped out.)
	///
	/// The arguments are collected as-are without any normalization or
	/// parsing. If you _want_ them parsed, you can create a new [`Argue`]
	/// instance from the collection by passing it to `Argue::from`.
	///
	/// ## Example
	///
	/// ```
	/// use argyle::{Argue, Argument};
	///
	/// let mut args = argyle::args();
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
/// entry — the script path — since that isn't super useful.
///
/// (If you disagree on that last point, create your instance using
/// `Argue::from(std::env::args_os())` instead.)
pub fn args() -> Argue<Skip<ArgsOs>> {
	Argue {
		iter: std::env::args_os().skip(1),
		keys: BTreeSet::new(),
	}
}



#[cfg(test)]
mod test {
	use super::*;
	use std::ffi::OsString;

	#[test]
	fn t_argue() {
		let mut cli = vec![
			OsString::from(""),
			OsString::from("-s"),
			OsString::from("--long"),
			OsString::from("-t2"),
			OsString::from("--m=yar"),
			OsString::from("--n"),
			OsString::from("yar"),
			OsString::from("-u"),
			OsString::from("2"),
			OsString::from("/foo/bar"),
			OsString::from("--"),
		];

		// Without keywords, everything should turn up other.
		let mut args = Argue::from(cli.iter().cloned());
		assert_eq!(args.next(), Some(Argument::Other("-s".to_owned())));
		assert_eq!(args.next(), Some(Argument::Other("--long".to_owned())));
		assert_eq!(args.next(), Some(Argument::Other("-t2".to_owned())));
		assert_eq!(args.next(), Some(Argument::Other("--m=yar".to_owned())));
		assert_eq!(args.next(), Some(Argument::Other("--n".to_owned())));
		assert_eq!(args.next(), Some(Argument::Other("yar".to_owned())));
		assert_eq!(args.next(), Some(Argument::Other("-u".to_owned())));
		assert_eq!(args.next(), Some(Argument::Other("2".to_owned())));
		assert_eq!(args.next(), Some(Argument::Other("/foo/bar".to_owned())));
		assert_eq!(args.next(), None); // Without trailing arguments, the end is noned.

		// Try again with some keywords.
		args = Argue::from(cli.iter().cloned())
			.with_keywords([
				KeyWord::Key("-s"),
				KeyWord::Key("--long"),
				KeyWord::KeyWithValue("-t"),
				KeyWord::KeyWithValue("--m"),
				KeyWord::KeyWithValue("--n"),
				KeyWord::KeyWithValue("-u"),
			]);
		assert_eq!(args.next(), Some(Argument::Key("-s")));
		assert_eq!(args.next(), Some(Argument::Key("--long")));
		assert_eq!(args.next(), Some(Argument::KeyWithValue("-t", "2".to_owned())));
		assert_eq!(args.next(), Some(Argument::KeyWithValue("--m", "yar".to_owned())));
		assert_eq!(args.next(), Some(Argument::KeyWithValue("--n", "yar".to_owned())));
		assert_eq!(args.next(), Some(Argument::KeyWithValue("-u", "2".to_owned())));
		assert_eq!(args.next(), Some(Argument::Other("/foo/bar".to_owned())));
		assert_eq!(args.next(), None); // Without trailing arguments, the end is noned.

		// Add some trailing arguments for good measure.
		cli.push(OsString::from("Björk"));
		cli.push(OsString::from("is"));
		cli.push(OsString::from("best"));

		args = Argue::from(cli.iter().cloned())
			.with_keywords([
				KeyWord::Key("-s"),
				KeyWord::Key("--long"),
				KeyWord::KeyWithValue("-t"),
				KeyWord::KeyWithValue("--m"),
				KeyWord::KeyWithValue("--n"),
				KeyWord::KeyWithValue("-u"),
			]);
		assert_eq!(args.next(), Some(Argument::Key("-s")));
		assert_eq!(args.next(), Some(Argument::Key("--long")));
		assert_eq!(args.next(), Some(Argument::KeyWithValue("-t", "2".to_owned())));
		assert_eq!(args.next(), Some(Argument::KeyWithValue("--m", "yar".to_owned())));
		assert_eq!(args.next(), Some(Argument::KeyWithValue("--n", "yar".to_owned())));
		assert_eq!(args.next(), Some(Argument::KeyWithValue("-u", "2".to_owned())));
		assert_eq!(args.next(), Some(Argument::Other("/foo/bar".to_owned())));
		assert_eq!(args.next(), Some(Argument::End(vec![
			OsString::from("Björk"),
			OsString::from("is"),
			OsString::from("best"),
		])));
		assert_eq!(args.next(), None);

		// Shorten the test so we can focus on key types.
		cli.truncate(0);
		cli.push(OsString::from("-t2"));
		cli.push(OsString::from("--m=yar"));

		// As before.
		args = Argue::from(cli.iter().cloned())
			.with_keywords([
				KeyWord::Key("--long"), // Unused.
				KeyWord::KeyWithValue("-t"),
				KeyWord::KeyWithValue("--m"),
			]);
		assert_eq!(args.next(), Some(Argument::KeyWithValue("-t", "2".to_owned())));
		assert_eq!(args.next(), Some(Argument::KeyWithValue("--m", "yar".to_owned())));
		assert_eq!(args.next(), None);

		// The values should get dropped for booleans.
		args = Argue::from(cli.iter().cloned())
			.with_keywords([
				KeyWord::Key("-t"),
				KeyWord::Key("--m"),
			]);
		assert_eq!(args.next(), Some(Argument::Key("-t")));
		assert_eq!(args.next(), Some(Argument::Key("--m")));
		assert_eq!(args.next(), None);
	}

	#[test]
	fn t_argue_duplicate() {
		let cli: Vec<OsString> = Vec::new();
		let mut args = Argue::from(cli.iter().cloned())
			.with_keywords([KeyWord::Key("-h")]);

		// It should be a boolean.
		let key = args.keys.get("-h").copied().unwrap();
		assert!(matches!(key, KeyWord::Key("-h")));
		assert!(! matches!(key, KeyWord::KeyWithValue("-h")));

		// Now it should require a value.
		args = args.with_keywords([KeyWord::KeyWithValue("-h")]);
		let key = args.keys.get("-h").copied().unwrap();
		assert!(! matches!(key, KeyWord::Key("-h")));
		assert!(matches!(key, KeyWord::KeyWithValue("-h")));
	}
}
