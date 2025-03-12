/*!
# Argyle: Flag Builder.
*/

#![expect(clippy::doc_markdown, reason = "Too annoying for code-gen.")]

mod write;

use std::{
	cmp::Ordering,
	collections::BTreeSet,
	fmt,
	path::Path,
};



#[derive(Debug, Clone)]
/// # (Bit)Flag/Enum Builder.
///
/// [`FlagsBuilder`] is a compile-time (`build.rs`) tool for generating
/// small (single-byte) bitflag enums, with every flag — and _combination_
/// — explicitly defined as its own unique variant.
///
/// It supports `1..=8` primary flags, zero, and a couple hundred combinations
/// (that `argyle` will figure out for you).
///
/// The resulting code contains no unsafe blocks, no dependencies (other than
/// `std`), and no runtime performance penalties, just the warmth and
/// reassurance of a strictly-bound type.
///
/// ## Examples
///
/// As the name suggests, [`FlagsBuilder`] employs builder-style configuration
/// methods, allowing indefinite chaining, start to finish.
///
/// ### `build.rs`
///
/// ```ignore
/// use argyle::FlagsBuilder;
///
/// FlagsBuilder::new("MyFlags") // Name of enum.
///     .with_docs("# My Flags Are Awesome!") // Enum docs.
///     .public() // Make it pub instead of pub(crate).
///     .with_flag("FlagOne", None) // A primary flag.
///     .with_flag("FlagTwo", Some("# Second Flag")) // A primary flag with docs.
///     .with_alias("Both", ["FlagOne", "FlagTwo"], None) // A named combo.
///     .with_defaults(["FlagOne", "FlagTwo"]) // MyFlags::default() == MyFlags::Both
///     .save(std::env::var("OUT_DIR").unwrap().join("flags.rs")); // Save it!
/// ```
///
/// ### `lib.rs`
///
/// To use it, just import the build artifact into your library like:
///
/// ```ignore
/// // Generated by build.rs.
/// include!(concat!(env!("OUT_DIR"), "/flags.rs"));
///
/// // The type is "yours"; expand as needed!
/// impl From<MyFlags> for u8 {
///     #[inline]
///     fn from(src: MyFlags) -> Self { src as Self }
/// }
/// ```
///
/// ### Generated Structure.
///
/// Custom enums generated by [`FlagsBuilder`] implement the various
/// bit-related traits —
/// [`BitAnd`](std::ops::BitAnd)/[`BitAndAssign`](std::ops::BitAndAssign),
/// [`BitOr`](std::ops::BitOr)/[`BitOrAssign`](std::ops::BitOrAssign), and
/// [`BitXor`](std::ops::BitXor)/[`BitXorAssign`](std::ops::BitXorAssign) —
/// so can be worked with in the usual manner.
///
/// A few additional helpers like `contains` and some basic unit tests are
/// provided as well.
///
/// All told, you'll wind up with something like this:
///
/// ```ignore
/// #[repr(u8)]
/// #[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
/// /// # My Awesome Enum.
/// pub(crate) enum AwesomeEnum {
///     #[default]
///     None = 0b0000_u8, // Zero is always called "None".
///     A =    0b0001_u8,
///     B =    0b0010_u8,
///     C =    0b0100_u8,
///     D =    0b1000_u8,
///     // …
///     Z =    0b1111_u8,
/// }
///
/// impl AwesomeEnum {
///     /// # (Primary) Flags.
///     pub(crate) const FLAGS: [Self; 4] = [
///         Self::A, Self::B, Self::C, Self::D,
///     ];
///
///     #[must_use]
///     /// # (Try) From `u8`.
///     ///
///     /// Find and return the flag corresponding to the `u8`, if any.
///     pub(crate) const fn from_u8(num: u8) -> Option<Self> {
///         // …
///         # None
///     }
/// }
///
/// impl AwesomeEnum {
///     #[must_use]
///     /// # Contains Flag?
///     ///
///     /// Returns `true` if `self` is or comprises `other`, `false` if not.
///     pub(crate) const fn contains(self, other: Self) -> bool {
///         // …
///         # true
///     }
///
///     #[must_use]
///     /// # Contains Any Part of Flag?
///     ///
///     /// Returns the bits common to `self` and `other`, if any.
///     pub(crate) const fn contains_any(self, other: Self) -> Option<Self> {
///         // …
///         # None
///     }
///
///     #[must_use]
///     /// # Is None?
///     ///
///     /// Returns `true` if [`AwesomeEnum::None`], meaning no bits are set.
///     pub(crate) const fn is_none(self) -> bool {
///         // …
///         # false
///     }
///
///    #[must_use]
///    /// # With Flag Bits.
///    ///
///    /// Return the combination of `self` and `other`.
///    ///
///    /// This is equivalent to `self | other`, but constant.
///    pub(crate) const fn with(self, other: Self) -> Self {
///        // …
///        # Self::None
///    }
///
///    #[must_use]
///    /// # Without Flag Bits.
///    ///
///    /// Remove `other` from `self`, returning the difference.
///    ///
///    /// This is equivalent to `self & ! other`, but constant.
///    pub(crate) const fn without(self, other: Self) -> Self {
///        // …
///        # Self::None
///    }
/// }
/// ```
pub struct FlagsBuilder {
	/// # Enum Name.
	name: String,

	/// # Documentation.
	docs: String,

	/// # Enum/Member Visibility.
	scope: Scope,

	/// # Primary Flags.
	primary: BTreeSet<Flag>,

	/// # Alias Flags.
	alias: BTreeSet<Flag>,

	/// # Default Flags.
	default: BTreeSet<String>,

	/// # Default All.
	default_all: bool,
}

impl fmt::Display for FlagsBuilder {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let writer = write::FlagsWriter::from_builder(self);
		<write::FlagsWriter as fmt::Display>::fmt(&writer, f)
	}
}

impl FlagsBuilder {
	#[must_use]
	/// # New Instance.
	///
	/// Initialize a new builder for a custom enum named `name`.
	///
	/// As with flag/alias names, the enum name must be ASCII alphanumeric
	/// (alpha first), and PascalCase.
	///
	/// ## Examples
	///
	/// ```
	/// use argyle::FlagsBuilder;
	///
	/// // Start a builder called Settings.
	/// let mut builder = FlagsBuilder::new("Settings");
	/// ```
	///
	/// ## Panics
	///
	/// This method will panic if the name is invalid.
	pub fn new<S: AsRef<str>>(name: S) -> Self {
		let name = flag_ident(name.as_ref());
		let docs = format!("# {name}.");
		Self {
			name,
			docs,
			scope: Scope::PubCrate,
			primary: BTreeSet::new(),
			alias: BTreeSet::new(),
			default: BTreeSet::new(),
			default_all: false,
		}
	}

	#[must_use]
	/// # With Documentation.
	///
	/// Customize the documentation that will wind up being attached to the
	/// enum definition.
	///
	/// ## Examples
	///
	/// ```
	/// use argyle::FlagsBuilder;
	///
	/// FlagsBuilder::new("Settings")
	///     .with_docs("# App Settings
	/// This enum holds the various boolean…");
	/// ```
	///
	/// Long write-ups can quickly get untidy with all the line breaks and
	/// whatnot. Storing the docs externally can be useful:
	///
	/// ```ignore
	/// use argyle::FlagsBuilder;
	///
	/// FlagsBuilder::new("Settings")
	///     .with_docs(include_str!("../skel/settings.txt"));
	/// ```
	pub fn with_docs<S: AsRef<str>>(mut self, docs: S) -> Self {
		let docs = docs.as_ref().trim();
		if ! docs.is_empty() { docs.clone_into(&mut self.docs); }
		self
	}

	#[must_use]
	/// # Private Scope.
	///
	/// By default, the generated enum (and its members) are scoped to
	/// `pub(crate)` visibility; use this method to make them private instead.
	///
	/// ## Examples
	///
	/// ```
	/// use argyle::FlagsBuilder;
	///
	/// FlagsBuilder::new("InternalFlags")
	///     .private();
	/// ```
	pub const fn private(mut self) -> Self {
		self.scope = Scope::Private;
		self
	}

	#[must_use]
	/// # Public Scope.
	///
	/// By default, the generated enum (and its members) are scoped to
	/// `pub(crate)` visibility; use this method to make them public instead.
	///
	/// ## Examples
	///
	/// ```
	/// use argyle::FlagsBuilder;
	///
	/// FlagsBuilder::new("PeoplesFlags")
	///     .public();
	/// ```
	pub const fn public(mut self) -> Self {
		self.scope = Scope::Pub;
		self
	}

	#[must_use]
	/// # With Default(s).
	///
	/// By default, the [`Default`] implementation for custom enums is `None`.
	///
	/// If you'd rather it start with one or more flags flipped on, use this
	/// method to specify them.
	///
	/// If you'd rather default to _everything_, use
	/// [`FlagsBuilder::with_default_all`] instead.
	///
	/// ## Examples
	///
	/// ```
	/// use argyle::FlagsBuilder;
	///
	/// FlagsBuilder::new("Fruit")
	///     .with_flag("Apples", None)
	///     .with_flag("Bananas", None)
	///     .with_flag("Carrots", None)
	///     .with_flag("Dates", None)
	///     .with_defaults(["Apples", "Bananas", "Carrots"]);
	/// ```
	///
	/// ## Panics
	///
	/// This method doesn't check that the flags have been defined — that
	/// happens during save — but will panic if the names are
	/// invalid.
	pub fn with_defaults<S, I>(mut self, flags: I) -> Self
	where
		S: AsRef<str>,
		I: IntoIterator<Item=S>,
	{
		self.default.extend(flags.into_iter().map(|f| flag_ident(f.as_ref())));
		self
	}

	#[must_use]
	/// # Default Everything!
	///
	/// By default, the [`Default`] implementation for custom enums is `None`.
	///
	/// Use this method to have it return _all the bits_ (every flag) instead.
	///
	/// If you'd rather the default fall somewhere between the extremes, use
	/// [`FlagsBuilder::with_defaults`] instead.
	///
	/// ## Examples
	///
	/// ```
	/// use argyle::FlagsBuilder;
	///
	/// FlagsBuilder::new("Fruit")
	///     .with_flag("Apples", None)
	///     .with_flag("Bananas", None)
	///     .with_flag("Carrots", None)
	///     .with_flag("Dates", None)
	///     .with_default_all();
	/// ```
	pub const fn with_default_all(mut self) -> Self {
		self.default_all = true;
		self
	}
}

impl FlagsBuilder {
	/// # Check for Duplicates.
	///
	/// Initialize a new flag with the given name and return it, provided it
	/// has not already been defined as a primary flag or alias.
	///
	/// ## Panics
	///
	/// This method will panic if the name is invalid or the flag has already
	/// been defined.
	fn unique_flag<S: AsRef<str>>(&self, name: S) -> Flag {
		let flag = Flag::new(name);
		assert!(
			! self.alias.contains(&flag) && ! self.primary.contains(&flag),
			"TYPO: duplicate flag/alias ({}). (argyle::FlagsBuilder)",
			flag.name,
		);
		flag
	}
}

/// # Primary Flags.
impl FlagsBuilder {
	#[must_use]
	/// # With Flag.
	///
	/// Use this method to define a new primary flag with the given variant
	/// name and (optional) documentation.
	///
	/// As with the enum itself, names must be ASCII alphanumeric (alpha
	/// first), and PascalCase.
	///
	/// A given enum can have anywhere from `1..=8` primary flags.
	///
	/// ## Examples
	///
	/// ```
	/// use argyle::FlagsBuilder;
	///
	/// FlagsBuilder::new("Animals")
	///     .with_flag("BullFrog", None);
	/// ```
	///
	/// ## Panics
	///
	/// This method will panic if the flag name is invalid or has already been
	/// defined.
	pub fn with_flag<S: AsRef<str>>(mut self, name: S, docs: Option<S>) -> Self {
		let mut flag = self.unique_flag(name);
		if let Some(docs) = docs { flag = flag.with_docs(docs); }
		self.primary.insert(flag);
		self
	}

	#[must_use]
	/// # With Complex Flag.
	///
	/// Like [`FlagsBuilder::with_flag`], but for a flag that implies _other_
	/// flag(s), inheriting their bits (along with its own).
	///
	/// Implied flag names passed to this method must correspond to _flags_;
	/// aliases are not allowed.
	///
	/// ## Examples
	///
	/// ```
	/// use argyle::FlagsBuilder;
	///
	/// FlagsBuilder::new("Animals")
	///     .with_flag("BullFrog", None)                   // 0b0001
	///     .with_flag("Bat", None)                        // 0b0010
	///     .with_complex_flag("Frog", ["BullFrog"], None) // 0b0101
	///     .with_alias("All", ["Bat", "Frog"], None);     // 0b0111
	/// ```
	///
	/// ## Panics
	///
	/// This method will panic if any of the flag names are invalid, or the
	/// main name has already been defined.
	pub fn with_complex_flag<S, I>(mut self, name: S, flags: I, docs: Option<S>) -> Self
	where
		S: AsRef<str>,
		I: IntoIterator<Item=S>,
	{
		let mut flag = self.unique_flag(name).with_deps(flags);
		if let Some(docs) = docs { flag = flag.with_docs(docs); }
		self.primary.insert(flag);
		self
	}

	#[must_use]
	/// # With Alias.
	///
	/// Combinative aliases — the "AB" in `A | B == AB` — are automatically
	/// generated by [`FlagsBuilder`], but can be selectively named if any of
	/// them hold special meaning for you.
	///
	/// As with the enum and primary flags, alias names must be ASCII
	/// alphanumeric (alpha first), and PascalCase.
	///
	/// ## Examples
	///
	/// ```
	/// use argyle::FlagsBuilder;
	///
	/// FlagsBuilder::new("Compression")
	///     .with_flag("FmtAvif", None)
	///     .with_flag("FmtJpeg", None)
	///     .with_flag("FmtJxl", None)
	///     .with_flag("FmtPng", None)
	///     .with_flag("FmtWebP", None)
	///     .with_alias("FmtOld", ["FmtJpeg", "FmtPng"], None)
	///     .with_alias("FmtNew", ["FmtAvif", "FmtJxl", "FmtWebP"], None);
	/// ```
	///
	/// ## Panics
	///
	/// This method will panic if the alias or flag names are invalid, if the
	/// alias has already been defined (including as a flag), or if it
	/// references fewer than two flags.
	pub fn with_alias<S, I>(mut self, name: S, flags: I, docs: Option<S>) -> Self
	where
		S: AsRef<str>,
		I: IntoIterator<Item=S>,
	{
		let mut flag = self.unique_flag(name).with_deps(flags);
		if let Some(docs) = docs { flag = flag.with_docs(docs); }

		assert!(
			1 < flag.deps.len(),
			"TYPO: aliases need at least two references ({}) (argyle::FlagsBuilder)",
			flag.name,
		);

		self.alias.insert(flag);
		self
	}
}

impl FlagsBuilder {
	/// # Save it to a File!
	///
	/// Generate and save the custom flag enum to the specified file.
	///
	/// Note that many environments prohibit writes to arbitrary locations; for
	/// best results, your path should be somewhere under `OUT_DIR`.
	///
	/// ## Examples
	///
	/// ```ignore
	/// let out_dir: &Path = std::env::var("OUT_DIR").unwrap().as_ref();
	/// flags.save(out_dir.join("flags.rs"));
	/// ```
	///
	/// If you'd prefer to handle the saving manually, the code can be
	/// obtained by simply calling `FlagsBuilder::to_string` instead.
	///
	/// ```
	/// use argyle::FlagsBuilder;
	///
	/// let out = FlagsBuilder::new("Names")
	///     .with_flag("John", None)
	///     .with_flag("Jane", None)
	///     .to_string();
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



#[derive(Debug, Clone)]
/// # Flag/Alias (Singular).
///
/// This struct holds a name, documentation, and list of associated flags (if
/// an alias).
struct Flag {
	/// # (Variant) Name.
	name: String,

	/// # Docs.
	docs: String,

	/// # Dependent Flags.
	///
	/// The flags an alias is aliasing.
	deps: BTreeSet<String>,
}

impl Eq for Flag {}
impl PartialEq for Flag {
	fn eq(&self, other: &Self) -> bool { self.name == other.name }
}

impl Ord for Flag {
	fn cmp(&self, other: &Self) -> Ordering { self.name.cmp(&other.name) }
}
impl PartialOrd for Flag {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Flag {
	#[must_use]
	/// # New Flag.
	///
	/// Define and return a new flag variant with the name `name`.
	///
	/// ## Panics
	///
	/// Panics if `name` is invalid.
	fn new<S: AsRef<str>>(name: S) -> Self {
		let name = flag_ident(name.as_ref());
		let docs = format!("# {name}.");
		Self {
			name,
			docs,
			deps: BTreeSet::new(),
		}
	}

	#[must_use]
	/// # With Documentation.
	///
	/// Attach custom documentation to the variant's definition, instead of the
	/// default name-derived title.
	fn with_docs<S: AsRef<str>>(mut self, docs: S) -> Self {
		let docs = docs.as_ref().trim();
		if ! docs.is_empty() { docs.clone_into(&mut self.docs); }
		self
	}

	#[must_use]
	/// # With Dependencies.
	///
	/// Set the flags the alias should be aliasing.
	///
	/// ## Panics
	///
	/// This method will panic if any of the flags are invalid.
	fn with_deps<I, S>(mut self, flags: I) -> Self
	where
		S: AsRef<str>,
		I: IntoIterator<Item=S>
	{
		for flag in flags {
			let flag = flag_ident(flag.as_ref());
			if flag != self.name { self.deps.insert(flag); }
		}

		self
	}
}



#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// # Flags Builder Scope.
///
/// This is used to constrain the visibility of enums and methods generated by
/// [`FlagsBuilder`].
///
/// In the interest of keeping it simple, `pub(super)` and `pub(in)` are
/// unsupported, but that may change if the need arises.
enum Scope {
	/// # Private.
	Private,

	/// # Public.
	Pub,

	/// # Crate-Wide.
	PubCrate,
}

impl fmt::Display for Scope {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Private => Ok(()),
			Self::Pub => f.write_str("pub "),
			Self::PubCrate => f.write_str("pub(crate) "),
		}
	}
}



/// # Sanitize Ident.
///
/// Require the value be ASCII alphanumeric, starting with an alphabetic, and
/// PascalCase.
///
/// ## Panics
///
/// This method will panic if the name is invalid or matches a reserved name
/// (None or any of the Zxx-style combinative names).
fn flag_ident(name: &str) -> String {
	let name = name.trim();
	let name2 = to_pascal_case(name);

	// Panic if the name was reformatted.
	assert!(
		name == name2,
		"TYPO: Ident {name:?} should be formatted {name2:?}. (argyle::FlagsBuilder)"
	);

	// None and Z{02X} are reserved.
	assert!(
		name2 != "None" && ! is_generated_flag(name),
		"TYPO: Idents may not be called ({name2})",
	);

	name2
}

/// # Is Auto-Generated Flag?
///
/// Returns true if the flag name matches one of our `Zxx` variants.
const fn is_generated_flag(name: &str) -> bool {
	matches!(name.as_bytes(), [b'Z', b'0'..=b'9' | b'a'..=b'f', b'0'..=b'9' | b'a'..=b'f'])
}

/// # To Pascal Case.
///
/// Turn `flag_name` into `FlagName`.
fn to_pascal_case(raw: &str) -> String {
	let mut out = String::with_capacity(raw.len());

	// First character should be upper.
	let mut chars = raw.chars();
	let first = chars.next()
		.expect("TYPO: Idents must start with an ASCII alphabetic. (argyle::FlagsBuilder)")
		.to_ascii_uppercase();
	assert!(
		first.is_ascii_alphabetic(),
		"TYPO: Idents must start with an ASCII alphabetic. (argyle::FlagsBuilder)",
	);
	out.push(first);

	// The rest will probably be pushed through as-are, unless they're all
	// uppercase.
	let mut under = false;
	let mut lower = false;
	for c in chars {
		match c {
			'A'..='Z' | '0'..='9' => {
				out.push(c);
				under = false;
			},
			'_' =>  { under = true; },
			'a'..='z' =>
				if under {
					out.push(c.to_ascii_uppercase());
					under = false;
				}
				else {
					out.push(c);
					lower = true;
				},
			_ => panic!("TYPO: Idents must be ASCII alphanumeric. (argyle::FlagsBuilder)"),
		}
	}

	// Lowercase the second half.
	if 1 < out.len() && ! lower { out[1..].make_ascii_lowercase(); }

	out
}

/// # To Snake Case.
///
/// Turn `FlagName` into `flag_name`.
fn to_snake_case(raw: &str) -> String {
	let mut out = String::with_capacity(raw.len());
	let has_lower = raw.chars().any(|c| c.is_ascii_lowercase());
	let mut under = true;
	for c in raw.chars() {
		match c {
			'A'..='Z' => {
				if has_lower && ! under {
					out.push('_');
					under = true;
				}
				out.push(c.to_ascii_lowercase());
			},
			'_' =>  if ! under {
				out.push('_');
				under = true;
			},
			'a'..='z' | '0'..='9' => {
				under = false;
				out.push(c);
			},
			_ => panic!("TYPO: Idents must be ASCII alphanumeric. (argyle::FlagsBuilder)"),
		}
	}

	// Don't end on a _.
	if out.ends_with('_') { out.truncate(out.len() - 1); }

	assert!(
		out.chars().next().is_some_and(|c| c.is_ascii_alphabetic()),
		"TYPO: Idents must start with an ASCII alphabetic. (argyle::FlagsBuilder)",
	);

	out
}



#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn t_pascal_case() {
		for (raw, ex) in [
			("hello", "Hello"),
			("HELLO", "Hello"),
			("hello_world", "HelloWorld"),
			("hello_1world", "Hello1world"),
		] {
			assert_eq!(
				to_pascal_case(raw),
				ex,
				"Pascal case failed for {raw}."
			);
		}
	}

	#[test]
	fn t_snake_case() {
		for (raw, ex) in [
			("hello", "hello"),
			("HELLO", "hello"),
			("hello_world", "hello_world"),
			("HelloWorld", "hello_world"),
		] {
			assert_eq!(
				to_snake_case(raw),
				ex,
				"Snake case failed for {raw}."
			);
		}
	}

	#[test]
	#[should_panic(expected = "TYPO: Idents must be ASCII alphanumeric. (argyle::FlagsBuilder)")]
	fn t_flag_ident_not_alphanumeric() {
		let _res = flag_ident("Hello World");
	}

	#[test]
	#[should_panic(expected = "TYPO: Idents must start with an ASCII alphabetic. (argyle::FlagsBuilder)")]
	fn t_flag_ident_not_first_alpha() {
		let _res = flag_ident("1Direction");
	}

	#[test]
	#[should_panic]
	fn t_flag_ident_not_pascal() {
		let _res = flag_ident("wrong_way");
	}

	#[test]
	#[should_panic(expected = "TYPO: Idents may not be called (None)")]
	fn t_flag_ident_reserved1() {
		let _res = flag_ident("None");
	}

	#[test]
	#[should_panic(expected = "TYPO: Idents may not be called (Zaa)")]
	fn t_flag_ident_reserved2() {
		let _res = flag_ident("Zaa");
	}

	#[test]
	#[should_panic(expected = "TYPO: duplicate flag/alias (Bar). (argyle::FlagsBuilder)")]
	fn t_flag_builder_dupe_flag() {
		FlagsBuilder::new("Foo")
			.with_flag("Bar", None)
			.with_flag("Bar", None)
			.to_string();
	}

	#[test]
	#[should_panic(expected = "TYPO: duplicate flag/alias (Bar). (argyle::FlagsBuilder)")]
	fn t_flag_builder_dupe_alias() {
		FlagsBuilder::new("Foo")
			.with_flag("Foo", None)
			.with_flag("Bar", None)
			.with_flag("Baz", None)
			.with_alias("Bar", ["Foo", "Baz"], None)
			.to_string();
	}

	#[test]
	#[should_panic(expected = "TYPO: aliases need at least two references (Baz) (argyle::FlagsBuilder)")]
	fn t_flag_builder_unalias() {
		FlagsBuilder::new("Foo")
			.with_flag("Bar", None)
			.with_alias("Baz", ["Baz"], None)
			.to_string();
	}

	#[test]
	#[should_panic(expected = "TYPO: flag (Baz) is undefined. (argyle::FlagsBuilder)")]
	fn t_flag_builder_undefined_ref() {
		FlagsBuilder::new("Foo")
			.with_flag("Bar", None)
			.with_complex_flag("Foo", ["Baz"], None)
			.to_string();
	}
}
