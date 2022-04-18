/*!
# Benchmark: `argyle::Argue`

For best results, this should be called with additional runtime arguments, like:
cargo bench --bench fm_argue -- --one val -tVal -k -v --apples
*/

use brunch::{
	Bench,
	benches,
};
use argyle::Argue;
use std::time::Duration;

benches!(
	Bench::new("argyle::Argue", "new(0)")
		.timed(Duration::from_secs(1))
		.with(|| Argue::new(0)),
);
