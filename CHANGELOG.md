# Changelog



## [0.6.6](https://github.com/Blobfolio/argyle/releases/tag/v0.6.6) - 2023-02-04

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
