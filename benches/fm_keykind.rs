/*!
# Benchmark: `argyle::KeyKind`
*/

use brunch::{
	Bench,
	benches,
};
use argyle::KeyKind;

benches!(
	Bench::new("argyle::KeyKind::from(Hello World)")
		.run(|| KeyKind::from(&b"Hello World"[..])),

	Bench::new("argyle::KeyKind::from(-p)")
		.run(|| KeyKind::from(&b"-p"[..])),

	Bench::new("argyle::KeyKind::from(--prefix)")
		.run(|| KeyKind::from(&b"--prefix"[..])),

	Bench::new("argyle::KeyKind::from(--prefix-color=199)")
		.run(|| KeyKind::from(&b"--prefix-color=199"[..]))
);
