/*!
# Argyle

[![docs.rs](https://img.shields.io/docsrs/argyle.svg?style=flat-square&label=docs.rs)](https://docs.rs/argyle/)
[![changelog](https://img.shields.io/crates/v/argyle.svg?style=flat-square&label=changelog&color=9b59b6)](https://github.com/Blobfolio/argyle/blob/master/CHANGELOG.md)<br>
[![crates.io](https://img.shields.io/crates/v/argyle.svg?style=flat-square&label=crates.io)](https://crates.io/crates/argyle)
[![ci](https://img.shields.io/github/actions/workflow/status/Blobfolio/argyle/ci.yaml?style=flat-square&label=ci)](https://github.com/Blobfolio/argyle/actions)
[![deps.rs](https://deps.rs/repo/github/blobfolio/argyle/status.svg?style=flat-square&label=deps.rs)](https://deps.rs/repo/github/blobfolio/argyle)<br>
[![license](https://img.shields.io/badge/license-wtfpl-ff1493?style=flat-square)](https://en.wikipedia.org/wiki/WTFPL)
[![contributions welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square&label=contributions)](https://github.com/Blobfolio/argyle/issues)

This crate provides a simple streaming CLI argument parser/iterator called [`Argue`](crate::stream::Argue), offering a middle ground between the standard library's barebones [`std::env::args_os`] helper and full-service crates like [clap](https://crates.io/crates/clap).

[`Argue`](crate::stream::Argue) performs some basic normalization — it handles string conversion in a non-panicking way, recognizes shorthand value assignments like `-kval`, `-k=val`, `--key=val`, and handles end-of-command (`--`) arguments — and will help identify any special subcommands and/or keys/values expected by your app.

The subsequent validation and handling, however, are left _entirely up to you_. Loop, match, and proceed however you see fit.

If that sounds terrible, just use [clap](https://crates.io/crates/clap) instead. Haha.



## Example

A general setup might look something like the following.

Refer to the documentation for [`Argue`](crate::stream::Argue) and [`Argumuent`](crate::stream::Argument) for more information, caveats, etc.

```
use argyle::stream::Argument;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
/// # Configuration.
struct Settings {
    threads: usize,
    verbose: bool,
    paths: Vec<PathBuf>,
}

let args = argyle::stream::args()
    .with_keys([
        ("-h", false),        // Boolean flag.
        ("--help", false),    // Boolean flag.
        ("--threads", true),  // Expects a value.
        ("--verbose", false), // Boolean flag.
    ])
    .unwrap(); // An error will only occur if a
               // duplicate or invalid key is declared.

// Loop and handle!
let mut settings = Settings::default();
for arg in args {
    match arg {
        Argument::Key("-h" | "--help") => {
            println!("Help Screen Goes Here.");
            return;
        },
        Argument::Key("--verbose") => {
            settings.verbose = true;
        },
        Argument::KeyWithValue("--threads", threads) => {
            settings.threads = threads.parse().expect("Threads must be a number!");
        },
        // Something else… maybe you want to assume it's a path?
        Argument::Other(v) => {
            settings.paths.push(PathBuf::from(v));
        },
        // Also something else, but not String-able. Paths don't care,
        // though, so for this example maybe you just keep it?
        Argument::InvalidUtf8(v) => {
            settings.paths.push(PathBuf::from(v));
        },
        _ => {}, // Not relevant here.
    }
}

// Do something with those settings…

```
*/

#![forbid(unsafe_code)]

#![deny(
	clippy::allow_attributes_without_reason,
	clippy::correctness,
	unreachable_pub,
)]

#![warn(
	clippy::complexity,
	clippy::nursery,
	clippy::pedantic,
	clippy::perf,
	clippy::style,

	clippy::allow_attributes,
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

#![expect(clippy::module_name_repetitions, reason = "Repetition is preferred.")]
#![expect(deprecated, reason = "The deprecated parts aren't gone yet.")]

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
