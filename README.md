# interrexx

A Rust implementation of the ooRexx language. Behaviour is defined differentially: output must match the C++ ooRexx interpreter byte for byte, apart from the deviations in `spec/src/deviations.md`.

It keeps ooRexx's dynamic object model and replaces the execution model: a register IR, stackless activities under a scheduler, and value representations a program cannot observe.

This is an independent project, not affiliated with the Rexx Language Association or the ooRexx project.

## Building

```sh
git clone --recurse-submodules https://github.com/antiguru/interrexx
cd interrexx/rust
cargo build --release
```

The build reads ooRexx sources from the `third_party/ooRexx` submodule. Binaries are `rexx` and `rexxc`.

## Testing

Tests compare against an ooRexx build and run ooTest:

```sh
scripts/setup-third-party.sh
cd rust
cargo nextest run --workspace --profile ci
cargo test --workspace --doc
```

Install nextest with `cargo install cargo-nextest --locked` or from a prebuilt binary
(<https://nexte.st>); `cargo test --workspace` runs the same tests, more slowly.

`docs/developing.md` covers running the tests and both interpreters safely; `docs/design/`
describes the implementation.

## License

MIT OR Apache-2.0. See `NOTICE` for the third-party code that built binaries contain.

CI checks both sides of this. `scripts/check-licence-text.sh` fails when a tracked file names the
ooRexx licence (`NOTICE` excepted until #47 lands), and `cargo deny check licenses`, run from
`rust/`, fails when a dependency's licence is not one that `rust/deny.toml` allows.
