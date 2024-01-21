/*!
# Argyle: Argue

This example prints the trailing arguments, if any, including those from
-l/--list.
*/

fn main() {
	let args = argyle::Argue::new(argyle::FLAG_REQUIRED);
	match args {
		Ok(mut a) => {
			a = a.with_list();
			println!("\x1b[2mArguments:\x1b[0m");
			let mut any = false;
			for v in a.args_os() {
				any = true;
				println!("  {v:?}");
			}
			if ! any {
				println!("  \x1b[91mNo Arguments Passed\x1b[0m");
				std::process::exit(1);
			}

			println!("");
		},
		Err(e) => {
			println!("\x1b[1;91mError:\x1b[0m {e}");
			std::process::exit(1);
		},
	}
}
