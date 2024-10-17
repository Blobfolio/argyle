# Argyle

[![docs.rs](https://img.shields.io/docsrs/argyle.svg?style=flat-square&label=docs.rs)](https://docs.rs/argyle/)
[![changelog](https://img.shields.io/crates/v/argyle.svg?style=flat-square&label=changelog&color=9b59b6)](https://github.com/Blobfolio/argyle/blob/master/CHANGELOG.md)<br>
[![crates.io](https://img.shields.io/crates/v/argyle.svg?style=flat-square&label=crates.io)](https://crates.io/crates/argyle)
[![ci](https://img.shields.io/github/actions/workflow/status/Blobfolio/argyle/ci.yaml?style=flat-square&label=ci)](https://github.com/Blobfolio/argyle/actions)
[![deps.rs](https://deps.rs/repo/github/blobfolio/argyle/status.svg?style=flat-square&label=deps.rs)](https://deps.rs/repo/github/blobfolio/argyle)<br>
[![license](https://img.shields.io/badge/license-wtfpl-ff1493?style=flat-square)](https://en.wikipedia.org/wiki/WTFPL)
[![contributions welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square&label=contributions)](https://github.com/Blobfolio/argyle/issues)

This crate provides a simple streaming CLI argument parser/iterator called `Argue`, offering a middle ground between the standard library's barebones `std::env::args_os` helper and full-service crates like [clap](https://crates.io/crates/clap).

`Argue` performs some basic normalization — it handles string conversion in a non-panicking way, recognizes shorthand value assignments like `-kval`, `-k=val`, `--key=val`, and handles end-of-command (`--`) arguments — and will help identify any special keys/values expected by your app.

The subsequent validation and handling, however, are left _entirely up to you_. Loop, match, and proceed however you see fit.

If that sounds terrible, just use [clap](https://crates.io/crates/clap) instead. Haha.



## Installation

Add `argyle` to your `dependencies` in `Cargo.toml`, like:

```
[dependencies]
argyle = "0.9.*"
```



## Example

A general setup might look something like the following.

Refer to the documentation for `Argue`, `KeyWord`, and `Argument` for more information, caveats, etc.

```rust
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



## License

See also: [CREDITS.md](CREDITS.md)

Copyright © 2024 [Blobfolio, LLC](https://blobfolio.com) &lt;hello@blobfolio.com&gt;

This work is free. You can redistribute it and/or modify it under the terms of the Do What The Fuck You Want To Public License, Version 2.

    DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
    Version 2, December 2004
    
    Copyright (C) 2004 Sam Hocevar <sam@hocevar.net>
    
    Everyone is permitted to copy and distribute verbatim or modified
    copies of this license document, and changing it is allowed as long
    as the name is changed.
    
    DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
    TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION
    
    0. You just DO WHAT THE FUCK YOU WANT TO.
