# Argyle

[![Documentation](https://docs.rs/argyle/badge.svg)](https://docs.rs/argyle/)
[![crates.io](https://img.shields.io/crates/v/argyle.svg)](https://crates.io/crates/argyle)
[![Build Status](https://github.com/Blobfolio/argyle/workflows/Build/badge.svg)](https://github.com/Blobfolio/argyle/actions)
[![Dependency Status](https://deps.rs/repo/github/blobfolio/argyle/status.svg)](https://deps.rs/repo/github/blobfolio/argyle)

This crate contains an agnostic CLI argument parser called [`Argue`]. Unlike more robust libraries like [clap](https://crates.io/crates/clap), [`Argue`] does not hold information about expected or required arguments; it merely parses the raw arguments into a consistent state so the implementor can query them as needed.

Post-processing is an exercise largely left to the implementing library to do in its own way, in its own time. [`Argue`] exposes several methods for quickly querying the individual pieces of the set, but it can also be dereferenced to a slice or consumed into an owned vector for fully manual processing if desired.

Arguments are processed and held as bytes — `Cow<'static, [u8]>` — rather than (os)strings, again leaving the choice of later conversion entirely up to the implementor. For non-Musl Linux systems, this is almost entirely non-allocating as CLI arguments map directly back to the `CStr` pointers. For other systems, [`Argue`] falls back to [`std::env::args_os`], so requires a bit more allocation.

For simple applications, this agnostic approach can significantly reduce the overhead of processing CLI arguments, but because handling is left to the implementing library, it might be too tedious or limiting for more complex use cases.



## Installation

Add `argyle` to your `dependencies` in `Cargo.toml`, like:

```
[dependencies]
argyle = "0.5.*"
```



## Example

A general setup might look something like the following. Refer to the documentation for [`Argue`] for more information, caveats, etc.

```rust
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



## License

See also: [CREDITS.md](CREDITS.md)

Copyright © 2022 [Blobfolio, LLC](https://blobfolio.com) &lt;hello@blobfolio.com&gt;

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
