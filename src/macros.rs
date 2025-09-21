/*!
# Argyle: Macros.
*/



#[macro_export(local_inner_macros)]
/// # Generate CLI Argument Enum and Parser/Iterator.
///
/// This macro generates a custom enum and iterator to help with CLI argument
/// parsing.
///
/// `argue!` is intended for use cases requiring more than the standard library's
/// barebones [`args_os`](::std::env::args_os) helper, but less than the
/// full-service offerings (and overhead) of a crate like [clap](https://crates.io/crates/clap).
///
/// It'll automatically convert UTF-8 arguments to `String`s (without
/// panicking), untangle combined key/value pair representations like `-kval`
/// or `--key=val`, and stop if/when it encounters an end-of-command terminator
/// (`"--"`).
///
/// The subsequent validation and handling, however, are left _entirely up to
/// you_. Loop, match, and proceed however you see fit!
///
/// ## Example
///
/// ```
/// use argyle::argue;
///
/// // Construct the enum and iterator.
/// argue! {
///     // By default, this macro will call the enum "Argument" and the
///     // iterator "ArgumentIter". If you'd rather they be called something
///     // else, or have a non-private scope, you can override the defaults
///     // by kicking things off with the following.
///
///     /// # My Arguments Enum.
///     ///
///     /// If you supply documentation like this
///     #[doc = "and/or like this"]
///     /// it'll be attached to the generated object.
///     pub             // You can optionally change the scope like so.
///     MyArgument,     // A name and trailing comma are required.
///
///     MyArgumentIter, // Naked works too if you don't care about docs/scope,
///                     // though clippy may scold you. ;)
///
///     // --------------------
///
///     // If you have valueless keywords, they come next as a comma-separated
///     // list.
///     //
///     // Each entry needs an ident for the variant name and one or more
///     // string literals to match against.
///     Help    "-h" "--help",
///     Version "-V" "--version",
///     Stderr       "--stderr",
///
///     // --------------------
///
///     // If you have option keywords, those come next, but require an
///     // "@options" marker to announce their presence.
///     @options
///
///     // The list format is otherwise identical to their valueless
///     // counterparts.
///     Format       "--format",
///     Level   "-l" "--level",
///
///     // --------------------
///
///     // If you'd like to differentiate unmatched _paths_ from arbitrary
///     // string values, you can declare a variant for the purpose like so.
///     @catchall-paths Path,
///
///     // --------------------
///
///     // Last but not least, the enum will need two catchall variants to
///     // handle unmatched String and OsString values.
///     //
///     // By default, these are auto-generated as "Other" and "OtherOs",
///     // but if you'd like to call them something else, now's the time!
///     @catchall Invalid InvalidUtf8,
/// }
///
/// /// # Main.
/// fn main() {
/// # use std::path::PathBuf;
///     // Example settings.
///     let mut stderr = false;
///     let mut format: Option<Format> = None;
///     let mut level = 0_u8;
///     let mut paths: Vec<PathBuf> = Vec::new();
///
///     // Loop through the environmental arguments, taking whatever actions
///     // make sense for your application.
///     for arg in MyArgument::args_os() {
///         match arg {
///             // You named these!
///             MyArgument::Help => print_help(),
///             MyArgument::Version => print_version(),
///             MyArgument::Stderr => { stderr = true; },
///
///             // Options come with the value as a String.
///             MyArgument::Format(v) => {
///                 format = Format::from_str(v);
///             },
///             MyArgument::Level(v) => {
///                 level = v.parse().unwrap_or(0);
///             },
///
///             // If you specified @catchall-paths, unmatched OsString values
///             // that happen to be (valid) filesystem paths will be mapped
///             // thusly (instead of to a generic catchall).
///             MyArgument::Path(v) => {
///                 paths.push(PathBuf::from(v));
///             },
///
///             // Unmatched String values map to the first generic catchall.
///             MyArgument::Invalid(v) => {
///                 eprintln!("Warning: unrecognized CLI argument {v}.");
///             },
///
///             // Unmatched values with invalid UTF-8 will be passed through
///             // to the second generic catchall as OsString values.
///             MyArgument::InvalidUtf8(v) => {
///                 eprintln!(
///                     "Warning: unrecognized CLI argument {}.",
///                     v.display(),
///                 );
///             },
///         }
///     }
///
///     // Now that the settings have been worked out, do something!
///     // …
/// }
/// # fn print_help() {}
/// # fn print_version() {}
/// # enum Format { Plain, Json }
/// # impl Format {
/// #     fn from_str(str: String) -> Option<Self> { None }
/// # }
/// ```
///
/// ## Generated Code.
///
/// If you're curious or need to do something more complicated, taking a look
/// at the generated code can be helpful.
///
/// The call to `argue!` in the previous example, for example, will have added
/// the following to the module:
///
/// ```
/// # use std::env::ArgsOs;
/// # use std::ffi::OsString;
/// # use std::iter::FusedIterator;
/// # use std::iter::Skip;
/// #[derive(Debug, Clone, Eq, PartialEq)]
/// /// # My Arguments Enum.
/// ///
/// /// If you supply documentation like this and/or like this it'll be
/// /// attached to the generated object.
/// pub enum MyArgument {
///     /// # Matches "-h" "--help".
///     Help,
///
///     /// # Matches "-V" "--version".
///     Version,
///
///     /// # Matches "--stderr".
///     Stderr,
///
///     /// # Matches "--format".
///     Format(String),
///
///     /// # Matches "-l" "--level".
///     Level(String),
///
///     /// # Unassociated Path Value.
///     Path(OsString),
///
///     /// # Unspecified Value.
///     Invalid(String),
///
///     /// # Unspecified Value (Invalid UTF-8).
///     InvalidUtf8(OsString),
/// }
///
/// impl MyArgument {
///     /// # Environmental Argument Iterator.
///     ///
///     /// Return a new [`MyArgumentIter`] instance seeded with [`ArgsOs`]
///     /// (minus the first entry corresponding to the executable path).
///     pub fn args_os() -> MyArgumentIter<Skip<ArgsOs>> {
/// # MyArgumentIter::new(::std::env::args_os().skip(1))
///         // …
///     }
/// }
///
/// #[derive(Debug, Clone)]
/// struct MyArgumentIter<T> {
/// #    iter: T,
/// #    done: bool,
///         // …
/// }
///
/// // Note: the generated member methods share the parent's scope. The
/// // iterator was left private in the example, so the generated methods are
/// // private too.
///
/// impl<T: Iterator<Item=OsString>> MyArgumentIter<T> {
///     #[inline]
///     #[must_use]
///     /// # New Instance.
///     ///
///     /// Create and return a new parsing iterator over any arbitrary
///     /// iterator of `OsString`.
///     const fn new(src: T) -> Self {
/// #        Self {
/// #            iter: src,
/// #            done: false,
/// #        }
///         // …
///     }
///
///     #[inline]
///     #[must_use]
///     /// # Into Inner (Iterator).
///     ///
///     /// Return what's left of the inner iterator.
///     fn into_inner(self) -> T {
/// # self.iter
///         // …
///     }
/// }
///
/// impl<T: Iterator<Item=OsString>> Iterator for MyArgumentIter<T> {
///     type Item = MyArgument;
///
///     fn next(&mut self) -> Option<Self::Item> {
/// # None
///         // …
///     }
/// }
///
/// impl<T: Iterator<Item=OsString>> FusedIterator for MyArgumentIter<T> {}
/// ```
///
/// ## Keyword Formatting
///
/// The macro supports (practically) any number of named keywords, with or without values,
/// but there are _rules_ for the literals they match against to ensure proper
/// parsing.
///
/// * Short keys — `"-k"` — must be exactly two bytes: a hyphen and an ASCII alphanumeric.
/// * Long keys — `"--key"` — must start with two hyphens and an ASCII alphanumeric, and contain only alphanumerics, hyphens, and underscores thereafter.
/// * Commands — `"keyword"` — must start with an ASCII alphanumeric, and contain only alphanumerics, hyphens, and underscores thereafter.
///
/// Format sanity is evaluated at compile-time, so issues like the following
/// will trigger an error.
///
/// ```compile_fail
/// argyle::argue! {
///     MyArgument,
///     MyArgumentIter,
///
///     Level "-level", // Not short enough.
/// }
/// ```
///
/// ```compile_fail
/// argyle::argue! {
///     MyArgument,
///     MyArgumentIter,
///
///     Level "-❤️", // Cute, but not ASCII alphanumeric.
/// }
/// ```
///
/// ```compile_fail
/// argyle::argue! {
///     MyArgument,
///     MyArgumentIter,
///
///     FooBar "--foo bar", // Whitespace is illegal.
/// }
/// ```
///
/// ```compile_fail
/// argyle::argue! {
///     MyArgument,
///     MyArgumentIter,
///
///     Build "build!!!", // Settle down…
/// }
/// ```
///
/// This probably goes without saying, but keyword idents and literals must
/// also be unique. Haha.
///
/// ## Parsing Particulars
///
/// Key/value pairs are parsed identically whether they appear consecutively
/// — e.g. `--key` then `value` — or combined in any of the following ways:
/// * `-kvalue`
/// * `-k=value`
/// * `-k = value`
/// * `--key=value`
/// * `--key = value`
///
/// Option values must, however, be valid UTF-8, otherwise the key and value
/// will be returned as a joint `OtherOs(OsString)` in `--key=value` format.
///
/// Keyword matches are otherwise a case-sensitive, all-or-nothing affair.
///
/// Parsing will stop early if an end-of-command terminator (`"--"`) is
/// encountered. If your program needs to handle what comes _after_, adjust
/// the loop like so:
///
/// ```
/// # argyle::argue! {};
/// # type MyArgument = Argument;
/// # type MyArgumentIter<T> = ArgumentIter<T>;
/// // Save the iterator to a variable and traverse it one value at a time
/// // to keep it in scope.
/// let mut args = MyArgument::args_os();
/// while let Some(arg) = args.next() {
///     // Process as normal.
/// }
///
/// // Create a second iterator instance from the remains of the first to
/// // loop through whatever was left, if anything.
/// for arg in MyArgumentIter::new(args.into_inner()) {
///     // Do something.
/// }
/// ```
macro_rules! argue {
	// The full menu.
	(
		$( #[doc = $enum_doc:expr] )*
		$enum_vis:vis $enum:ident,

		$( #[doc = $iter_doc:expr] )*
		$iter_vis:vis $iter:ident,

		$( $key:ident $( $key_lit:literal )+, )*

		$( @options $( $keyvalue:ident $( $keyvalue_lit:literal )+, )+ )?

		$( @catchall-paths $path:ident, )?

		@catchall $other:ident $otheros:ident,
	) => (
		#[allow(dead_code, reason = "Auto-generated.")]
		#[derive(Debug, Clone, Eq, PartialEq)]
		$( #[doc = $enum_doc] )*
		$enum_vis enum $enum {
			$(
				#[doc = ::std::concat!(
					"# Matches",
					$( " \"", ::std::stringify!($key_lit), "\"", )+
					".",
				)]
				$key,
			)*
			$( $(
				#[doc = ::std::concat!(
					"# Matches",
					$( " \"", ::std::stringify!($keyvalue_lit), "\"", )+
					".",
				)]
				$keyvalue(String),
			)+ )?

			$(
				/// # Unassociated Path Value.
				$path(::std::ffi::OsString),
			)?

			/// # Unspecified Value.
			$other(String),

			/// # Unspecified Value (Invalid UTF-8).
			$otheros(::std::ffi::OsString),
		}

		/// # Check Key Validity.
		///
		/// The compiler should optimize this out.
		const _: () = {
			/// # Check Validity.
			const fn check(k: &str) -> bool {
				let mut k = k.as_bytes();
				match k {
					// Short key.
					[b'-', a] => return a.is_ascii_alphanumeric(),

					// Long key/Command.
					[b'-', b'-', a, rest @ ..] |
					[a, rest @ .. ] =>
						if a.is_ascii_alphanumeric() { k = rest; }
						else { return false; },

					// Dunno, but it's wrong.
					_ => return false,
				}

				// Make sure the rest is ASCII alphanumeric or - or _.
				while let [n, rest @ ..] = k {
					if ! (n.is_ascii_alphanumeric() || ::std::matches!(*n, b'-' | b'_')) {
						return false;
					}
					k = rest;
				}

				true
			}

			$($(
				::std::assert!(
					check($key_lit),
					"Invalid `argue!` keyword literal.",
				);
			)+)*
			$($($(
				::std::assert!(
					check($keyvalue_lit),
					"Invalid `argue!` option literal.",
				);
			)+)+)?
		};

		#[allow(dead_code, reason = "Auto-generated.")]
		impl $enum {
			/// # Environmental Argument Iterator.
			///
			/// Return a new
			#[doc = ::std::concat!("[`", ::std::stringify!($iter), "`]")]
			/// instance seeded with [`ArgsOs`](::std::env::ArgsOs) (minus the
			/// first entry corresponding to the executable path).
			$enum_vis fn args_os() -> $iter<::std::iter::Skip<::std::env::ArgsOs>> {
				$iter::new(::std::env::args_os().skip(1))
			}
		}

		#[derive(Debug, Clone)]
		$( #[doc = $iter_doc] )*
		$iter_vis struct $iter<T> {
			/// # Inner Iterator.
			iter: T,

			/// # Terminator "--" Found.
			done: bool,
		}

		#[allow(dead_code, reason = "Auto-generated.")]
		impl<T: Iterator<Item=::std::ffi::OsString>> $iter<T> {
			#[inline]
			#[must_use]
			/// # New Iterator.
			$iter_vis const fn new(src: T) -> Self {
				Self {
					iter: src,
					done: false,
				}
			}

			#[inline]
			#[must_use]
			/// # Into Inner (Iterator).
			///
			/// Return what's left of the inner iterator.
			$iter_vis fn into_inner(self) -> T { self.iter }
		}

		impl<T: Iterator<Item=::std::ffi::OsString>> Iterator for $iter<T> {
			type Item = $enum;

			fn next(&mut self) -> Option<Self::Item> {
				// Already terminated!
				if self.done { return None; }

				loop {
					let next = match self.iter.next()?.into_string() {
						Ok(next) => next,

						// We can't do anything with OsString; return as-is.
						Err(e) => {
							$(
								// Or maybe not nothing…
								if ::std::matches!(::std::fs::exists(&e), Ok(true)) {
									return Some($enum::$path(::std::ffi::OsString::from(e)));
								}
							)?

							return Some($enum::$otheros(e));
						},
					};

					// Skip empty values.
					if next.is_empty() { continue; }

					// If we've hit the separator, gobble up the remaining bits
					// and return.
					if next == "--" {
						self.done = true;
						return None;
					}

					// Try to match a key exactly.
					match next.as_str() {
						$(
							$( $key_lit )|+ => return Some($enum::$key),
						)*
						$($(
							$( $keyvalue_lit )|+ => match self.iter.next()?.into_string() {
								Ok(s) => return Some($enum::$keyvalue(s)),
								// Build a value we can return.
								Err(e) => {
									let mut boo = ::std::ffi::OsString::from(next);
									boo.push("=");
									boo.push(e);
									return Some($enum::$otheros(boo));
								},
							},
						)+)?
						_ => {},
					}

					// Try to match a key-and-value.
					$(
						// Try to match a --key=value.
						if next.starts_with("--") {
							if let Some((a, b)) = next.split_once('=') {
								match a.trim_ascii_end() {
									$(
										$( $keyvalue_lit )|+ => return Some(
											$enum::$keyvalue(b.trim().to_owned())
										),
									)+
									_ => {},
								}
							}
						}

						// Try to match a -kValue.
						else if next.starts_with('-') && let Some((a, b)) = next.split_at_checked(2) {
							match a {
								$(
									$( $keyvalue_lit )|+ => {
										let mut b = b.trim_ascii();
										if let Some(rest) = b.strip_prefix('=') {
											b = rest.trim_ascii_start();
										}

										return Some($enum::$keyvalue(b.to_owned()));
									},
								)+
								_ => {},
							}
						}
					)?

					$(
						// Maybe it's a path?
						if ::std::matches!(::std::fs::exists(&next), Ok(true)) {
							return Some($enum::$path(::std::ffi::OsString::from(next)));
						}
					)?

					// Who knows?
					return Some($enum::$other(next));
				}
			}

			fn size_hint(&self) -> (usize, Option<usize>) {
				if self.done { (0, Some(0)) }
				else {
					let (_, upper) = self.iter.size_hint();
					(0, upper.map(|n| n * 2))
				}
			}
		}

		impl<T: Iterator<Item=::std::ffi::OsString>> ::std::iter::FusedIterator for $iter<T> {}
	);

	// Same as above, but without @catchall overrides.
	(
		$( #[doc = $enum_doc:expr] )*
		$enum_vis:vis $enum:ident,

		$( #[doc = $iter_doc:expr] )*
		$iter_vis:vis $iter:ident,

		$( $key:ident $( $key_lit:literal )+, )*

		$( @options $( $keyvalue:ident $( $keyvalue_lit:literal )+, )+ )?

		$( @catchall-paths $path:ident, )?
	) => (
		// Recurse with the default values filled out.
		$crate::argue! {
			$( #[doc = $enum_doc] )*
			$enum_vis $enum,

			$( #[doc = $iter_doc] )*
			$iter_vis $iter,

			$( $key $( $key_lit )+, )*

			$( @options $( $keyvalue $( $keyvalue_lit )+, )+ )?

			$( @catchall-paths $path, )?

			@catchall Other OtherOs,
		}
	);

	// Same as above, but without the enum/iterator overrides.
	(
		$( $key:ident $( $key_lit:literal )+, )*

		$( @options $( $keyvalue:ident $( $keyvalue_lit:literal )+, )+ )?

		$( @catchall-paths $path:ident, )?

		$( @catchall $other:ident $otheros:ident, )?
	) => (
		// Recurse with the default values filled out.
		$crate::argue! {
			/// # CLI Arguments.
			Argument,

			/// # CLI Argument Iterator.
			ArgumentIter,

			$( $key $( $key_lit )+, )*

			$( @options $( $keyvalue $( $keyvalue_lit )+, )+ )?

			$( @catchall-paths $path, )?

			$( @catchall $other $otheros, )?
		}
	);
}



#[cfg(test)]
mod tests {
	use std::ffi::OsString as Os;

	#[test]
	fn t_argue() {
		argue!{
			/// # My Arguments.
			///
			/// Why argue?
			pub(crate) MyArgument,

			/// # My Argument Iterator.
			///
			/// Why argue?
			pub(crate) MyArgumentIter,

			Help    "-h" "--help"    "help",
			Version "-V" "--version" "version",

			@options
			Output  "-o" "--output",

			@catchall-paths Path,

			// Stick with the default Other/OtherOs catchalls.
		}

		let args: Vec<Os> = vec![
			Os::from("-h"),
			Os::from("--help"),
			Os::from("help"),

			Os::from("-V"),
			Os::from("--version"),
			Os::from("version"),

			Os::from("-o"),
			Os::from("/path/to/foo"),

			Os::from("--output"),
			Os::from("/path/to/foo"),

			Os::from("Cargo.toml"),
			Os::from("Dunno"),

			Os::from("--"),
			Os::from("a"),
			Os::from("b"),
		];

		let mut iter = MyArgumentIter::new(args.into_iter());
		assert_eq!(iter.next(), Some(MyArgument::Help));
		assert_eq!(iter.next(), Some(MyArgument::Help));
		assert_eq!(iter.next(), Some(MyArgument::Help));

		assert_eq!(iter.next(), Some(MyArgument::Version));
		assert_eq!(iter.next(), Some(MyArgument::Version));
		assert_eq!(iter.next(), Some(MyArgument::Version));

		assert_eq!(iter.next(), Some(MyArgument::Output("/path/to/foo".to_owned())));
		assert_eq!(iter.next(), Some(MyArgument::Output("/path/to/foo".to_owned())));

		assert_eq!(iter.next(), Some(MyArgument::Path(Os::from("Cargo.toml"))));
		assert_eq!(iter.next(), Some(MyArgument::Other("Dunno".to_owned())));

		assert!(iter.next().is_none());

		// There should actually be two items left; but we have to reach
		// inside to get 'em.
		let mut iter = iter.into_inner();
		assert_eq!(iter.next(), Some(Os::from("a")));
		assert_eq!(iter.next(), Some(Os::from("b")));

		// Now we should be done for real.
		assert!(iter.next().is_none());
	}

	#[test]
	fn t_kv() {
		argue! {
			@options Key "-k" "--key",
		}

		let mut iter = ArgumentIter::new([
			Os::from("-kValue"),
			Os::from("-k=Value"),
			Os::from("-k = Value"),
			Os::from("-k"),
			Os::from("Value"),
			Os::from("--key=Value"),
			Os::from("--key = Value"),
			Os::from("--key"),
			Os::from("Value"),
		].into_iter());

		assert_eq!(iter.next(), Some(Argument::Key("Value".to_owned())));
		assert_eq!(iter.next(), Some(Argument::Key("Value".to_owned())));
		assert_eq!(iter.next(), Some(Argument::Key("Value".to_owned())));
		assert_eq!(iter.next(), Some(Argument::Key("Value".to_owned())));
		assert_eq!(iter.next(), Some(Argument::Key("Value".to_owned())));
		assert_eq!(iter.next(), Some(Argument::Key("Value".to_owned())));
		assert_eq!(iter.next(), Some(Argument::Key("Value".to_owned())));
		assert!(iter.next().is_none());
	}
}
