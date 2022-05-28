/*!
# Argyle: Argument Iterator
*/

use std::{
	ffi::OsStr,
	os::unix::ffi::OsStrExt,
};



#[derive(Debug, Clone)]
/// # Argument `OsStr` Iterator.
///
/// This iterates through the arguments of an [`Argue`](crate::Argue) as [`OsStr`](std::ffi::OsStr) values.
pub struct ArgsOsStr<'a> {
	inner: &'a [Vec<u8>],
	pos: usize,
}

impl<'a> Iterator for ArgsOsStr<'a> {
	type Item = &'a OsStr;

	/// # Next.
	fn next(&mut self) -> Option<Self::Item> {
		if self.pos < self.inner.len() {
			let out = OsStr::from_bytes(&self.inner[self.pos]);
			self.pos += 1;
			Some(out)
		}
		else { None }
	}

	/// # Size Hint.
	fn size_hint(&self) -> (usize, Option<usize>) {
		let len = self.inner.len() - self.pos;
		(len, Some(len))
	}
}

impl ExactSizeIterator for ArgsOsStr<'_> {
	/// # Length.
	fn len(&self) -> usize { self.inner.len() - self.pos }
}

impl<'a> ArgsOsStr<'a> {
	#[inline]
	/// # New.
	pub(crate) const fn new(inner: &'a [Vec<u8>]) -> Self {
		Self {
			inner,
			pos: 0,
		}
	}
}
