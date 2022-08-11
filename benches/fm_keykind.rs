/*!
# Benchmark: `argyle::KeyKind`
*/

use brunch::{
	Bench,
	benches,
};
use argyle::KeyKind;
use std::time::Duration;

benches!(
	Bench::new("argyle::KeyKind", "from(Hello World)")
		.timed(Duration::from_secs(1))
		.with(|| KeyKind::from(&b"Hello World"[..])),

	Bench::new("argyle::KeyKind", "from(-p)")
		.timed(Duration::from_secs(1))
		.with(|| KeyKind::from(&b"-p"[..])),

	Bench::new("argyle::KeyKind", "from(--prefix)")
		.timed(Duration::from_secs(1))
		.with(|| KeyKind::from(&b"--prefix"[..])),

	Bench::new("argyle::KeyKind", "from(--prefix-color=199)")
		.timed(Duration::from_secs(1))
		.with(|| KeyKind::from(&b"--prefix-color=199"[..]))
);
