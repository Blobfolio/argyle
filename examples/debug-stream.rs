/*!
# Argyle: Streaming Argue

This example parses any arbitrary arguments fed to it and displays the results.
*/

fn main() {
	for arg in argyle::stream::args() {
		println!("\x1b[2m-----\x1b[0m\n{arg:?}");
	}
	println!("\x1b[2m-----\x1b[0m");
}
