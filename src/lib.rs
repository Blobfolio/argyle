/*!
# Argyle

[![Documentation](https://docs.rs/argyle/badge.svg)](https://docs.rs/argyle/)
[![crates.io](https://img.shields.io/crates/v/argyle.svg)](https://crates.io/crates/argyle)

This crate contains an agnostic CLI argument parser called [`Argue`]. Unlike more robust libraries like [clap](https://crates.io/crates/clap), [`Argue`] does not hold information about expected or required arguments; it merely parses the raw arguments into a consistent state so the implementor can query them as needed.

Post-processing is an exercise largely left to the implementing library to do in its own way, in its own time. [`Argue`] exposes several methods for quickly querying the individual pieces of the set, but it can also be dereferenced to a slice or consumed into an owned vector for fully manual processing if desired.

Arguments are processed and held as bytes — `Cow<'static, [u8]>` — rather than (os)strings, again leaving the choice of later conversion entirely up to the implementor. For non-Musl Linux systems, this is almost entirely non-allocating as CLI arguments map directly back to the `CStr` pointers. For other systems, [`Argue`] falls back to [`std::env::args_os`], so requires a bit more allocation.

For simple applications, this agnostic approach can significantly reduce the overhead of processing CLI arguments, but because handling is left to the implementing library, it might be too tedious or limiting for more complex use cases.



## Installation

Add `argyle` to your `dependencies` in `Cargo.toml`, like:

```
[dependencies]
argyle = "0.2.*"
```



## Example

A general setup might look something like the following. Refer to the documentation for [`Argue`] for more information, caveats, etc.

```no_run
use argyle::{
    Argue,
    ArgyleError,
    FLAG_HELP,
    FLAG_REQUIRED,
    FLAG_VERSION,
};

fn main() {
    if let Err(e) = _main() {
        match(e) {
            // A "-V" or "--version" flag was present.
            Err(ArgyleError::WantsVersion) => {
                println!("MyApp v{}", env!("CARGO_PKG_VERSION"));
            },
            // A "-h" or "--help" flag was present.
            Err(ArgyleError::WantsHelp) => {
                println!("Help stuff goes here...");
            },
            // An actual error!
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            },
        }
    }
}

fn _main() -> Result<(), ArgyleError> {
    // Parse CLI arguments.
    let args = Argue::new(FLAG_HELP | FLAG_REQUIRED | FLAG_VERSION)?;

    // Pull the pieces you want.
    let clean: bool = args.switch(b"--clean");
    let prefix: Option<&[u8]> = args.option2(b"-p", b"--prefix");

    ...
}
```
*/

#![warn(clippy::filetype_is_file)]
#![warn(clippy::integer_division)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
#![warn(clippy::suboptimal_flops)]
#![warn(clippy::unneeded_field_pattern)]
#![warn(macro_use_extern_crate)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(non_ascii_idents)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unused_crate_dependencies)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::map_err_ignore)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]



mod argue;
mod error;
mod keykind;

#[doc(hidden)]
pub mod utility;

pub use argue::{
	Argue,
	FLAG_DYNAMIC_HELP,
	FLAG_HELP,
	FLAG_REQUIRED,
	FLAG_SEPARATOR,
	FLAG_SUBCOMMAND,
	FLAG_VERSION,
};
pub use error::ArgyleError;
pub use keykind::KeyKind;
