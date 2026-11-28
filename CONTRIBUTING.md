# Contributing to interrexx

Read [Developing interrexx](docs/developing.md) for setup and
[Decisions](docs/design/decisions.md) before proposing a design change. The
[language specification](spec/src/introduction.md) defines the behaviour.
The pinned ooRexx interpreter is evidence when checking it.

## Before changing code

- Follow the [clean-room rules](docs/developing.md#setup). Write your own source and
  documentation, and leave `third_party/`, `ootest/` and `oodocs/` read-only.
- Get the project owner's ruling before writing a new Deviation. Get approval before
  adding an `unsafe` site or a macro beyond small mechanical code.
- Cite the specification by rule ID in comments that explain behaviour. Explain the
  contract and what makes it hold.

## Run the checks

For a first change to the numeric crate, this works from a fresh clone
without the SVN checkouts.

```sh
cd rust
cargo test -p interrexx-num --lib
cargo fmt --all -- --check --config error_on_line_overflow=true
```

For broader changes, see the [test setup](README.md#testing) and run Clippy
from `rust/` with `cargo clippy --workspace --all-targets -- -D warnings`.
Use the [memory cap](docs/developing.md#running-oorexx-and-interrexx) when
running either interpreter. For a new regression test, check that it fails
with the fix reverted. Prefer checking a side effect the fix must produce.

## Open a pull request

Open it as a draft and describe the change as it will read after merging.
Keep dependent PRs that touch the same files in a linear stack. Link the
issue and report which checks passed. File or link issues for deferred work.
