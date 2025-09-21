/*!
# Argyle

[![docs.rs](https://img.shields.io/docsrs/argyle.svg?style=flat-square&label=docs.rs)](https://docs.rs/argyle/)
[![changelog](https://img.shields.io/crates/v/argyle.svg?style=flat-square&label=changelog&color=9b59b6)](https://github.com/Blobfolio/argyle/blob/master/CHANGELOG.md)<br>
[![crates.io](https://img.shields.io/crates/v/argyle.svg?style=flat-square&label=crates.io)](https://crates.io/crates/argyle)
[![ci](https://img.shields.io/github/actions/workflow/status/Blobfolio/argyle/ci.yaml?style=flat-square&label=ci)](https://github.com/Blobfolio/argyle/actions)
[![deps.rs](https://deps.rs/crate/argyle/latest/status.svg?style=flat-square&label=deps.rs)](https://deps.rs/crate/argyle/)<br>
[![license](https://img.shields.io/badge/license-wtfpl-ff1493?style=flat-square)](https://en.wikipedia.org/wiki/WTFPL)
[![contributions welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square&label=contributions)](https://github.com/Blobfolio/argyle/issues)

This crate provides an [`argue`] macro for generating a CLI argument enum and parsing iterator, ideal for use cases requiring more than the standard library's barebones [`args_os`](std::env::args_os) helper, but less than full-service options like [clap](https://crates.io/crates/clap).

## Example

The [docs](argue) contain a lot of addition details, but here's a minimal preview to give you an idea of what it's all about.

```
use argyle::argue;

// Construct the enum and iterator.
argue! {
    Help    "-h" "--help",
    Version "-V" "--version",
    Stderr       "--stderr",

    @options
    Format       "--format",
    Level   "-l" "--level",
}

/// # Main.
fn main() {
# use std::path::PathBuf;
    // Example settings.
    let mut stderr = false;
    let mut format: Option<Format> = None;
    let mut level = 0_u8;
    let mut paths: Vec<PathBuf> = Vec::new();

    // Loop through the environmental arguments, taking whatever actions
    // make sense for your application.
    for arg in Argument::args_os() {
        match arg {
            // You named these!
            Argument::Help => print_help(),
            Argument::Version => print_version(),
            Argument::Stderr => { stderr = true; },

            // Options come with the value as a String.
            Argument::Format(v) => {
                format = Format::from_str(v);
            },
            Argument::Level(v) => {
                level = v.parse().unwrap_or(0);
            },

            // Unmatched String values map to the first generic catchall.
            Argument::Other(v) => {
                eprintln!("Warning: unrecognized CLI argument {v}.");
            },

            // Unmatched values with invalid UTF-8 will be passed through
            // to the second generic catchall as OsString values.
            Argument::OtherOs(v) => {
                eprintln!(
                    "Warning: unrecognized CLI argument {}.",
                    v.display(),
                );
            },
        }
    }

    // Now that the settings have been worked out, do something!
    // …
}
# fn print_help() {}
# fn print_version() {}
# enum Format { Plain, Json }
# impl Format {
#     fn from_str(str: String) -> Option<Self> { None }
# }
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
#![expect(deprecated, reason = "They're *our* deprecations. Haha.")]

#![cfg_attr(docsrs, feature(doc_cfg))]



mod argue;
mod flag;
mod key;
#[macro_use] mod macros;

pub use flag::FlagsBuilder;

pub use argue::{
	args,
	Argue,
	ArgueEnv,
	Argument,
};
pub use key::{
	KeyWord,
	KeyWordsBuilder,
};
