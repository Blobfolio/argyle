/*!
# Argyle: Key Kind

**Note:** This is not intended for external use and is subject to change.
*/

use std::num::NonZeroU16;



#[doc(hidden)]
#[derive(Debug, Clone, Copy, Hash, PartialEq)]
/// The `KeyKind` enum is used to differentiate between the types of CLI argument
/// keys [`Argue`](crate::Argue) might encounter during parsing (and `None` in the case of a
/// non-key-looking entry).
///
/// In keeping with the general ethos of this crate, speed is the name of the game,
/// which is achieved primarily through simplicity:
/// * If an entry begins with a single `-` and an ASCII letter, it is assumed to be a short key.
/// * If a short key consists of more than two characters, `2..` is assumed to be a value.
/// * If an entry begins with two `--` and an ASCII letter, it is assumed to be a long key.
/// * If a long key contains an `=`, everything after that is assumed to be a value.
pub enum KeyKind {
	/// Not a key.
	None,
	/// A short key.
	Short,
	/// A short key with a value.
	ShortV,
	/// A long key.
	Long,
	/// A long key with a value. The number indicates the position of the `=`
	/// character. Everything before is the key; everything after the value.
	LongV(NonZeroU16),
}

impl Default for KeyKind {
	#[inline]
	fn default() -> Self { Self::None }
}

impl From<&[u8]> for KeyKind {
	fn from(txt: &[u8]) -> Self {
		let len: usize = txt.len();
		if len >= 2 && txt[0] == b'-' {
			// Could be long.
			if txt[1] == b'-' {
				// Is a long.
				if len > 2 && txt[2].is_ascii_alphabetic() {
					return txt.iter()
						.position(|&x| x == b'=')
						.map_or(
							Self::Long, |x| u16::try_from(x)
								// Safety: Argue verifies the length is less
								// than u16::MAX, and this method verifies
								// non-empty.
								.map_or(Self::Long, |x| Self::LongV(unsafe {
									NonZeroU16::new_unchecked(x)
								}))
						);
				}
			}
			// Is short.
			else if txt[1].is_ascii_alphabetic() {
				if len == 2 { return Self::Short; }
				return Self::ShortV;
			}
		}

		Self::None
	}
}



#[cfg(test)]
mod tests {
	use super::*;
	use brunch as _;

	#[test]
	fn t_from() {
		assert_eq!(KeyKind::from(&b"Your Mom"[..]), KeyKind::None);
		assert_eq!(KeyKind::from(&b"--"[..]), KeyKind::None);
		assert_eq!(KeyKind::from(&b"-"[..]), KeyKind::None);
		assert_eq!(KeyKind::from(&b"-0"[..]), KeyKind::None);
		assert_eq!(KeyKind::from(&b"-y"[..]), KeyKind::Short);
		assert_eq!(KeyKind::from(&b"-yp"[..]), KeyKind::ShortV);
		assert_eq!(KeyKind::from(&b"--0"[..]), KeyKind::None);
		assert_eq!(KeyKind::from(&b"--yes"[..]), KeyKind::Long);
		assert_eq!(KeyKind::from(&b"--y-p"[..]), KeyKind::Long);
		assert_eq!(KeyKind::from(&b"--yes=no"[..]), KeyKind::LongV(NonZeroU16::new(5).unwrap()));
		assert_eq!(KeyKind::from(&b"--yes="[..]), KeyKind::LongV(NonZeroU16::new(5).unwrap()));

		// Test in and around the 16-char boundary.
		assert_eq!(KeyKind::from(&b"--yes_="[..]), KeyKind::LongV(NonZeroU16::new(6).unwrap()));
		assert_eq!(KeyKind::from(&b"--yes__="[..]), KeyKind::LongV(NonZeroU16::new(7).unwrap()));
		assert_eq!(KeyKind::from(&b"--yes___="[..]), KeyKind::LongV(NonZeroU16::new(8).unwrap()));
		assert_eq!(KeyKind::from(&b"--yes____="[..]), KeyKind::LongV(NonZeroU16::new(9).unwrap()));
		assert_eq!(KeyKind::from(&b"--yes_____="[..]), KeyKind::LongV(NonZeroU16::new(10).unwrap()));
		assert_eq!(KeyKind::from(&b"--yes______="[..]), KeyKind::LongV(NonZeroU16::new(11).unwrap()));
		assert_eq!(KeyKind::from(&b"--yes_______="[..]), KeyKind::LongV(NonZeroU16::new(12).unwrap()));
		assert_eq!(KeyKind::from(&b"--yes________="[..]), KeyKind::LongV(NonZeroU16::new(13).unwrap()));
		assert_eq!(KeyKind::from(&b"--yes_________="[..]), KeyKind::LongV(NonZeroU16::new(14).unwrap()));
		assert_eq!(KeyKind::from(&b"--yes__________="[..]), KeyKind::LongV(NonZeroU16::new(15).unwrap()));
		assert_eq!(KeyKind::from(&b"--yes___________="[..]), KeyKind::LongV(NonZeroU16::new(16).unwrap()));
		assert_eq!(KeyKind::from(&b"--yes____________="[..]), KeyKind::LongV(NonZeroU16::new(17).unwrap()));
		assert_eq!(KeyKind::from(&b"--yes____________-="[..]), KeyKind::LongV(NonZeroU16::new(18).unwrap()));
		assert_eq!(KeyKind::from(&b"--yes_____________-="[..]), KeyKind::LongV(NonZeroU16::new(19).unwrap()));
		assert_eq!(KeyKind::from(&b"--yes______________-="[..]), KeyKind::LongV(NonZeroU16::new(20).unwrap()));
		assert_eq!(KeyKind::from(&b"--yes_____________"[..]), KeyKind::Long);

		// Does this work?
		assert_eq!(
			KeyKind::from("--BjörkGuðmundsdóttir".as_bytes()),
			KeyKind::Long
		);
		assert_eq!(
			KeyKind::from("--BjörkGuðmunds=dóttir".as_bytes()),
			KeyKind::LongV(NonZeroU16::new(17).unwrap())
		);
	}
}
