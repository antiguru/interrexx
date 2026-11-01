# interrexx

A Rust implementation of the ooRexx language. Behaviour is defined differentially: output must match the C++ ooRexx interpreter byte for byte, apart from the deviations in `docs/superpowers/plans/phase-4-exclusions.txt`.

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
cargo test --workspace
```

## License

MIT OR Apache-2.0. See `NOTICE` for the third-party code that built binaries contain.
