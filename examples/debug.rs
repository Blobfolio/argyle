/*!
# Argyle: Argue

This example parses any arbitrary arguments fed to it and displays the results.
*/

use std::ffi::OsStr;
use std::mem::size_of;
use std::os::unix::ffi::OsStrExt;



fn main() {
	println!("Struct size: {}", size_of::<argyle::Argue>());
	println!("");

	let args = argyle::Argue::new(argyle::FLAG_REQUIRED);
	match args {
		Ok(a) => {
			println!("\x1b[2mRAW:\x1b[0m");
			println!("{:?}", a);

			println!("");
			println!("\x1b[2mPRETTY:\x1b[0m");

			a.take().iter().for_each(|b| {
				println!("{}", OsStr::from_bytes(b).to_str().unwrap_or("[Invalid UTF-8]"));
			});

			println!("");
		},
		Err(e) => {
			println!("Error: {}", e);
		},
	}
}
