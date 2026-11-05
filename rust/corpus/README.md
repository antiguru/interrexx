# L0 differential corpus

Each `.rex` file under `lang/` and `num/` (and the few others `differential.txt`
names) is run under both interpreters by the differential harnesses, and their
normalised stdout, stderr, and exit code must match exactly.

## The one rule: determinism

A program here must produce byte-identical output on every run of the *same*
interpreter. No `DATE()`, no `TIME()`, no process IDs, no file system state, no
directory listings, no addresses, no iteration over an unordered collection.

**Narrowed for a program's own files, not dropped.** Each corpus program runs
with its working directory set to one of its own, created empty before the
oracle's run and emptied again before this crate's, so both sides see the same
absolute path and the same state. A program there may create, read and delete
files inside that directory. It still prints no timestamp, no size of a file it
did not write, no directory listing it did not create, and no host variable's
value.

**Narrowed again for compiled extensions.** A program may load one of the
oracle's own compiled extensions, which is a file outside its run directory and
outside this repository's own build. It is named through an
`LD_LIBRARY_PATH={oraclelib}` sidecar rather than found on the ambient machine,
it is loaded and never rebuilt, and it prints no address: a `.Pointer` renders
differently on every run.

**`rxmath`, `rxregexp`, `rxsock` and `rxunixsys` are this crate's own Rust
libraries**, answered before any `dlopen`, so a program naming them tests those
libraries against the oracle's shared ones and never reaches this crate's
shared-library path. A program that is about that path names `rxmathc`,
`rxregexpc`, `rxsockc` or `rxunixsysc` instead and puts `{instruments}` on its
`LD_LIBRARY_PATH` after `{oraclelib}`: the harness's directory of links to the
oracle's `librxmath.so`, `librxregexp.so`, `librxsock.so` and
`librxunixsys.so` under those names (`INSTRUMENTS` in
`crates/rexx-exec/tests/support/sidecar.rs`), so both sides load the same C
binary. Each such program has a `_rust` copy naming the Rust library.

**Hash iteration order.** The order `DO OVER` yields a hash collection's
indexes in is the order of the oracle's own hash table: bucket
`key->getHashValue() % bucketSize`, iterated bucket-ascending. Whether that
reproduces from run to run depends on what the keys are, not on which collection
class it is. A string's `getHashValue` is `31*h + byte` over its own content
(`classes/StringClass.hpp:328`), and a number's delegates to its string value.
Every other object's is `RexxObject::identityHash`, which is
`((uintptr_t)this) ^ UINTPTR_MAX` (`classes/ObjectClass.hpp:340`) -- the
address, which ASLR moves on every run.

Measured, ten oracle runs each: a program filling an `IdentityTable` with eight
fresh `.Object`s and printing their values in iteration order gives five
distinct outputs, each a rotation of one cyclic sequence. The same program over
a `StringTable` of eight string keys gives one output in ten.

So **no program here may print the iteration order of a collection whose keys
are objects rather than strings or numbers**. That covers an `IdentityTable`
outright and a `Table`, `Set`, `Bag` or `Relation` whenever what went into it
was not a string. The oracle does not agree with itself there, so a differential
has nothing to compare against. Iterating a string-keyed table and printing
something order-independent -- a count, or a lookup of each index against the
table -- is fine, and is what `lang/do_over_string_table.rex` does.

A class object is an exception, because `RexxClass::getHashValue`
(`classes/ClassClass.cpp:209`) answers the hash of the class's id string, so a
class object's bucket does not move with ASLR. `RexxNilObject`'s hash is a
stored constant, so `.nil` is reproducible too. The collection this reaches is
the collector's own uninit table, whose iteration order is the order class
`UNINIT`s run at termination. `lang/uninit_class_sweep_order.rex` prints that
order and is allowed to; the sequences it and the oracle produce are asserted in
`crates/rexx-classes/tests/uninit_sweep_order.rs`. An instance in the same table
is not reproducible (its hash is the inherited `identityHash`): a program whose
termination sweep mixes an instance `UNINIT` with a class `UNINIT` is the shape
the rule forbids -- measured, twenty runs of one class at bucket 16 beside one
live instance gave the instance first nineteen times and the class first once.

When the self-test (`--cpp X --rs X`, the same binary twice) reports a
divergence, the corpus is at fault, not the differ. Fix the program; never loosen
`normalize` to make it pass, because everything normalisation strips is a class
of divergence that can no longer be detected.

## A program that needs a second file beside it

Two shapes work. What selects a program here is the extension, so a required
file spelled `.cls` is outside every scan; and a required file spelled `.rex` is
simply a corpus program of its own, filed like any other. A file opening with a
directive has no main body and runs as a no-op, which makes the second shape
available even for a pure library.

What is not allowed is a `.rex` helper nobody files: it would be a program
nothing runs, which `every_lang_program_is_run_or_named_unfiled` exists to
catch.

`lang/package_requires.rex` and `lang/package_namespace.rex` take the `.cls`
shape, because their helpers are class libraries. Naming them without an
extension in the directive means the search's own `.cls` step finds them. A
`.cls` helper needs no `sourceline_oracle` expectation and appears in no corpus
list, because no scanner sees it.

## Running it

```sh
cargo build --release
./target/release/rexx-diff \
    --cpp ../build/bin/rexx --rs ../build/bin/rexx --corpus corpus
```

Expect `N programs, 0 divergences` and exit 0, where `N` is whatever the tool
itself counts (`find corpus -name '*.rex' | wc -l` agrees with it). Substituting
any other binary for `--rs` should report `N` divergences and exit 1; that
negative control is what makes a zero meaningful.

## Corpus lists

One path per line, relative to this directory; `#` starts a comment.

* `differential.txt` -- every program the differential harnesses run under both
  interpreters (`crates/rexx-exec/tests/corpus.rs`, `coverage.rs`,
  `collect_stress.rs`, `ir_recorded.rs`, `trace_oracle.rs`, and the IR shape
  sweep). Every `lang/*.rex` is either listed here or named in `unfiled.txt`
  with a reason; the entries outside `lang/` are pinned by `corpus.rs`'s
  `NON_LANG_PROGRAMS`.
* `concurrency.txt` -- the programs that start activities, which the
  scheduler-simulation gate (`tests/group_runs/`) replays. Each is also in
  `differential.txt`, and `corpus.rs` checks that.
* `suite-rexxutil-groups.txt` -- the ooTest groups that name a RexxUtil routine,
  for `REXX_SUITE_GROUPS`; `suite.rs` re-derives it from the command its header
  records.

What a listed program may not print, beyond the determinism rule above:

* **`QUEUED()` only over a queue the program filled itself.** The data queue is
  named by `RXQUEUE`/`rxapi` and outlives the program, so a count taken over
  lines the program did not push is a statement about the session it ran in.
* **Not `PARSE SOURCE`'s third word**, the program's own absolute path.
  `lang/parse_sources.rex` and `lang/source_arg.rex` project it away.
* **Not `PARSE VERSION`**: all three fields are the interpreter's own build
  identity. `crates/rexx-exec/tests/parse_version_oracle.rs` checks it instead.

The command line and a non-empty console are outside every corpus program's
reach: the harness passes no arguments and an empty stdin. Both are compared
against the oracle in `crates/rexx-exec/tests/input_oracle.rs`, which drives the
`rexx` binary with an `argv` and a fed stdin instead.

## Directories

* `lang/` -- the language and class-library programs, one subject per file, named
  for what they pin. Most say in their header which wrong answer a defective
  engine would print.
* `num/` -- the numeric core: `NUMERIC DIGITS`/`FORM`/`FUZZ`, rounding,
  notation thresholds, comparison, operators, error numbers. All are in
  `differential.txt`.
* `expr/` -- not a `rexx-diff` corpus. `precedence.tsv` is a table of generated
  expressions with the value or error number the oracle gave for each, read by
  `rexx-parse`'s differential test so that `cargo test` needs no built C++
  interpreter. Its own header records how it was generated and what the columns
  mean.
* `errors/` -- not a `rexx-diff` corpus either. `parse-errors.tsv` holds one row
  per program with the answer `build/bin/rexxc` gave for it: the major number,
  sub-number, reported line and secondary message when it refused, and `ok` when
  it translated. `rexx-parse`'s `tests/errors.rs` reads it and checks both
  directions, so a parser that rejected nothing and one that rejected everything
  both fail; `rexx-exec`'s `tests/rexxc.rs` checks this crate's `rexxc` against
  each row. The `class` field says whether the oracle refused to *translate* the
  program or rejected it later, while installing the package, and is recorded per
  row rather than derived from the error number: the two install-time classes
  are `98.903` and `90.999`, so a rule keyed on a `98.9xx` prefix would cover
  half of them. A row records only what the oracle said, so a divergence shows up
  as a test failure rather than as an expected value.
* `gate-tables/` -- `.rex` probes, one per row of a gate table, and **not** a
  `rexx-diff` corpus in the sense the top of this file means. Most of them
  diverge from the oracle, which is what their row records.
  `gate-tables/README.md` says which table reads which subdirectory and what a
  missing or unnamed probe does. The determinism rule still binds them, because
  `rexx-diff`'s walk is recursive and its self-test reads every `.rex` under
  `corpus/`; agreement between the two interpreters does not.
* `model/` -- the executable numeric and stream models, checked against the
  native implementations by `crates/rexx-exec/tests/numeric_model.rs` and
  `stream_model.rs`.
* `docs/` -- tables extracted from the reference documentation (class set,
  class methods, directive options, hierarchy edges, `provide` sections).
* `sim-oracle/` -- the committed oracle outcome sets of the scheduler-simulation
  gate.
* `suite-oracle-runs/` -- recorded oracle runs behind the suite's environment
  licences.
* The remaining `.tsv` and `.txt` files are tables read by one test each; each
  names its reader and column meanings in its own header.

## Things this corpus learned the hard way

`say a"|"b` does not concatenate three values. `"|"b` is read as a **binary
string literal** -- the `b` suffix binds to the preceding quote -- and the
program dies with error 15.4. `parse_template.rex` uses explicit `||` for
exactly this reason.

The same rule usually fails *silently*. `say a''b` looks like the classic idiom
for blank-free concatenation but prints `x`, not `xy`: `''b` is an empty binary
literal, so the line concatenates `a` with `""` and never reads `b`
(`whitespace_significant.rex`).

`.integer` is not an environment symbol; `Integer` is internal and unexposed.
`.rexxinfo` is an *instance*, not a class, so it has no `~id`.

`LEAVE name`/`ITERATE name` accept two different kinds of name, and a *clause*
label is neither of them. `outer: do i = 1 to 3` then `leave outer` fails the
same way wherever the clause label sits relative to the `DO`: `rexxc` accepts
the program, and at run time `LEAVE`/`ITERATE` refuse the name with error 28.3
("must either match the label of a current loop or block instruction"). A
clause label is a `SIGNAL` target, and 28.3 does not distinguish "wrong kind of
name" from "right kind, wrong loop". Error 47.2 ("Labels are not allowed within
a DO/LOOP block") is unrelated: a translate-time rejection of a clause label
written *inside* a loop's body. What works is the loop's own **control
variable** (`do outer = 1 to 3` / `leave outer`, used throughout
`leave_nested_outer.rex` and `leave_iterate_variants.rex`) or the explicit **`DO
LABEL name`** form (`do label outer i = 1 to 3` / `leave outer`, which also
works on a plain non-repetitive block and is the only corpus program that
constructs the parser's `Loop::label` field -- see `do_label.rex`).

A `WHEN` whose `THEN` instruction is itself a `WHEN` clause
(`select_when_absorption.rex`) parses and is accepted, and the second `WHEN` is
never added to the enclosing `SELECT`'s clause list: it is silently swallowed as
the first `WHEN`'s (empty) consequence. That is confirmed only for the case
where the first `WHEN`'s condition is *true*, which is all the shipped program
uses. The *false* variant crashes the oracle and is entry 1 of
`oracle-crashes.txt`; it is deliberately not a corpus program, because the
corpus's one rule is byte-identical output between the two interpreters, and a
memory-safe reimplementation neither can nor should reproduce a C++ segfault.

**Editing a `corpus/lang/*.rex` file invalidates its `sourceline_oracle`
expectation, even if you only touched a comment.**
`rust/crates/rexx-parse/tests/sourceline_oracle/<name>.txt` records the oracle's
line count and every one of its lines by position, so any edit to the text or the
line count desyncs it. The only tell is `cargo test -p rexx-parse` failing a
`sourceline`-named test. To change a comment, keep the line count and every
non-comment line, then regenerate with the driver named in
`sourceline_oracle.rs`'s own module comment: save it to a scratch file as
`srclines.rex`, then from the repository root,

```bash
for f in rust/corpus/lang/*.rex; do
  name=$(basename "$f" .rex)
  out=$( ( ulimit -v 1048576; build/bin/rexx SCRATCH/srclines.rex "$f" \
          2>/dev/null ) | grep '^%SRCG%' )
  count=$(printf '%s\n' "$out" | sed -n 's/^%SRCG%COUNT //p')
  { echo "count $count"; printf '%s\n' "$out" | sed -n 's/^%SRCG%L//p'; } \
    > rust/crates/rexx-parse/tests/sourceline_oracle/$name.txt
done
```

Regenerating for every file in the loop is cheap and idempotent, so do that
rather than hand-picking which files changed.
