/*!
# Benchmark: `argyle::KeyKind`
*/

use brunch::{
	Bench,
	benches,
};
use argyle::KeyKind;

benches!(
	Bench::new("argyle::KeyKind", "from(Hello World)")
		.with(|| KeyKind::from(&b"Hello World"[..])),

	Bench::new("argyle::KeyKind", "from(-p)")
		.with(|| KeyKind::from(&b"-p"[..])),

	Bench::new("argyle::KeyKind", "from(--prefix)")
		.with(|| KeyKind::from(&b"--prefix"[..])),

	Bench::new("argyle::KeyKind", "from(--prefix-color=199)")
		.with(|| KeyKind::from(&b"--prefix-color=199"[..]))
);
