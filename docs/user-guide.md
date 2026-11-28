# Using interrexx

## Install

The source build needs Cargo and Rust 1.95 or later. On Debian, use
`cargo-deb` to build an installable package.

```sh
git clone https://github.com/antiguru/interrexx.git
cd interrexx/rust
cargo install cargo-deb --locked
cargo deb -p interrexx
sudo apt install ./target/debian/interrexx_*.deb
```

The package replaces an installed `oorexx` package because both install
`rexx`, `rexxc` and shared libraries under the same names. If you want to
keep both interpreters, build from source and invoke each by its full path.
The README lists the package contents. [Debian package](../README.md#debian-package)

For a source build, run this from the repository's `rust/` directory.

```sh
cargo build --release --locked
./target/release/rexx -v
```

The commands are in `rust/target/release` from the repository root. On
Windows, run `.\target\release\rexx.exe -v` from `rust/`. For the examples
below, put that binary directory on your `PATH` or use full paths to the
interrexx commands.

## Run a program

Run a short command with `-e`.

```sh
rexx -e "say 2 + 2"
```

Save the following as `hello.rex` in your working directory.

```rexx
parse arg name
say 'Hello,' name
```

Run it from source, check that it translates, or compile it to an image and
run that image.

```sh
rexx hello.rex Ada
rexxc hello.rex
rexxc hello.rex hello.rxo
rexx hello.rxo Ada
```

`rexxc` replaces an existing output file, so use a distinct output path.
interrexx and ooRexx refuse each other's compiled images. Keep the source
and compile it with the interpreter you will use to run it.
[Invocation](../spec/src/environment/invocation.md#programs-without-source)

## Output, memory and the shared service

- Standard output going to a file or pipe is normally buffered. When you merge
  stdout and stderr, earlier output can appear after an error report. Use
  `rexx -u hello.rex Ada` or set `REXX_UNBUFFERED=1` to get unbuffered
  output on both streams. [Deviation 40](../spec/src/deviations.md#40-buffered-standard-output)
- Set `REXX_HEAP_LIMIT`, for example `64M`, to bound memory held by program
  objects. Crossing the limit can raise `SYNTAX 5.0`. The limit covers strings,
  buffers and collection slots. Interpreter stacks, translated programs and
  other process allocations are outside this limit.
  [Invocation](../spec/src/environment/invocation.md#the-forms-and-the-option-words)
- The `rxapi` service starts on demand for shared queues, registrations and
  the macrospace. Set `REXX_API_DIR` to use a separate service. On Unix, use a
  directory you own that grants no access to other users, such as one with
  mode `0700`. Programs sharing a service must use the same directory. Run
  `rxapi --stop` with that same environment to stop it.
  [Deviation 86](../spec/src/deviations.md#86-the-api-daemon)

## Differences to check when moving from ooRexx

- Hash collections iterate in insertion order. Replacing an item keeps its
  position, and removing it and adding it again puts it last.
  [Deviation 33](../spec/src/deviations.md#33-hash-iteration-order)
- Existing objects see later method changes to their class, including
  `define` and `delete`.
  [Deviation 110](../spec/src/deviations.md#110-instances-follow-every-change-to-their-class)
- If you embed more than one interpreter, each instance has its own
  `.environment` and objects. Keep native object handles with the instance
  that returned them.
  [Deviation 85](../spec/src/deviations.md#85-environment-per-interpreter)
- Use numeric error codes and subcodes in checks. The wording of error
  messages can differ.
  [Errors and results](../spec/src/conventions.md#errors-and-results)
- `rexx -v` identifies the implementation. `PARSE VERSION` still uses the
  ooRexx-style format and reports language level 6.06.
  [Deviation 96](../spec/src/deviations.md#96-version-and-library-banners),
  [Deviation 179](../spec/src/deviations.md#179-language-level-606)

## Known limits

- Language level 6.06 predates the `CATCH` and `FINALLY` clauses and the
  `THROW` instruction. [Deviation 179](../spec/src/deviations.md#179-language-level-606)
- Deeply nested calls raise `SYNTAX 11.1`. The implementation caps live call
  activations at about ten thousand, and available control-stack space can
  cause the condition earlier.
  [Nesting](../spec/src/expressions/index.md#nesting)
- CI tests Linux on x86-64 and 32-bit x86, runs a standalone suite on macOS
  and checks the Windows x64 interpreter and libraries. Host-specific
  routines vary by platform.
  [Test workflow](../.github/workflows/test.yml),
  [macOS tests](../.github/workflows/macos-nightly.yml),
  [Windows checks](../.github/workflows/windows-check.yml)

See the [language specification](https://antiguru.github.io/interrexx/spec/main/)
for the full reference and the [deviation list](../spec/src/deviations.md)
for other differences.
