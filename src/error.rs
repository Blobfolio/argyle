/*!
# Argyle: Errors

This is the obligatory error enum. It implements `Copy` unless the crate
feature `dynamic-help` is enabled, in which case it can only be `Clone`.
*/

use std::{
	error::Error,
	fmt,
};



#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(not(feature = "dynamic-help"), derive(Copy))]
/// # Error Struct.
pub enum ArgyleError {
	/// A custom error.
	Custom(&'static str),

	/// Missing anything/everything.
	Empty,

	/// Expected subcommand.
	NoSubCmd,

	/// Miscellaneous Silent Failure.
	///
	/// This has no corresponding error text, but does have its own exit code.
	Passthru(i32),

	#[cfg(feature = "dynamic-help")]
	#[cfg_attr(docsrs, doc(cfg(feature = "dynamic-help")))]
	/// Wants subcommand help.
	WantsDynamicHelp(Option<Box<[u8]>>),

	/// Wants help.
	WantsHelp,

	/// Wants version.
	WantsVersion,
}

impl AsRef<str> for ArgyleError {
	#[inline]
	fn as_ref(&self) -> &str { self.as_str() }
}

impl Error for ArgyleError {}

impl fmt::Display for ArgyleError {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl ArgyleError {
	#[must_use]
	/// # Exit code.
	///
	/// This returns the exit code for the error. Non-error errors like help
	/// and version have a non-error exit code of `0`. [`ArgyleError::Passthru`]
	/// returns whatever code was defined, while everything else just returns
	/// `1`.
	pub const fn exit_code(&self) -> i32 {
		match self {
			Self::Passthru(c) => *c,

			#[cfg(feature = "dynamic-help")]
			Self::WantsDynamicHelp(_)
				| Self::WantsHelp
				| Self::WantsVersion => 0,

			#[cfg(not(feature = "dynamic-help"))]
			Self::WantsHelp | Self::WantsVersion => 0,

			_ => 1,
		}
	}

	#[must_use]
	/// # As Str.
	///
	/// Return as a string slice.
	pub const fn as_str(&self) -> &'static str {
		match self {
			Self::Custom(s) => s,
			Self::Empty => "Missing options, flags, arguments, and/or ketchup.",
			Self::NoSubCmd => "Missing/invalid subcommand.",

			#[cfg(feature = "dynamic-help")]
			Self::Passthru(_)
				| Self::WantsDynamicHelp(_)
				| Self::WantsHelp
				| Self::WantsVersion => "",

			#[cfg(not(feature = "dynamic-help"))]
			Self::Passthru(_)
				| Self::WantsHelp
				| Self::WantsVersion => "",

		}
	}
}
