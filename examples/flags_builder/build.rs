/*!
# `FlagsBuilder` Build Tests.

This generates several different flag sets so that their unit tests/formatting
can be properly tested.
*/

use argyle::FlagsBuilder;
use std::{
	fs::File,
	io::Write,
	path::PathBuf,
};

fn main() {
	let out = [
		build1(),
		build2(),
		build3(),
	];

	let out = out.join("\n\n\n");

	File::create(out_path("flags.rs"))
		.and_then(|mut f| f.write_all(out.as_bytes()).and_then(|()| f.flush()))
		.expect("Unable to save flags.rs.");
}

/// # Single Flag.
fn build1() -> String {
	FlagsBuilder::new("Flags1")
		.with_docs("# One Flag.\n\nA single named flag!")
		.public()
		.with_flag("One", Some("# The Only One."))
		.to_string()
}

/// # Complex and Aliased.
fn build2() -> String {
	FlagsBuilder::new("FruitVeg")
		.with_docs("# Fruit/Veg Flags.\n\nThese include complex and alias members.")
		.private()
		.with_flag("Apples", None)
		.with_flag("Bananas", None)
		.with_flag("Carrots", None)
		.with_complex_flag("Dirt", ["Apples", "Bananas", "Carrots"], Some("# Things In Dirt."))
		.with_alias("Fruit", ["Apples", "Bananas"], None)
		.with_default_all()
		.to_string()
}

/// # Full Flags.
fn build3() -> String {
	FlagsBuilder::new("Animals")
		.with_docs("# Animal Flags.\n\nThis uses the maximum number of slots!")
		.with_flag("Aardvark", None)
		.with_flag("Beaver", None)
		.with_flag("Cat", None)
		.with_flag("Dog", None)
		.with_flag("Eagle", None)
		.with_flag("Ferret", None)
		.with_flag("Gnome", None)
		.with_flag("Hedgehog", None)
		.with_alias("Domestic", ["Cat", "Dog"], None)
		.with_defaults(["Hedgehog"])
		.to_string()
}


/// # Output Path.
///
/// Append the sub-path to OUT_DIR and return it.
fn out_path(stub: &str) -> PathBuf {
	std::fs::canonicalize(std::env::var("OUT_DIR").expect("Missing OUT_DIR."))
		.expect("Missing OUT_DIR.")
		.join(stub)
}
