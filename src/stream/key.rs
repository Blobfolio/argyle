/*!
# Argyle: Keywords.
*/

use crate::ArgyleError;
use std::{
	borrow::Borrow,
	cmp::Ordering,
};



#[derive(Debug, Clone, Copy)]
/// # Keywords.
///
/// This enum is used by [`Argue`](crate::Argue) for keyword identification.
///
/// Note: for equality/ordering purposes, the variants are irrelevant; only the
/// iner string slices are compared. For example:
///
/// ```
/// use argyle::KeyWord;
///
/// // These are equal because they both resolve to "--help".
/// assert_eq!(
///     KeyWord::Key("--help"),
///     KeyWord::KeyWithValue("--help"),
/// );
/// ```
pub enum KeyWord {
	#[cfg(feature = "commands")]
	#[cfg_attr(docsrs, doc(cfg(feature = "commands")))]
	/// # (Sub)command.
	Command(&'static str),

	/// # Boolean key.
	Key(&'static str),

	/// # Key with Value.
	KeyWithValue(&'static str),
}

impl Borrow<str> for KeyWord {
	#[inline]
	fn borrow(&self) -> &str { self.as_str() }
}

impl Eq for KeyWord {}

impl Ord for KeyWord {
	#[inline]
	fn cmp(&self, other: &Self) -> Ordering { self.as_str().cmp(other.as_str()) }
}

impl PartialEq for KeyWord {
	#[inline]
	fn eq(&self, other: &Self) -> bool { self.as_str() == other.as_str() }
}

impl PartialOrd for KeyWord {
	#[inline]
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl KeyWord {
	#[cfg(feature = "commands")]
	#[cfg_attr(docsrs, doc(cfg(feature = "commands")))]
	/// # New (Sub)Command (Checked).
	///
	/// Use this method to create a new (sub)command keyword that is
	/// guaranteed to be semantically valid.
	///
	/// That is, commands may only contain ASCII alphanumeric characters, `-`,
	/// and `_`, and must begin with an alphanumeric.
	///
	/// In practice, most command-like strings will match just fine even if
	/// these rules are ignored, but they might not.
	///
	/// ## Examples
	///
	/// ```
	/// use argyle::KeyWord;
	///
	/// // Checking doesn't much matter here:
	/// assert_eq!(
	///     KeyWord::checked_command("make").unwrap(),
	///     KeyWord::Command("make"),
	/// );
	///
	/// // But would save you from mistakes like this:
	/// assert!(KeyWord::checked_command("--help").is_err());
	/// ```
	///
	/// ## Errors
	///
	/// This will return an error if the command contains invalid characters.
	pub const fn checked_command(keyword: &'static str) -> Result<Self, ArgyleError> {
		if let [b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9', rest @ ..] = keyword.as_bytes() {
			if valid_suffix(rest) { return Ok(Self::Command(keyword)); }
		}

		Err(ArgyleError::InvalidKeyWord(keyword))
	}

	/// # New Boolean Key (Checked).
	///
	/// Use this method to create a new boolean key(word) that is
	/// guaranteed to be semantically valid.
	///
	/// For short keys, that means one dash and one ASCII alphanumeric
	/// character.
	///
	/// For long keys, that means two dashes, one ASCII alphanumeric character,
	/// and any number of alphanumerics, `-`, and `_`.
	///
	/// Key/value splitting, in particular, depends on these rules, so if you
	/// manually declare something like `KeyWord::Key(-björk)`, matching will
	/// be inconsistent.
	///
	/// ## Examples
	///
	/// ```
	/// use argyle::KeyWord;
	///
	/// // Checking doesn't much matter here:
	/// assert_eq!(
	///     KeyWord::checked_key("--help").unwrap(),
	///     KeyWord::Key("--help"),
	/// );
	///
	/// // But would save you from mistakes like this:
	/// assert!(KeyWord::checked_key("-help").is_err());
	/// ```
	///
	/// ## Errors
	///
	/// This will return an error if the key contains invalid characters.
	pub const fn checked_key(keyword: &'static str) -> Result<Self, ArgyleError> {
		if valid_key(keyword.as_bytes()) { Ok(Self::Key(keyword)) }
		else { Err(ArgyleError::InvalidKeyWord(keyword)) }
	}

	/// # New Key w/ Value (Checked).
	///
	/// Use this method to create a new option key(word) that is guaranteed to
	/// be semantically valid.
	///
	/// For short keys, that means one dash and one ASCII alphanumeric
	/// character.
	///
	/// For long keys, that means two dashes, one ASCII alphanumeric character,
	/// and any number of alphanumerics, `-`, and `_`.
	///
	/// Key/value splitting, in particular, depends on these rules, so if you
	/// manually declare something like `KeyWord::Key(-björk)`, matching will
	/// be inconsistent.
	///
	/// ## Examples
	///
	/// ```
	/// use argyle::KeyWord;
	///
	/// // Checking doesn't much matter here:
	/// assert_eq!(
	///     KeyWord::checked_key_with_value("--output").unwrap(),
	///     KeyWord::KeyWithValue("--output"),
	/// );
	///
	/// // But would save you from mistakes like this:
	/// assert!(KeyWord::checked_key_with_value("-output").is_err());
	/// ```
	///
	/// ## Errors
	///
	/// This will return an error if the key contains invalid characters.
	pub const fn checked_key_with_value(keyword: &'static str)
	-> Result<Self, ArgyleError> {
		if valid_key(keyword.as_bytes()) { Ok(Self::KeyWithValue(keyword)) }
		else { Err(ArgyleError::InvalidKeyWord(keyword)) }
	}
}

impl KeyWord {
	#[cfg(feature = "commands")]
	#[must_use]
	/// # As String Slice.
	///
	/// Return the keyword's inner value.
	pub const fn as_str(&self) -> &'static str {
		match self { Self::Command(s) | Self::Key(s) | Self::KeyWithValue(s) => s }
	}

	#[cfg(not(feature = "commands"))]
	#[must_use]
	/// # As String Slice.
	///
	/// Return the keyword's inner value.
	pub const fn as_str(&self) -> &'static str {
		match self { Self::Key(s) | Self::KeyWithValue(s) => s }
	}
}



/// # Valid Key?
const fn valid_key(bytes: &[u8]) -> bool {
	match bytes {
		// Short keys are easy.
		[b'-', b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9'] => true,
		// Long keys have a variable suffix.
		[b'-', b'-', b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9', rest @ ..] => valid_suffix(rest),
		// There is no such thing as a medium key.
		_ => false,
	}
}

/// # Valid Keyword Suffix?
///
/// Check that all bytes are ASCII alphanumeric, `-`, or `_`. This is required
/// for both long keys and commands.
const fn valid_suffix(mut bytes: &[u8]) -> bool {
	while let [b'-' | b'_' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9', rest @ ..] = bytes {
		bytes = rest;
	}

	// By process of elimination, everything validated!
	bytes.is_empty()
}



#[cfg(test)]
mod test {
	use super::*;
	use std::collections::BTreeSet;

	#[test]
	fn t_valid_key() {
		let first: BTreeSet<char> = ('0'..='9').into_iter()
			.chain('a'..='z')
			.chain('A'..='Z')
			.collect();

		let suffix: BTreeSet<char> = first.iter()
			.copied()
			.chain(['-', '_'])
			.collect();

		let bad: BTreeSet<char> = ['!', '?', '.', 'ö', '\n'].into_iter().collect();

		// The suffix allows two things the initial character doesn't.
		assert_eq!(first.len(), 26 * 2 + 10);
		assert_eq!(first.len() + 2, suffix.len());

		// None of the bad characters should be present in either.
		assert!(bad.iter().all(|c| ! first.contains(c) && ! suffix.contains(c)));

		// Let's build up some keys to make sure we aren't missing anything
		// in the match-based validation.
		for a in first.iter().copied() {
			// This should work for both long and short.
			assert!(valid_key(format!("-{a}").as_bytes()));
			assert!(valid_key(format!("--{a}").as_bytes()));

			// But not with the wrong number of dashes.
			assert!(! valid_key(format!("{a}").as_bytes()));
			assert!(! valid_key(format!("---{a}").as_bytes()));

			// Longer variations.
			for b in suffix.iter().copied() {
				// This should work for long keys.
				assert!(valid_key(format!("--{a}{b}").as_bytes()));

				// But not any other dash count.
				assert!(! valid_key(format!("{a}{b}").as_bytes()));
				assert!(! valid_key(format!("-{a}{b}").as_bytes()));
				assert!(! valid_key(format!("---{a}{b}").as_bytes()));

				// Not with bad stuff though.
				for c in bad.iter().copied() {
					assert!(! valid_key(format!("--{a}{c}{b}").as_bytes()));
					assert!(! valid_key(format!("--{a}{b}{c}").as_bytes()));
				}
			}
		}

		// Bad letters on their own should never work.
		for c in bad {
			assert!(! valid_key(format!("{c}").as_bytes()));
			assert!(! valid_key(format!("-{c}").as_bytes()));
			assert!(! valid_key(format!("--{c}").as_bytes()));
			assert!(! valid_key(format!("---{c}").as_bytes()));
		}

		// Bad starts should never work either.
		assert!(! valid_key(b""));
		assert!(! valid_key(b"-"));
		assert!(! valid_key(b"--"));
		assert!(! valid_key(b"---"));
	}
}
