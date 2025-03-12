# `FlagsBuilder` Demo.

This demo generates three flag sets via `build.rs`:
1. One with the minimum number of flags.
2. One with a few more, including complex and alias types.
3. One with the maximum number of flags.

To _see_ what that code looks like, simply build and run the demo:

```rust
cargo run --release
```

To run the generated unit tests, test it instead:
```rust
cargo test --release
```

It can also be linted to help rule out formatting weirdness:
```rust
cargo clippy --release
```
