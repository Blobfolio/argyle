/*!
# Argyle: Key Kind

**Note:** This is not intended for external use and is subject to change.
*/

#[doc(hidden)]
#[deprecated(since = "0.9.0", note = "use stream::Argue instead")]
#[derive(Debug, Clone, Copy, Default, Eq, Hash, PartialEq)]
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
	#[default]
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
	LongV(usize),
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
						.map_or(Self::Long, Self::LongV);
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

	#[test]
	#[allow(clippy::cognitive_complexity)] // It is what it is.
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
		assert_eq!(KeyKind::from(&b"--yes=no"[..]), KeyKind::LongV(5));
		assert_eq!(KeyKind::from(&b"--yes="[..]), KeyKind::LongV(5));

		// Test in and around the 16-char boundary.
		assert_eq!(KeyKind::from(&b"--yes_="[..]), KeyKind::LongV(6));
		assert_eq!(KeyKind::from(&b"--yes__="[..]), KeyKind::LongV(7));
		assert_eq!(KeyKind::from(&b"--yes___="[..]), KeyKind::LongV(8));
		assert_eq!(KeyKind::from(&b"--yes____="[..]), KeyKind::LongV(9));
		assert_eq!(KeyKind::from(&b"--yes_____="[..]), KeyKind::LongV(10));
		assert_eq!(KeyKind::from(&b"--yes______="[..]), KeyKind::LongV(11));
		assert_eq!(KeyKind::from(&b"--yes_______="[..]), KeyKind::LongV(12));
		assert_eq!(KeyKind::from(&b"--yes________="[..]), KeyKind::LongV(13));
		assert_eq!(KeyKind::from(&b"--yes_________="[..]), KeyKind::LongV(14));
		assert_eq!(KeyKind::from(&b"--yes__________="[..]), KeyKind::LongV(15));
		assert_eq!(KeyKind::from(&b"--yes___________="[..]), KeyKind::LongV(16));
		assert_eq!(KeyKind::from(&b"--yes____________="[..]), KeyKind::LongV(17));
		assert_eq!(KeyKind::from(&b"--yes____________-="[..]), KeyKind::LongV(18));
		assert_eq!(KeyKind::from(&b"--yes_____________-="[..]), KeyKind::LongV(19));
		assert_eq!(KeyKind::from(&b"--yes______________-="[..]), KeyKind::LongV(20));
		assert_eq!(KeyKind::from(&b"--yes_____________"[..]), KeyKind::Long);

		// Does this work?
		assert_eq!(
			KeyKind::from("--BjörkGuðmundsdóttir".as_bytes()),
			KeyKind::Long
		);
		assert_eq!(
			KeyKind::from("--BjörkGuðmunds=dóttir".as_bytes()),
			KeyKind::LongV(17)
		);
	}
}
