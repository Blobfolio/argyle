/*!
# Argyle: Errors.
*/

use std::{
	ffi::OsString,
	fmt,
};



#[derive(Debug, Clone)]
/// # Error!
pub enum ArgyleError {
	/// # Duplicate Key.
	DuplicateKey(&'static str),

	/// # Invalid Key.
	InvalidKey(&'static str),

	/// # Invalid UTF-8 in Argument.
	InvalidUtf8(OsString),
}

impl fmt::Display for ArgyleError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::DuplicateKey(s) => write!(f, "Duplicate key: {s}"),
			Self::InvalidKey(s) => write!(f, "Invalid key: {s}"),
			Self::InvalidUtf8(s) => write!(f, "Invalid UTF-8: {s:?}"),
		}
	}
}
