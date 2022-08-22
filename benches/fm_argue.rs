/*!
# Benchmark: `argyle::Argue`

For best results, this should be called with additional runtime arguments, like:
cargo bench --bench fm_argue -- --one val -tVal -k -v --apples /foo/bar /bar/baz
*/

use brunch::{
	Bench,
	benches,
};
use argyle::Argue;

fn argument() -> Argue {
	[
		&b"-k"[..],
		b"--key=val",
		b"-x",
		b"out",
		b"--quiet",
		b"/foo/bar",
		b"/bar/baz",
	].into_iter().collect()
}

benches!(
	Bench::new("argyle::Argue::new(0)")
		.run(|| Argue::new(0)),

	Bench::spacer(),

	Bench::new("argyle::Argue::option(-x)")
		.run_seeded_with(argument, |a| a.option(b"-x").is_some()),

	Bench::new("argyle::Argue::switch2(-q, --quiet)")
		.run_seeded_with(argument, |a| a.switch2(b"-q", b"--quiet")),
);
