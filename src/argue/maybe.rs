/*!
# Argyle: Maybe Extend (fallible Extend)
*/

use crate::{
	Argue,
	ArgyleError,
	KeyKind,
};
use std::borrow::Cow;
use super::{
	FLAG_HAS_HELP,
	FLAG_HAS_VERSION,
};



/// # Helper: Skip leading "empty" entries, and stop on --.
macro_rules! maybe_skip {
	($any:ident, $bytes:ident) => (
		if ! $any {
			if $bytes.is_empty() || $bytes.iter().all(u8::is_ascii_whitespace) {
				continue;
			}
			$any = true;
		}
		if $bytes == b"--" { break; }
	);
}

/// # A Fallible Extend.
///
/// We can use this until Rust lands a `TryExtend` trait.
pub(super) trait MaybeExtend<A> {
	fn maybe_extend<T>(&mut self, iter: T) -> Result<(), ArgyleError>
	where T: Iterator<Item = A> + ExactSizeIterator;
}

impl MaybeExtend<&'static [u8]> for Argue {
	fn maybe_extend<T>(&mut self, iter: T) -> Result<(), ArgyleError>
	where T: Iterator<Item = &'static [u8]> + ExactSizeIterator {
		// Loop and add!
		let mut any = false;
		for bytes in iter {
			maybe_skip!(any, bytes);

			// Find out what we've got!
			match KeyKind::from(bytes) {
				// Passthrough.
				KeyKind::None => { self.args.push(Cow::Borrowed(bytes)); },
				// Record the key and passthrough.
				KeyKind::Short => {
					// Safety: Short keys are always 2 bytes.
					self.set_short_help_version(*unsafe { bytes.get_unchecked(1) });
					self.add_key(Cow::Borrowed(bytes))?;
				},
				// Record the key and passthrough.
				KeyKind::Long => {
					if bytes == b"--version" { self.flags |= FLAG_HAS_VERSION; }
					else if bytes == b"--help" { self.flags |= FLAG_HAS_HELP; }
					self.add_key(Cow::Borrowed(bytes))?;
				},
				// Split a short key/value pair.
				KeyKind::ShortV => {
					let (a, b) = bytes.split_at(2);
					self.add_key_value(Cow::Borrowed(a), Cow::Borrowed(b))?;
				},
				// Split a long key/value pair.
				KeyKind::LongV(x) => {
					let end: usize = x.get() as usize;

					if end + 1 < bytes.len() {
						self.add_key_value(Cow::Borrowed(&bytes[..end]), Cow::Borrowed(&bytes[end + 1..]))?;
					}
					else {
						self.add_key_value(Cow::Borrowed(&bytes[..end]), Cow::Borrowed(&[]))?;
					}
				},
			}
		}

		Ok(())
	}
}

impl MaybeExtend<Vec<u8>> for Argue {
	fn maybe_extend<T>(&mut self, iter: T) -> Result<(), ArgyleError>
	where T: Iterator<Item = Vec<u8>> + ExactSizeIterator {
		// Loop and add!
		let mut any = false;
		for mut bytes in iter {
			maybe_skip!(any, bytes);

			// Find out what we've got!
			match KeyKind::from(&bytes[..]) {
				// Passthrough.
				KeyKind::None => { self.args.push(Cow::Owned(bytes)); },
				// Record the key and passthrough.
				KeyKind::Short => {
					// Safety: Short keys are always 2 bytes.
					self.set_short_help_version(*unsafe { bytes.get_unchecked(1) });
					self.add_key(Cow::Owned(bytes))?;
				},
				// Record the key and passthrough.
				KeyKind::Long => {
					if bytes == b"--version" { self.flags |= FLAG_HAS_VERSION; }
					else if bytes == b"--help" { self.flags |= FLAG_HAS_HELP; }
					self.add_key(Cow::Owned(bytes))?;
				},
				// Split a short key/value pair.
				KeyKind::ShortV => {
					let v2 = bytes.split_off(2);
					self.add_key_value(Cow::Owned(bytes), Cow::Owned(v2))?;
				},
				// Split a long key/value pair.
				KeyKind::LongV(x) => {
					let end: usize = x.get() as usize;

					if end + 1 < bytes.len() {
						let v2 = bytes.split_off(end + 1);
						bytes.truncate(end);
						self.add_key_value(Cow::Owned(bytes), Cow::Owned(v2))?;
					}
					else {
						bytes.truncate(end);
						self.add_key_value(Cow::Owned(bytes), Cow::Borrowed(&[]))?;
					}
				},
			}
		}
		Ok(())
	}
}
