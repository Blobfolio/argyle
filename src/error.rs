/*!
# Argyle: Errors

This is the obligatory error enum.
*/

use std::{
	error::Error,
	fmt,
};



#[derive(Debug, Clone)]
/// # Error Struct.
pub enum ArgyleError {
	/// A custom error.
	Custom(&'static str),
	/// Missing anything/everything.
	Empty,
	/// No trailing args.
	NoArg,
	/// Expected subcommand.
	NoSubCmd,
	/// Miscellaneous Silent Failure.
	///
	/// This has no corresponding error text, but does have its own exit code.
	Passthru(i32),
	/// Too many options defined.
	TooManyKeys,
	/// Wants subcommand help.
	WantsDynamicHelp(Option<Box<[u8]>>),
	/// Wants help.
	WantsHelp,
	/// Wants version.
	WantsVersion,
}

impl AsRef<str> for ArgyleError {
	fn as_ref(&self) -> &str {
		match self {
			Self::Custom(s) => s,
			Self::Empty => "Missing options, flags, arguments, and/or ketchup.",
			Self::NoArg => "Missing required trailing argument.",
			Self::NoSubCmd => "Missing/invalid subcommand.",
			Self::Passthru(_)
				| Self::WantsDynamicHelp(_)
				| Self::WantsHelp
				| Self::WantsVersion => "",
			Self::TooManyKeys => "Too many keys.",
		}
	}
}

impl Error for ArgyleError {}

impl fmt::Display for ArgyleError {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_ref())
	}
}

impl ArgyleError {
	#[must_use]
	/// # Exit code.
	pub const fn exit_code(&self) -> i32 {
		match self {
			Self::Passthru(c) => *c,
			Self::WantsDynamicHelp(_)
				| Self::WantsHelp
				| Self::WantsVersion => 0,
			_ => 1,
		}
	}
}
