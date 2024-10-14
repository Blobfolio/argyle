/*!
# Argyle: Argument Iterator
*/

use std::{
	ffi::OsStr,
	os::unix::ffi::OsStrExt,
};



#[derive(Debug, Clone)]
#[deprecated(since = "0.9.0", note = "use stream::Argue instead")]
/// # Argument `OsStr` Iterator.
///
/// This iterates through the arguments of an [`Argue`](crate::Argue) as [`OsStr`](std::ffi::OsStr) values.
pub struct ArgsOsStr<'a> {
	/// # Slice.
	///
	/// A borrowed copy of the arguments, as a slice.
	inner: &'a [Vec<u8>],

	/// # Next Index.
	///
	/// The position of the next argument to pull, i.e. `inner[pos]`. The value
	/// is incremented after each successful fetch.
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



#[derive(Debug, Clone, Default)]
#[deprecated(since = "0.9.0", note = "use stream::Argue instead")]
/// # Option Values Iterator.
///
/// This iterator yields the value(s) corresponding to a given option, useful
/// for commands that accept the same argument multiple times.
///
/// It is the return value for [`Argue::option_values`](crate::Argue::option_values) and [`Argue::option2_values`](crate::Argue::option2_values).
pub struct Options<'a> {
	/// # Found-but-Unyielded Values.
	///
	/// If iteration encounters more values than it can return, the extras are
	/// added to this buffer so they can be yielded on subsequent passes.
	buf: Vec<&'a [u8]>,

	/// # Slice.
	///
	/// A borrowed copy of the arguments. Note iteration potentially shrinks
	/// this slice. If both it and `buf` are empty, iteration is done.
	inner: &'a [Vec<u8>],

	/// # Needle.
	///
	/// Only values corresponding to this key are yielded.
	k1: &'a [u8],

	/// # Optional Second Needle.
	k2: Option<&'a [u8]>,

	/// # Value Delimiter.
	///
	/// If specified, a matching value will be split on this character,
	/// potentially yielded multiple values instead of just one. For example,
	/// a comma would turn `one,two,three` into `one`, `two`, and `three`.
	delimiter: Option<u8>,
}

impl<'a> Iterator for Options<'a> {
	type Item = &'a [u8];

	fn next(&mut self) -> Option<Self::Item> {
		// Steal from the buffer first.
		if let Some(o) = self.buf.pop() { return Some(o); }

		// Cut away parts until we reach the end.
		let mut found = false;
		while let [first, rest @ ..] = self.inner {
			self.inner = rest;

			if found {
				// If we're splitting values, use the buffer.
				if let Some(d) = self.delimiter {
					self.buf.extend(first.split(|&b| b == d));
					self.buf.reverse();
					return self.buf.pop();
				}

				// Otherwise return it whole.
				return Some(first.as_slice());
			}
			else if self.k1 == first || self.k2.map_or(false, |k2| k2 == first) {
				found = true;
			}
		}

		None
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(0, Some(self.inner.len() + self.buf.len()))
	}
}

impl<'a> Options<'a> {
	/// # New.
	pub(crate) const fn new(
		inner: &'a [Vec<u8>],
		k1: &'a [u8],
		k2: Option<&'a [u8]>,
		delimiter: Option<u8>
	) -> Self {
		Self {
			buf: Vec::new(),
			inner, k1, k2, delimiter,
		}
	}
}



#[derive(Debug, Clone, Default)]
#[deprecated(since = "0.9.0", note = "use stream::Argue instead")]
/// # Option Values (`OsStr`) Iterator.
///
/// This iterator yields the value(s) corresponding to a given option, useful
/// for commands that accept the same argument multiple times.
///
/// It is the return value for [`Argue::option_values_os`](crate::Argue::option_values_os) and [`Argue::option2_values_os`](crate::Argue::option2_values_os).
pub struct OptionsOsStr<'a>(pub(crate) Options<'a>);

impl<'a> Iterator for OptionsOsStr<'a> {
	type Item = &'a OsStr;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(OsStr::from_bytes)
	}

	fn size_hint(&self) -> (usize, Option<usize>) { self.0.size_hint() }
}



#[cfg(test)]
mod tests {
	use super::*;
	use crate::Argue;

	#[test]
	fn t_option_values() {
		let base: Vec<&[u8]> = vec![
			b"hey",
			b"-kVal",
			b"-k",
			b"hello,world",
			b"--key=nice",
		];

		let args: Argue = base.iter().copied().collect();

		assert_eq!(
			args.option_values(b"-k", None).collect::<Vec<&[u8]>>(),
			[&b"Val"[..], b"hello,world"],
		);

		assert_eq!(
			args.option_values(b"-k", Some(b',')).collect::<Vec<&[u8]>>(),
			[&b"Val"[..], b"hello", b"world"],
		);

		assert_eq!(
			args.option2_values(b"-k", b"--key", None).collect::<Vec<&[u8]>>(),
			[&b"Val"[..], b"hello,world", b"nice"],
		);

		assert_eq!(
			args.option2_values(b"-k", b"--key", Some(b',')).collect::<Vec<&[u8]>>(),
			[&b"Val"[..], b"hello", b"world", b"nice"],
		);
	}

	#[test]
	fn t_option_values_os() {
		let base: Vec<&[u8]> = vec![
			b"hey",
			b"-kVal",
			b"-k",
			b"hello,world",
			b"--key=nice",
		];

		let args: Argue = base.iter().copied().collect();

		assert_eq!(
			args.option_values_os(b"-k", None).collect::<Vec<&OsStr>>(),
			[OsStr::new("Val"), OsStr::new("hello,world")],
		);

		assert_eq!(
			args.option_values_os(b"-k", Some(b',')).collect::<Vec<&OsStr>>(),
			[OsStr::new("Val"), OsStr::new("hello"), OsStr::new("world")],
		);

		assert_eq!(
			args.option2_values_os(b"-k", b"--key", None).collect::<Vec<&OsStr>>(),
			[OsStr::new("Val"), OsStr::new("hello,world"), OsStr::new("nice")],
		);

		assert_eq!(
			args.option2_values_os(b"-k", b"--key", Some(b',')).collect::<Vec<&OsStr>>(),
			[OsStr::new("Val"), OsStr::new("hello"), OsStr::new("world"), OsStr::new("nice")],
		);
	}
}
