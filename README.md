# interrexx

A Rust implementation of the ooRexx language. Behaviour is defined differentially: output must match the C++ ooRexx interpreter byte for byte, apart from the deviations in `spec/src/deviations.md`.

It keeps ooRexx's dynamic object model and replaces the execution model: a register IR, stackless activities under a scheduler, and value representations a program cannot observe.

This is an independent project, not affiliated with the Rexx Language Association or the ooRexx project.

For installation, command examples and differences to check when moving from
ooRexx, see the [user guide](https://github.com/antiguru/interrexx/blob/main/docs/user-guide.md).

## Building

```sh
git clone https://github.com/antiguru/interrexx
cd interrexx/rust
cargo build --release
```

The build reads only files of this repository, so it needs no submodule. Binaries are `rexx` and `rexxc`.

### Debian package

[cargo-deb](https://github.com/kornelski/cargo-deb) builds a `.deb` from the same tree:

```sh
cargo install cargo-deb --locked
cd interrexx/rust
cargo deb -p interrexx
sudo apt install ./target/debian/interrexx_*.deb
```

The package installs `rexx`, `rexxc`, `rexx-ir`, `rxapi`, `rxqueue` and `rxsubcom` into `/usr/bin` and `librexx.so` and `librexxapi.so`, with their `.so.4` SONAME links, into `/usr/lib`. Debug symbols go into a separate `interrexx-dbgsym_*.ddeb` package beside it. It replaces an installed `oorexx` package, whose commands and library names it shares. The `[package.metadata.deb]` table in `rust/crates/interrexx/Cargo.toml` lists the contents.

## Testing

The regular tests need no ooRexx build and no submodule. They need the ooTest and ooRexx
documentation checkouts, `svn` (the extractor tests read a checkout's revision) and a C and a C++ compiler (the ABI dump and the link test
compile against `include/`). The preflight script lists the checkouts that are missing and prints
the commands that fetch them:

```sh
python3 rust/scripts/test_preflight.py --standalone
cd rust
cargo nextest run --workspace --profile standalone
cargo test --workspace --doc
```

The upstream tests and the comparisons against an ooRexx build need the pinned `third_party/ooRexx`
submodule and its build, which `scripts/setup-third-party.sh` makes. They run in the `ci` profile
together with the regular tests:

```sh
scripts/setup-third-party.sh
cd rust
cargo nextest run --workspace --profile ci
cargo test --workspace --doc
```

Install nextest with `cargo install cargo-nextest --locked` or from a prebuilt binary
(<https://nexte.st>). Run the regular tests through nextest with the `standalone` profile:
`cargo test --workspace` has no filter, so on a checkout without the submodule it also runs the
upstream tier, which fails.

`docs/developing.md` covers running the tests and both interpreters safely; `docs/design/`
describes the implementation.

## License

MIT OR Apache-2.0. See `NOTICE` for the third-party code that built binaries contain.

CI checks both sides of this. `scripts/check-licence-text.sh` fails when a tracked file names the
ooRexx licence (`NOTICE` excepted until #47 lands), and `cargo deny check licenses`, run from
`rust/`, fails when a dependency's licence is not one that `rust/deny.toml` allows.
