/*!
# Argyle: Keywords.
*/

use std::{
	borrow::Borrow,
	cmp::Ordering,
	collections::BTreeMap,
	fmt,
	path::Path,
};



#[derive(Debug, Clone, Copy)]
/// # Keyword.
///
/// This enum is used by [`Argue::with_keywords`](crate::Argue::with_keywords)
/// to declare the special CLI (sub)commands and/or keys used by the app.
///
/// Each variant has its own formatting requirements, so it is recommended you
/// create new instances using the [`KeyWord::command`], [`KeyWord::key`], and
/// [`KeyWord::key_with_value`] methods rather than populating variants
/// directly.
///
/// For a compile-time alternative, see [`KeyWordsBuilder`].
///
/// Note that for the purposes of equality and ordering, the variants are
/// irrelevant; only the words are used.
///
/// For example, the following are "equal" despite one requiring a value and
/// one not.
///
/// ```
/// assert_eq!(
///     argyle::KeyWord::key("--help"),
///     argyle::KeyWord::key_with_value("--help"),
/// );
/// ```
pub enum KeyWord {
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
	#[must_use]
	/// # New (Sub)Command.
	///
	/// Validate and return a new (sub)command keyword, or `None` if invalid.
	///
	/// (Sub)commands may only contain ASCII alphanumeric characters, `-`,
	/// and `_`, and must begin with an alphanumeric.
	///
	/// ## Examples
	///
	/// ```
	/// use argyle::KeyWord;
	///
	/// // Totally fine.
	/// assert!(KeyWord::command("make").is_some());
	///
	/// // This, however, does not work.
	/// assert!(KeyWord::command("--help").is_none());
	/// ```
	///
	/// For a compile-time alternative, see [`KeyWordsBuilder`].
	pub const fn command(word: &'static str) -> Option<Self> {
		if valid_command(word.as_bytes()) { Some(Self::Command(word)) }
		else { None }
	}

	#[must_use]
	/// # New Boolean Key.
	///
	/// Validate and return a new boolean/switch keyword — a flag that stands
	/// on its own — or `None` if invalid.
	///
	/// Both long and short style keys are supported:
	/// * Short keys must be two bytes: a dash and an ASCII alphanumeric character.
	/// * Long keys can be any length, but must start with two dashes and an ASCII alphanumeric character; all other characters must be alphanumerics, `-`, and `_`.
	///
	/// ## Examples
	///
	/// ```
	/// use argyle::KeyWord;
	///
	/// // Totally fine.
	/// assert!(KeyWord::key("--help").is_some());
	///
	/// // These, however, do not work.
	/// assert!(KeyWord::key("--björk").is_none());
	/// assert!(KeyWord::key("-too_long_to_be_short").is_none());
	/// ```
	///
	/// For a compile-time alternative, see [`KeyWordsBuilder`].
	pub const fn key(keyword: &'static str) -> Option<Self> {
		if valid_key(keyword.as_bytes()) { Some(Self::Key(keyword)) }
		else { None }
	}

	#[must_use]
	/// # New Option Key.
	///
	/// Validate and return a new option keyword — a key that expects a value —
	/// or `None` if invalid.
	///
	/// Both long and short style keys are supported:
	/// * Short keys must be two bytes: a dash and an ASCII alphanumeric character.
	/// * Long keys can be any length, but must start with two dashes and an ASCII alphanumeric character; all other characters must be alphanumerics, `-`, and `_`.
	///
	/// ## Examples
	///
	/// ```
	/// use argyle::KeyWord;
	///
	/// // Totally fine.
	/// assert!(KeyWord::key_with_value("--help").is_some());
	///
	/// // These, however, do not work.
	/// assert!(KeyWord::key_with_value("--björk").is_none());
	/// assert!(KeyWord::key_with_value("-too_long_to_be_short").is_none());
	/// ```
	///
	/// For a compile-time alternative, see [`KeyWordsBuilder`].
	pub const fn key_with_value(keyword: &'static str) -> Option<Self> {
		if valid_key(keyword.as_bytes()) { Some(Self::KeyWithValue(keyword)) }
		else { None }
	}
}

impl KeyWord {
	#[must_use]
	/// # As String Slice.
	///
	/// Return the keyword's inner value.
	pub const fn as_str(&self) -> &'static str {
		match self { Self::Command(s) | Self::Key(s) | Self::KeyWithValue(s) => s }
	}
}



#[derive(Debug, Default, Clone)]
/// # Compile-Time [`KeyWord`]s Codegen.
///
/// This struct can be used by build scripts to generate a [`KeyWord`] array
/// suitable for use with [`Argue::with_keywords`](crate::Argue::with_keywords).
///
/// It provides the same semantic safety guarantees as [`KeyWord::key`]
/// and family, but at compile-time, eliminating the (mild) runtime overhead.
///
/// The builder also frees you from [`KeyWord`]'s usual `&'static` lifetime
/// constraints, allowing for more programmatic population.
///
/// ## Examples
///
/// ```
/// use argyle::KeyWordsBuilder;
///
/// // Start by adding your keywords.
/// let mut words = KeyWordsBuilder::default();
/// for i in 0..10_u8 {
///     words.push_key(format!("-{i}")); // Automation for the win!
/// }
/// ```
///
/// You can grab a copy of the code by leveraging `Display::to_string`, but in
/// most cases you'll probably just want to use [`KeyWordsBuilder::save`] to write
/// it straight to a file:
///
/// ```ignore
/// let out_dir: &Path = std::env::var("OUT_DIR").unwrap().as_ref();
/// words.save(out_dir.join("keyz.rs"));
/// ```
///
/// Having done that, just `include!` it within your library where needed:
///
/// ```ignore
/// let ags = argyle::args()
///     .with_keywords(include!(concat!(env!("OUT_DIR"), "/keyz.rs")));
/// ```
///
/// For a real-world example, check out the build script for [adbyss](https://github.com/Blobfolio/adbyss/blob/master/adbyss/build.rs).
pub struct KeyWordsBuilder(BTreeMap<String, String>);

impl fmt::Display for KeyWordsBuilder {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("[")?;

		let mut iter = self.0.values();
		if let Some(v) = iter.next() {
			// Write the first value.
			<String as fmt::Display>::fmt(v, f)?;

			// Write the rest with leading comma/space separators.
			for v in iter {
				f.write_str(", ")?;
				<String as fmt::Display>::fmt(v, f)?;
			}
		}

		f.write_str("]")
	}
}

impl KeyWordsBuilder {
	#[inline]
	#[must_use]
	/// # Is Empty?
	///
	/// Returns `true` if there are no keywords.
	///
	/// ## Examples
	///
	/// ```
	/// use argyle::KeyWordsBuilder;
	///
	/// // Duh.
	/// assert!(KeyWordsBuilder::default().is_empty());
	/// ```
	pub fn is_empty(&self) -> bool { self.0.is_empty() }

	#[inline]
	#[must_use]
	/// # Length.
	///
	/// Returns the number of keywords currently in the set.
	///
	/// ## Examples
	///
	/// ```
	/// use argyle::KeyWordsBuilder;
	///
	/// let mut builder = KeyWordsBuilder::default();
	/// builder.push_keys([
	///     "-h", "--help",
	///     "-V", "--version",
	/// ]);
	/// assert_eq!(builder.len(), 4);
	/// ```
	pub fn len(&self) -> usize { self.0.len() }
}

impl KeyWordsBuilder {
	/// # Add Keyword.
	///
	/// Add a keyword, ensuring the string portion is unique.
	///
	/// ## Panics
	///
	/// This will panic if the string part is not unique.
	fn push(&mut self, k: &str, v: String) {
		assert!(! self.0.contains_key(k), "Duplicate key: {k}");
		self.0.insert(k.to_owned(), v);
	}

	/// # Add a Command.
	///
	/// Use this to add a [`KeyWord::Command`] to the list.
	///
	/// ## Examples
	///
	/// ```
	/// use argyle::KeyWordsBuilder;
	///
	/// let mut builder = KeyWordsBuilder::default();
	/// builder.push_command("make");
	/// ```
	///
	/// ## Panics
	///
	/// This will panic if the command is invalid or repeated;
	pub fn push_command<S: AsRef<str>>(&mut self, key: S) {
		let k: &str = key.as_ref().trim();
		assert!(valid_command(k.as_bytes()), "Invalid command: {k}");
		let v = format!("argyle::KeyWord::Command({k:?})");
		self.push(k, v);
	}

	/// # Add Commands.
	///
	/// Use this to add one or more [`KeyWord::Command`] to the list.
	///
	/// ## Examples
	///
	/// ```
	/// use argyle::KeyWordsBuilder;
	///
	/// let mut builder = KeyWordsBuilder::default();
	/// builder.push_commands([
	///     "check",
	///     "make",
	///     "test",
	/// ]);
	/// ```
	///
	/// ## Panics
	///
	/// This will panic if any commands are invalid or repeated;
	pub fn push_commands<I: IntoIterator<Item=S>, S: AsRef<str>>(&mut self, keys: I) {
		for k in keys { self.push_command(k); }
	}

	/// # Add a Boolean Key.
	///
	/// Use this to add a [`KeyWord::Key`] to the list.
	///
	/// ## Examples
	///
	/// ```
	/// use argyle::KeyWordsBuilder;
	///
	/// let mut builder = KeyWordsBuilder::default();
	/// builder.push_key("--help");
	/// ```
	///
	/// ## Panics
	///
	/// This will panic if the key is invalid or repeated.
	pub fn push_key<S: AsRef<str>>(&mut self, key: S) {
		let k: &str = key.as_ref().trim();
		assert!(valid_key(k.as_bytes()), "Invalid key: {k}");
		let v = format!("argyle::KeyWord::Key({k:?})");
		self.push(k, v);
	}

	/// # Add Boolean Keys.
	///
	/// Use this to add one or more [`KeyWord::Key`] to the list.
	///
	/// ## Examples
	///
	/// ```
	/// use argyle::KeyWordsBuilder;
	///
	/// let mut builder = KeyWordsBuilder::default();
	/// builder.push_keys([
	///     "-h", "--help",
	///     "-V", "--version",
	/// ]);
	/// ```
	///
	/// ## Panics
	///
	/// This will panic if any keys are invalid or repeated.
	pub fn push_keys<I: IntoIterator<Item=S>, S: AsRef<str>>(&mut self, keys: I) {
		for k in keys { self.push_key(k); }
	}

	/// # Add a Key that Expects a Value.
	///
	/// Use this to add a [`KeyWord::KeyWithValue`] to the list.
	///
	/// ## Examples
	///
	/// ```
	/// use argyle::KeyWordsBuilder;
	///
	/// let mut builder = KeyWordsBuilder::default();
	/// builder.push_key_with_value("--output");
	/// ```
	///
	/// ## Panics
	///
	/// This will panic if the key is invalid or repeated.
	pub fn push_key_with_value<S: AsRef<str>>(&mut self, key: S) {
		let k: &str = key.as_ref().trim();
		assert!(valid_key(k.as_bytes()), "Invalid key: {k}");
		let v = format!("argyle::KeyWord::KeyWithValue({k:?})");
		self.push(k, v);
	}

	/// # Add Keys that Expect Values.
	///
	/// Use this to add one or more [`KeyWord::KeyWithValue`] to the list.
	///
	/// ## Examples
	///
	/// ```
	/// use argyle::KeyWordsBuilder;
	///
	/// let mut builder = KeyWordsBuilder::default();
	/// builder.push_keys_with_values([
	///     "-i", "--input",
	///     "-o", "--output",
	/// ]);
	/// ```
	///
	/// ## Panics
	///
	/// This will panic if any keys are invalid or repeated.
	pub fn push_keys_with_values<I: IntoIterator<Item=S>, S: AsRef<str>>(&mut self, keys: I) {
		for k in keys { self.push_key_with_value(k); }
	}
}

impl KeyWordsBuilder {
	/// # Save it to a File!
	///
	/// Generate and save the [`KeyWord`] array code to the specified file.
	///
	/// Note that many environments prohibit writes to arbitrary locations; for
	/// best results, your path should be somewhere under `OUT_DIR`.
	///
	/// ## Examples
	///
	/// ```ignore
	/// let out_dir: &Path = std::env::var("OUT_DIR").unwrap().as_ref();
	/// words.save(out_dir.join("keyz.rs"));
	/// ```
	///
	/// ## Panics
	///
	/// This method will panic if the write fails for any reason.
	pub fn save<P: AsRef<Path>>(&self, file: P) {
		use std::io::Write;

		let file = file.as_ref();
		let code = self.to_string();

		// Save it!
		assert!(
			std::fs::File::create(file).and_then(|mut out|
				out.write_all(code.as_bytes()).and_then(|()| out.flush())
			).is_ok(),
			"Unable to write to {file:?}.",
		);
	}
}



/// # Valid Command?
const fn valid_command(bytes: &[u8]) -> bool {
	if let [b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9', rest @ ..] = bytes {
		valid_suffix(rest)
	}
	else { false }
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
		let first: BTreeSet<char> = ('0'..='9')
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
		for a in first.iter() {
			// This should work for both long and short.
			assert!(valid_key(format!("-{a}").as_bytes()));
			assert!(valid_key(format!("--{a}").as_bytes()));

			// But not with the wrong number of dashes.
			assert!(! valid_key(format!("{a}").as_bytes()));
			assert!(! valid_key(format!("---{a}").as_bytes()));

			// Longer variations.
			for b in suffix.iter() {
				// This should work for long keys.
				assert!(valid_key(format!("--{a}{b}").as_bytes()));

				// But not any other dash count.
				assert!(! valid_key(format!("{a}{b}").as_bytes()));
				assert!(! valid_key(format!("-{a}{b}").as_bytes()));
				assert!(! valid_key(format!("---{a}{b}").as_bytes()));

				// Not with bad stuff though.
				for c in bad.iter() {
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

	#[test]
	fn t_builder() {
		let mut builder = KeyWordsBuilder::default();
		assert_eq!(builder.to_string(), "[]"); // Empty.

		builder.push_key("-h");
		assert_eq!(builder.to_string(), "[argyle::KeyWord::Key(\"-h\")]");

		builder.push_key("--help");
		assert_eq!(
			builder.to_string(),
			"[argyle::KeyWord::Key(\"--help\"), argyle::KeyWord::Key(\"-h\")]"
		);

		builder.push_key_with_value("--output");
		assert_eq!(
			builder.to_string(),
			"[argyle::KeyWord::Key(\"--help\"), argyle::KeyWord::KeyWithValue(\"--output\"), argyle::KeyWord::Key(\"-h\")]"
		);

		builder.push_command("make");
		assert_eq!(
			builder.to_string(),
			"[argyle::KeyWord::Key(\"--help\"), argyle::KeyWord::KeyWithValue(\"--output\"), argyle::KeyWord::Key(\"-h\"), argyle::KeyWord::Command(\"make\")]"
		);
	}

	#[test]
	#[should_panic]
	fn t_builder_invalid() {
		let mut builder = KeyWordsBuilder::default();
		builder.push_key("--Björk"); // Invalid characters.
	}

	#[test]
	#[should_panic]
	fn t_builder_duplicate() {
		let mut builder = KeyWordsBuilder::default();
		builder.push_key("--help");
		builder.push_key_with_value("--help"); // Repeated string.
	}

	#[test]
	fn t_builder_plural() {
		let mut builder1 = KeyWordsBuilder::default();
		builder1.push_key("-h");
		builder1.push_key("--help");

		let mut builder2 = KeyWordsBuilder::default();
		builder2.push_keys(["-h", "--help"]);

		assert_eq!(builder1.to_string(), builder2.to_string());

		builder1 = KeyWordsBuilder::default();
		builder1.push_key_with_value("-o");
		builder1.push_key_with_value("--output");

		builder2 = KeyWordsBuilder::default();
		builder2.push_keys_with_values(["-o", "--output"]);

		assert_eq!(builder1.to_string(), builder2.to_string());

		builder1 = KeyWordsBuilder::default();
		builder1.push_command("build");
		builder1.push_command("check");

		builder2 = KeyWordsBuilder::default();
		builder2.push_commands(["build", "check"]);

		assert_eq!(builder1.to_string(), builder2.to_string());
	}
}
