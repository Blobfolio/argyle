# Changelog



## [0.11.0](https://github.com/Blobfolio/argyle/releases/tag/v0.11.0) - 2025-02-23

### Changed

* Bump Rust edition to 2024
* Bump MSRV to `1.85`



## [0.10.1](https://github.com/Blobfolio/argyle/releases/tag/v0.10.1) - 2024-11-28

### Changed

* Miscellaneous code changes and lints
* Miscellaneous doc changes



## [0.10.0](https://github.com/Blobfolio/argyle/releases/tag/v0.10.0) - 2024-10-17

This release finishes the work of the last one. The streaming version of `Argue` is now stable and all there is; the old methods and structs have been removed.

Check out the [docs](https://docs.rs/argyle/latest/argyle/) to see how it all works!



## [0.9.0](https://github.com/Blobfolio/argyle/releases/tag/v0.9.0) - 2024-10-14

This release introduces a brand new streaming version of the argument parser `Argue`. It is simpler and cleaner, but works completely differently than the original.

Sorry about that!

Old and new are both present in this release to ease the transition, but projects should migrate to the new version (or another crate) when convenient as the duality won't last.

### New

* `argyle::stream::args`
* `argyle::stream::Argue`
* `argyle::stream::Argument`
* `argyle::stream::ArgyleError`

### Changed

* Bump MSRV to `1.81`
* Update lints

### Deprecated

* `argyle::Argue`
* `argyle::ArgyleError`
* `argyle::KeyKind`



## [0.8.1](https://github.com/Blobfolio/argyle/releases/tag/v0.8.1) - 2024-09-05

### Changed

* Minor code changes and lints



## [0.8.0](https://github.com/Blobfolio/argyle/releases/tag/v0.8.0) - 2024-08-08

### New

* `Argue::take_trailing`



## [0.7.2](https://github.com/Blobfolio/argyle/releases/tag/v0.7.2) - 2024-02-15

### New

* `Argue::check_keys`



## [0.7.1](https://github.com/Blobfolio/argyle/releases/tag/v0.7.1) - 2024-02-08

### Changed

* Minor code cleanup and lints



## [0.7.0](https://github.com/Blobfolio/argyle/releases/tag/v0.7.0) - 2024-01-20

### Breaking

* Bump MSRV to `1.70`
* `Argue::with_list` will now read lines from STDIN when the path is given as `-`

### New

* `Argue::with_trailing_args`

### Changed

* `Argue::with_list` now buffers file reads (instead of reading everything in one go)



## [0.6.8](https://github.com/Blobfolio/argyle/releases/tag/v0.6.8) - 2023-06-01

### Changed

* Bump dev dependencies
* CI: test in debug and release modes
* CI: test MSRV



## [0.6.7](https://github.com/Blobfolio/argyle/releases/tag/v0.6.7) - 2023-02-07

### Changed

* Rename `Argue::option2_iter` to `Argue::option2_values`
* Rename `Argue::option2_iter_os` to `Argue::option2_values_os`



## [0.6.6](https://github.com/Blobfolio/argyle/releases/tag/v0.6.6) - 2023-02-04

### Changed

* Improve docs.rs environment detection



## [0.6.5](https://github.com/Blobfolio/argyle/releases/tag/v0.6.5) - 2023-01-26

### Changed

* Bump brunch `0.4`



## [0.6.4](https://github.com/Blobfolio/argyle/releases/tag/v0.6.4) - 2022-12-26

### New

* `Argue::option_values`
* `Argue::option_values_os`
* `Argue::option2_values`
* `Argue::option2_values_os`
* `Argue::switch_count`
* `Argue::switch2_count`

### Changed

* `Argue::option`, `Argue::option2`, etc., now return the _last_ value rather than the _first_ (in cases where the same flag is present multiple times).

### Fixed

* Updated ci badge syntax (docs).



## [0.6.3](https://github.com/Blobfolio/argyle/releases/tag/v0.6.3) - 2022-09-22

### Changed

* Improved docs
* Update (dev) dependencies



## [0.6.2](https://github.com/Blobfolio/argyle/releases/tag/v0.6.2) - 2022-08-12

### New

* `Argue::option_by_prefix_os`
* `Argue::option_by_prefix`
* `Argue::switch_by_prefix_os`
* `Argue::switch_by_prefix`



## [0.6.1](https://github.com/Blobfolio/argyle/releases/tag/v0.6.1) - 2022-08-11

### Changed

* Bump MSRV 1.62



## [0.6.0](https://github.com/Blobfolio/argyle/releases/tag/v0.6.0) - 2022-05-30

All arguments held by `Argue` are now stored as owned bytes (no more `Cow`). This will be a little slower than previous releases, but safer, as the state can now be maintained in the (unlikely) event the environment is later mutated.

The total number of keys are no longer restricted. Similarly, individual argument length restrictions have been removed. Go wild!

The different Unix implementations have all been merged together under the hood, so should now be more consistent across platforms. (This library is still _not_ compatible with Windows.)

Other changes to note:

* `Argue::peek`, `Argue::peek_unchecked` have been removed (in favor of conventional indexing; see next two lines)
* `Argue` now implements `std::ops::Index`
* New: `Argue::get`, `Argue::is_empty`, `Argue::len`
* `Argue::first_arg`, `Argue::first_arg_os` have been removed (use `Argue::arg(0)` instead)



## [0.5.6](https://github.com/Blobfolio/argyle/releases/tag/v0.5.6) - 2022-05-19

### Changed

* Improved documentation



## [0.5.5](https://github.com/Blobfolio/argyle/releases/tag/v0.5.5) - 2022-04-19

### Changed

* Improve performance of `Argue::new` by ~20%
* Trailing arguments are now explicitly capped to `u16::MAX` entries for consistency (previously only keys needed to sit within the `u16` range)

### Deprecated

* `Argue::with_flags` (set flags during `Argue::new` instead)



## [0.5.4](https://github.com/Blobfolio/argyle/releases/tag/v0.5.4) - 2022-04-14

### Changed

* Make unit tests target-agnostic
* Force `std::env::args_os` fallback for `miri`
* Miscellaneous refactoring and code cleanup
* Eliminate a few unnecessary allocations



## [0.5.3](https://github.com/Blobfolio/argyle/releases/tag/v0.5.3) - 2022-03-29

### New

* `Argue::args_os`
* `Argue::arg_os`
* `Argue::first_arg_os`
* `Argue::option_os`
* `Argue::option2_os`



## [0.5.2](https://github.com/Blobfolio/argyle/releases/tag/v0.5.2) - 2021-12-25

### New

* `Argue::bitflags`

### Changed

* Misc code cleanup.



## [0.5.1](https://github.com/Blobfolio/argyle/releases/tag/v0.5.1) - 2021-12-02

### Changed

* Docs.
* Fix justfile `credits` task.



## [0.5.0](https://github.com/Blobfolio/argyle/releases/tag/v0.5.0) - 2021-10-21

### Added

* This changelog! Haha.

### Changed

* Use Rust edition 2021.
