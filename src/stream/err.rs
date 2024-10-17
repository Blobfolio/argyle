/*!
# Argyle: Errors.
*/

use std::fmt;



#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// # Error!
pub enum ArgyleError {
	/// # Invalid Key.
	InvalidKeyWord(&'static str),
}

impl std::error::Error for ArgyleError {}

impl fmt::Display for ArgyleError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::InvalidKeyWord(s) => write!(f, "Invalid keyword: {s}"),
		}
	}
}

impl ArgyleError {
	#[must_use]
	/// # As String Slice.
	pub const fn as_str(&self) -> &'static str {
		"Invalid keyword."

		/*match self {
			Self::InvalidKey(_) => "Invalid key.",
		}*/
	}
}
