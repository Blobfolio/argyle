/*!
# Argyle: Utility Methods.
*/



#[doc(hidden)]
#[allow(clippy::suspicious_else_formatting)] // It is what it is.
/// # Escape Arg String.
///
/// This is a very *crude* reverse argument parser that 'quotes' values needing
/// quoting, used by [`Argue`](crate::Argue) to recombine the arguments
/// following an "--" entry when [`FLAG_SEPARATOR`](crate::FLAG_SEPARATOR) is set.
///
/// This method is optimized for speed rather than robustness — hence its
/// crudeness — so should probably not be used in other contexts unless you're
/// expecting fairly straight-forward data.
///
/// The following adjustments are made:
///
/// * Backslashes are converted to forward slashes.
/// * Single quotes are escaped with a backslash.
/// * If the string is empty or contains anything other than `-`, `_`, `=`, `+`, `/`, `,`, `.`, `a-z`, or `0-9`, the value is wrapped in single quotes.
pub fn esc_arg_b(v: &mut Vec<u8>) {
	let mut needs_quote: bool = v.is_empty();

	if ! needs_quote {
		let mut idx: usize = 0;
		let mut len: usize = v.len();
		while idx < len {
			// Replace backslashes with forward slashes.
			if v[idx] ==  b'\\' {
				v[idx] = b'/';
				idx += 1;
			}
			// Backslash quotes.
			else if v[idx] == b'\'' {
				v.insert(idx, b'\\');
				idx += 2;
				len += 1;
				needs_quote = true;
			}
			// Something else?
			else {
				if
					! needs_quote &&
					! matches!(v[idx], b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'-' | b'_' | b'=' | b'/' | b',' | b'.' | b'+')
				{
					needs_quote = true;
				}
				idx += 1;
			}
		}
	}

	if needs_quote {
		v.reserve(2);
		v.insert(0, b'\'');
		v.push(b'\'');
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_esc_arg_b() {
		for &(src, expected) in &[
			(&b""[..], &b"''"[..]),
			(&b" "[..], &b"' '"[..]),
			(&b"Hello"[..], &b"Hello"[..]),
			(&b"Hello World"[..], &b"'Hello World'"[..]),
			(&br"\path\to\file"[..], &b"/path/to/file"[..]),
			(&b"Eat at Joe's"[..], &br"'Eat at Joe\'s'"[..]),
			("Björk's Vespertine".as_bytes(), r"'Björk\'s Vespertine'".as_bytes()),
		] {
			let mut v = src.to_vec();
			esc_arg_b(&mut v);
			assert_eq!(v, expected);
		}
	}
}
