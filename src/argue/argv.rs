/*!
# Argyle: ARGV

This module contains a non-allocating version of [`std::env::args_os`] for
non-musl Linux systems inspired by [`argv`](https://crates.io/crates/argv).
*/

#![allow(clippy::similar_names)] // Follow convention.

use std::{
	ffi::CStr,
	os::raw::{
		c_char,
		c_int,
	},
};

static mut ARGC: c_int = 0;
static mut ARGV: *const *const c_char = std::ptr::null();

#[cfg(target_os = "linux")]
#[link_section = ".init_array"]
#[used]
static CAPTURE: unsafe extern "C" fn(c_int, *const *const c_char) = capture;

#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
#[allow(dead_code)]
unsafe extern "C" fn capture(argc: c_int, argv: *const *const c_char) {
	ARGC = argc;
	ARGV = argv;
}

/// # Raw Arguments.
///
/// This will skip the first (path) argument and return a pointer and
/// length if there's anything worth returning.
///
/// The actual iterables are byte slices in this case, rather than
/// (os)strings.
pub(super) struct Args {
	next: *const *const c_char,
	end: *const *const c_char,
}

impl Default for Args {
	#[allow(clippy::cast_sign_loss)] // ARGC is non-negative.
	/// # Raw Arguments.
	///
	/// ## Safety
	///
	/// This accesses mutable statics — `ARGC` and `ARGV` — but because
	/// they are only mutated prior to the execution of `main()`, it's
	/// A-OK.
	///
	/// Also worth noting, the operating system is responsible for ensuring
	/// `ARGV + ARGC` does not overflow, so no worries there either.
	fn default() -> Self {
		// We'll only return arguments if there are at least 2 of them.
		let len: usize = unsafe { ARGC } as usize;
		if len > 1 {
			Self {
				next: unsafe { ARGV.add(1) },
				end: unsafe { ARGV.add(len) },
			}
		}
		else {
			let end = unsafe { ARGV.add(len) };
			Self {
				next: end,
				end
			}
		}
	}
}

impl Iterator for Args {
	type Item = &'static [u8];

	fn next(&mut self) -> Option<Self::Item> {
		if self.next >= self.end { None }
		else {
			let out = unsafe { CStr::from_ptr(*self.next).to_bytes() };
			// Short circuit.
			if out == b"--" {
				self.next = self.end;
				None
			}
			else {
				self.next = unsafe { self.next.add(1) };
				Some(out)
			}
		}
	}

	#[allow(clippy::cast_sign_loss)] // Distance is always >= 0.
	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		let len = unsafe { self.end.offset_from(self.next) as usize };
		(len, Some(len))
	}
}
