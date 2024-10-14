/*!
# Argyle

[![docs.rs](https://img.shields.io/docsrs/argyle.svg?style=flat-square&label=docs.rs)](https://docs.rs/argyle/)
[![changelog](https://img.shields.io/crates/v/argyle.svg?style=flat-square&label=changelog&color=9b59b6)](https://github.com/Blobfolio/argyle/blob/master/CHANGELOG.md)<br>
[![crates.io](https://img.shields.io/crates/v/argyle.svg?style=flat-square&label=crates.io)](https://crates.io/crates/argyle)
[![ci](https://img.shields.io/github/actions/workflow/status/Blobfolio/argyle/ci.yaml?style=flat-square&label=ci)](https://github.com/Blobfolio/argyle/actions)
[![deps.rs](https://deps.rs/repo/github/blobfolio/argyle/status.svg?style=flat-square&label=deps.rs)](https://deps.rs/repo/github/blobfolio/argyle)<br>
[![license](https://img.shields.io/badge/license-wtfpl-ff1493?style=flat-square)](https://en.wikipedia.org/wiki/WTFPL)
[![contributions welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square&label=contributions)](https://github.com/Blobfolio/argyle/issues)

This crate contains an agnostic CLI argument parser for Unix platforms called [`Argue`]. Unlike more robust libraries like [clap](https://crates.io/crates/clap), [`Argue`] does not hold information about expected or required arguments; it merely parses the raw arguments ([`std::env::args_os`]) into a consistent state so the implementor can query them as needed.

Post-processing is an exercise largely left to the implementing library to do in its own way, in its own time. [`Argue`] exposes several methods for quickly querying the individual pieces of the set, but it can also be dereferenced to a slice or consumed into an owned vector for fully manual processing if desired.

Arguments are processed and held as bytes rather than (os)strings, again leaving the choice of later conversion entirely up to the implementor.

For simple applications, this agnostic approach can significantly reduce the overhead of processing CLI arguments, but because handling is left to the implementing library, it might be too tedious or limiting for more complex use cases.



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
            ArgyleError::WantsVersion => {
                println!("MyApp v{}", env!("CARGO_PKG_VERSION"));
            },
            // A "-h" or "--help" flag was present.
            ArgyleError::WantsHelp => {
                println!("Help stuff goes here...");
            },
            // An actual error!
            e => {
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

    Ok(())
}
```
*/

#![forbid(unsafe_code)]

#![deny(
	// TODO: clippy::allow_attributes_without_reason,
	clippy::correctness,
	unreachable_pub,
)]

#![warn(
	clippy::complexity,
	clippy::nursery,
	clippy::pedantic,
	clippy::perf,
	clippy::style,

	// TODO: clippy::allow_attributes,
	clippy::clone_on_ref_ptr,
	clippy::create_dir,
	clippy::filetype_is_file,
	clippy::format_push_string,
	clippy::get_unwrap,
	clippy::impl_trait_in_params,
	clippy::lossy_float_literal,
	clippy::missing_assert_message,
	clippy::missing_docs_in_private_items,
	clippy::needless_raw_strings,
	clippy::panic_in_result_fn,
	clippy::pub_without_shorthand,
	clippy::rest_pat_in_fully_bound_structs,
	clippy::semicolon_inside_block,
	clippy::str_to_string,
	clippy::string_to_string,
	clippy::todo,
	clippy::undocumented_unsafe_blocks,
	clippy::unneeded_field_pattern,
	clippy::unseparated_literal_suffix,
	clippy::unwrap_in_result,

	macro_use_extern_crate,
	missing_copy_implementations,
	missing_docs,
	non_ascii_idents,
	trivial_casts,
	trivial_numeric_casts,
	unused_crate_dependencies,
	unused_extern_crates,
	unused_import_braces,
)]

#![allow(clippy::module_name_repetitions)] // Repetition is preferred.

#![cfg_attr(docsrs, feature(doc_cfg))]



mod argue;
mod error;
mod iter;
mod keykind;
pub mod stream;

pub use argue::{
	Argue,
	FLAG_HELP,
	FLAG_REQUIRED,
	FLAG_SUBCOMMAND,
	FLAG_VERSION,
};

#[cfg(feature = "dynamic-help")]
#[cfg_attr(docsrs, doc(cfg(feature = "dynamic-help")))]
pub use argue::FLAG_DYNAMIC_HELP;

pub use error::ArgyleError;
pub use iter::{
	ArgsOsStr,
	Options,
	OptionsOsStr,
};
pub use keykind::KeyKind;
