/*!
# Argyle

[![docs.rs](https://img.shields.io/docsrs/argyle.svg?style=flat-square&label=docs.rs)](https://docs.rs/argyle/)
[![changelog](https://img.shields.io/crates/v/argyle.svg?style=flat-square&label=changelog&color=9b59b6)](https://github.com/Blobfolio/argyle/blob/master/CHANGELOG.md)<br>
[![crates.io](https://img.shields.io/crates/v/argyle.svg?style=flat-square&label=crates.io)](https://crates.io/crates/argyle)
[![ci](https://img.shields.io/github/actions/workflow/status/Blobfolio/argyle/ci.yaml?style=flat-square&label=ci)](https://github.com/Blobfolio/argyle/actions)
[![deps.rs](https://deps.rs/crate/argyle/latest/status.svg?style=flat-square&label=deps.rs)](https://deps.rs/crate/argyle/)<br>
[![license](https://img.shields.io/badge/license-wtfpl-ff1493?style=flat-square)](https://en.wikipedia.org/wiki/WTFPL)
[![contributions welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square&label=contributions)](https://github.com/Blobfolio/argyle/issues)

This crate provides a simple streaming CLI argument parser/iterator called [`Argue`], offering a middle ground between the standard library's barebones [`std::env::args_os`] helper and full-service crates like [clap](https://crates.io/crates/clap).

[`Argue`] performs some basic normalization — it handles string conversion in a non-panicking way, recognizes shorthand value assignments like `-kval`, `-k=val`, `--key=val`, and handles end-of-command (`--`) arguments — and will help identify any special  keys/values expected by your app.

The subsequent validation and handling, however, are left _entirely up to you_. Loop, match, and proceed however you see fit.

If that sounds terrible, just use [clap](https://crates.io/crates/clap) instead. Haha.



## Crate Features

The non-default **`try_paths`** feature can be enabled to expose an additional `Argument::Path` variant, used for unassociated-and-unrecognized values for which `std::fs::exists() == Ok(true)`.



## Example

A general setup might look something like the following.

Refer to the documentation for [`Argue`], [`KeyWord`], and [`Argument`] for more information, caveats, etc.

```
use argyle::{Argument, KeyWord};
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
/// # Configuration.
struct Settings {
    threads: usize,
    verbose: bool,
    paths: Vec<PathBuf>,
}

let args = argyle::args()
    .with_keywords([
        KeyWord::key("-h").unwrap(),            // Boolean flag (short).
        KeyWord::key("--help").unwrap(),        // Boolean flag (long).
        KeyWord::key_with_value("-j").unwrap(), // Expects a value.
        KeyWord::key_with_value("--threads").unwrap(),
    ]);

// Loop and handle!
let mut settings = Settings::default();
for arg in args {
    match arg {
        // Help flag match.
        Argument::Key("-h" | "--help") => {
            println!("Help Screen Goes Here.");
            return;
        },

        // Thread option match.
        Argument::KeyWithValue("-j" | "--threads", value) => {
            settings.threads = value.parse()
                .expect("Maximum threads must be a number!");
        },

        // Something else.
        Argument::Other(v) => {
            settings.paths.push(PathBuf::from(v));
        },

        // Also something else, but not String-able. PathBuf doesn't care
        // about UTF-8, though, so it might be fine!
        Argument::InvalidUtf8(v) => {
            settings.paths.push(PathBuf::from(v));
        },

        // Nothing else is relevant here.
        _ => {},
    }
}

// Now that you're set up, do stuff…
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

#![expect(clippy::unnecessary_debug_formatting, reason = "Clippy can't know this.")]

#![cfg_attr(docsrs, feature(doc_cfg))]



mod argue;
mod flag;
mod key;

pub use argue::{
	args,
	Argue,
	ArgueEnv,
	Argument,
};
pub use flag::FlagsBuilder;
pub use key::{
	KeyWord,
	KeyWordsBuilder,
};
