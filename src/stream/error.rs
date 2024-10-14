/*!
# Argyle: Errors.
*/

use std::fmt;



#[derive(Debug, Clone, Copy)]
/// # Error!
pub enum ArgyleError {
	/// # Duplicate Key.
	DuplicateKey(&'static str),

	/// # Invalid Key.
	InvalidKey(&'static str),
}

impl std::error::Error for ArgyleError {}

impl fmt::Display for ArgyleError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::DuplicateKey(s) => write!(f, "Duplicate key: {s}"),
			Self::InvalidKey(s) => write!(f, "Invalid key: {s}"),
		}
	}
}
