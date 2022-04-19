# Changelog


## [0.5.5](https://github.com/Blobfolio/argyle/releases/tag/v0.5.5) - TBD

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
