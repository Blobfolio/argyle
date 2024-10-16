/*!
# Argyle: Errors.
*/

use std::fmt;



#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// # Error!
pub enum ArgyleError {
	/// # Invalid Key.
	InvalidKey(&'static str),
}

impl std::error::Error for ArgyleError {}

impl fmt::Display for ArgyleError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::InvalidKey(s) => write!(f, "Invalid key: {s}"),
		}
	}
}

impl ArgyleError {
	#[must_use]
	/// # As String Slice.
	pub const fn as_str(&self) -> &'static str {
		"Invalid key."

		/*match self {
			Self::InvalidKey(_) => "Invalid key.",
		}*/
	}
}
