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



/// # A Fallible Extend.
///
/// We can use this until Rust lands a `TryExtend` trait.
pub(super) trait MaybeExtend<A> {
	fn maybe_extend<T>(&mut self, iter: T) -> Result<(), ArgyleError>
	where T: IntoIterator<Item = A>;
}

impl MaybeExtend<&'static [u8]> for Argue {
	fn maybe_extend<T>(&mut self, iter: T) -> Result<(), ArgyleError>
	where T: IntoIterator<Item = &'static [u8]> {
		// Reserve some space.
		let iter = iter.into_iter();
		let (len, _) = iter.size_hint();
		self.args.reserve(len.checked_next_power_of_two().unwrap_or(len));

		// Loop and add!
		let mut any = false;
		for bytes in iter {
			// Skip leading whitespace-only entries.
			if ! any {
				if bytes.is_empty() || bytes.iter().all(u8::is_ascii_whitespace) {
					continue;
				}

				any = true;
			}

			let idx = u16::try_from(self.args.len())
				.map_err(|_| ArgyleError::TooManyArgs)?;

			// Find out what we've got!
			match KeyKind::from(bytes) {
				// Passthrough.
				KeyKind::None => {
					self.args.push(Cow::Borrowed(bytes));
				},
				// Record the key and passthrough.
				KeyKind::Short => {
					if bytes[1] == b'V' { self.flags |= FLAG_HAS_VERSION; }
					else if bytes[1] == b'h' { self.flags |= FLAG_HAS_HELP; }

					self.args.push(Cow::Borrowed(bytes));
					self.insert_key(idx)?;
					self.last.set(idx);
				},
				// Record the key and passthrough.
				KeyKind::Long => {
					if bytes == b"--version" { self.flags |= FLAG_HAS_VERSION; }
					else if bytes == b"--help" { self.flags |= FLAG_HAS_HELP; }

					self.args.push(Cow::Borrowed(bytes));
					self.insert_key(idx)?;
					self.last.set(idx);
				},
				// Split a short key/value pair.
				KeyKind::ShortV => {
					self.args.push(Cow::Borrowed(&bytes[0..2]));
					self.args.push(Cow::Borrowed(&bytes[2..]));

					self.insert_key(idx)?;
					self.last.set(idx.checked_add(1).ok_or(ArgyleError::TooManyArgs)?);
				},
				// Split a long key/value pair.
				KeyKind::LongV(x) => {
					let end: usize = x.get() as usize;
					self.args.push(Cow::Borrowed(&bytes[0..end]));

					if end + 1 < bytes.len() {
						self.args.push(Cow::Borrowed(&bytes[end + 1..]));
					}
					else {
						self.args.push(Cow::Owned(Vec::new()));
					}

					self.insert_key(idx)?;
					self.last.set(idx.checked_add(1).ok_or(ArgyleError::TooManyArgs)?);
				},
			}
		}

		Ok(())
	}
}

impl MaybeExtend<Vec<u8>> for Argue {
	fn maybe_extend<T>(&mut self, iter: T) -> Result<(), ArgyleError>
	where T: IntoIterator<Item = Vec<u8>> {
		// Reserve some space.
		let iter = iter.into_iter();
		let (len, _) = iter.size_hint();
		self.args.reserve(len.checked_next_power_of_two().unwrap_or(len));

		// Loop and add!
		let mut any = false;
		for mut bytes in iter {
			// Skip leading whitespace-only entries.
			if ! any {
				if bytes.is_empty() || bytes.iter().all(u8::is_ascii_whitespace) {
					continue;
				}

				any = true;
			}

			let idx = u16::try_from(self.args.len())
				.map_err(|_| ArgyleError::TooManyArgs)?;

			// Find out what we've got!
			match KeyKind::from(&bytes[..]) {
				// Passthrough.
				KeyKind::None => {
					self.args.push(Cow::Owned(bytes));
				},
				// Record the key and passthrough.
				KeyKind::Short => {
					if bytes[1] == b'V' { self.flags |= FLAG_HAS_VERSION; }
					else if bytes[1] == b'h' { self.flags |= FLAG_HAS_HELP; }

					self.args.push(Cow::Owned(bytes));
					self.insert_key(idx)?;
					self.last.set(idx);
				},
				// Record the key and passthrough.
				KeyKind::Long => {
					if bytes == b"--version" { self.flags |= FLAG_HAS_VERSION; }
					else if bytes == b"--help" { self.flags |= FLAG_HAS_HELP; }

					self.args.push(Cow::Owned(bytes));
					self.insert_key(idx)?;
					self.last.set(idx);
				},
				// Split a short key/value pair.
				KeyKind::ShortV => {
					let v2 = bytes.split_off(2);
					self.args.push(Cow::Owned(bytes));
					self.args.push(Cow::Owned(v2));

					self.insert_key(idx)?;
					self.last.set(idx.checked_add(1).ok_or(ArgyleError::TooManyArgs)?);
				},
				// Split a long key/value pair.
				KeyKind::LongV(x) => {
					let end: usize = x.get() as usize;

					if end + 1 < bytes.len() {
						let v2 = bytes.split_off(end + 1);
						bytes.truncate(end);
						self.args.push(Cow::Owned(bytes));
						self.args.push(Cow::Owned(v2));
					}
					else {
						bytes.truncate(end);
						self.args.push(Cow::Owned(bytes));
						self.args.push(Cow::Owned(Vec::new()));
					}

					self.insert_key(idx)?;
					self.last.set(idx.checked_add(1).ok_or(ArgyleError::TooManyArgs)?);
				},
			}
		}
		Ok(())
	}
}
