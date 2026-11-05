# The interrexx language specification: design

## Intent

Interrexx implements a language, but the language is written down nowhere. Today the meaning of a
construct is spread across rexxref, the ANSI draft, the plans, the numbered Deviations, and what the
crate happens to do. This design makes it explicit: **interrexx's own rexxref**, the place an engineer
or an LLM looks up the precise semantics of a language element and gets an exact answer that CI keeps
true.

The specification is the authority. The oracle is evidence used while writing it, as Moritz ruled on
2026-10-19 ("Always try to derive the semantics, and follow what you could model. The oracle doesn't
always get it right."). When the specification, the crate and the oracle disagree, the fix is to the
code, to the rule, or a ruled Deviation, never a silent difference.

## Readers

* An engineer changing the crate, who needs the exact behaviour before touching code.
* An LLM agent doing the same, which reads raw files, often one page at a time, without the history.
* A reviewer judging whether a change is correct.

Not a reader: someone learning Rexx. This is a reference, not a tutorial.

## Goals

1. A reference organised like rexxref: chapters for the program structure, expressions, instructions,
   parsing, builtins, conditions, classes, and so on, so things are where a Rexx user expects them.
2. One page per language element, made of **rules**: each a numbered, stable ID followed by one
   short paragraph stating exactly one behaviour. Error cases name the condition and code (`SYNTAX
   40.23`), never "an error".
3. **Machine-checked:** CI fails when a rule has no test, a test cites a rule that does not exist, an
   example's output differs from the crate's, or a page's metadata is wrong.
4. Deviations from the oracle are rules in the specification, on the page of the element they
   affect.
5. A pilot chapter that proves the format: the string and word builtins `SUBSTR`, `SUBWORD`,
   `TRANSLATE`, `UPPER`, `VERIFY`, `WORD`, `WORDINDEX`, `WORDLENGTH`, `WORDPOS`, and the
   conventions chapter they rely on.

## Non-goals

* A formal or executable semantics (K, Redex, SpecTec).
* Covering the whole language now. Chapters arrive one at a time after the pilot.
* Moving the old plans, records or ledgers into the specification, or archiving history. Those wait
  for the repo cleanup.
* A tutorial, rationale essays, or a history of how a rule was decided.

## Anti-goals

* **No CPL content.** Moritz 2026-11-05: "I don't think we should have CPL content in the repo, so
  own words." rexxref and the ooRexx sources are CPL 1.0, so the specification is written fresh. It
  cites rexxref and the ANSI draft by section, and never copies or closely paraphrases their text,
  their examples, or ANSI's Rexx definitions of the builtins. Error message texts are not reproduced
  either; rules and examples name error codes.
* Prose that sounds precise but that no test pins.
* A second source of truth: a rule and the code it describes change in the same PR.

## Format

### Files

The specification lives in `spec/` at the repository root as an mdBook: `spec/book.toml`,
`spec/src/SUMMARY.md`, and one Markdown file per element, for example
`spec/src/builtins/substr.md`. A chapter has an index page holding its shared rules.

### Front matter

Every element page opens with YAML front matter holding only what tools sort, filter or check:

```yaml
---
element: SUBSTR
kind: builtin            # builtin | instruction | directive | class | method | concept
syntax: "SUBSTR(string, n [, length] [, pad])"
ansi: "9.3.21"           # section in the ANSI X3.274 draft, or omitted
rexxref: "Built-in Functions > SUBSTR"   # where rexxref covers it, or omitted
deviations: []           # numbers of the Deviations this page states
---
```

The checker validates the front matter against a fixed schema. Anything needing explanation goes in
the prose, not the front matter.

### Rules

A rule is a line `r[ID]` on its own, followed by one paragraph, as in the Rust Reference:

```markdown
r[bif.substr.pad.length]
A `pad` argument that is not exactly one character raises `SYNTAX 40.23`.
```

IDs are lowercase, dot-separated, and begin with the chapter's prefix (`bif.`, `instr.`, `expr.`,
`parse.`, `cond.`, `class.`, `conv.` for the conventions chapter). An ID never changes meaning once
merged. A rule that is withdrawn is deleted, and its ID is not reused; the checker keeps the list of
withdrawn IDs in `spec/withdrawn.txt` and rejects them.

A Deviation is a rule whose ID has the form `….deviation.N` (`bif.syswordexp.deviation.90`). Its
paragraph states what interrexx does, then "ooRexx differs:" and what it does. The page lists `N`
in `deviations`. The register (`docs/superpowers/plans/phase-4-exclusions.txt`) stays the one source
of Deviation numbers: every `deviation.N` rule names a register entry `N`, and a new Deviation takes
the next register number and adds its register entry in the same PR, after Moritz's ruling. Moving
the register itself into the specification is later work.

A rule states the behaviour derived from rexxref, the ANSI draft and the language, not whatever the
crate does. There is no "pending" or known-failing state: a rule merges only when the crate meets
it. Where the crate is wrong, the same PR fixes the crate. A fix too large for the PR stops for
Moritz, who decides whether it is fixed first, or the rule waits.

Each element is specified once. A builtin page specifies the function call. The `String` methods of
the same name get their pages in the classes chapter later, stating only where they differ from the
function and linking its rules for the rest.

### Examples

An example is a `rexx` code block followed directly by an `output` block, and belongs to the rule
above it:

````markdown
```rexx
say substr('abc', 2, 4, '.')
```
```output
bc..
```
````

The output block is the program's standard output, byte for byte. An example that ends in an error
catches it (`signal on syntax` and `say condition('o')~code`), so no message text appears. An example
may carry `rc=N` in the output block's info string (```` ```output rc=1 ````) for a nonzero exit.

### Data tables

Lists that are data, rather than prose, live in a file under `spec/data/` and are rendered into the
page. The pilot needs none; the mechanism is built when the first chapter needs it.

## Test citations

A test cites the rules it checks with the token `spec:` followed by one or more IDs, separated by
spaces or commas, inside a comment:

* Rust: `// spec: bif.substr.pad.length` on the line above the `#[test]`.
* A corpus `.rex` program: `/* spec: bif.substr.result bif.substr.pad.length */` near the top.
* A datadriven case file: `# spec: bif.wordpos.start`.

The checker finds citations by scanning the comments of the tracked files under `rust/` for that
token. A citation goes in a new test or corpus program, or in an existing Rust test. Existing corpus
`.rex` programs are not edited, since their bytes are compared and their line numbers recorded.

## The checker

A new crate, `rust/crates/rexx-spec`, parses `spec/src` and offers the parsed pages to two
consumers: a test that is the checker, and an mdBook preprocessor. The checker does not depend on
mdBook. It runs in the existing `cargo test` job and fails when:

1. a page's front matter is missing, does not parse, or breaks the schema;
2. an ID is malformed, duplicated, withdrawn, or has the wrong prefix for its chapter;
3. a rule has neither an example nor a test that cites it;
4. a citation names an ID that does not exist;
5. an example's output, or exit status, differs from the crate's, run in process the way the corpus
   tests run programs;
6. a page lists a Deviation number with no `deviation.N` rule on it, or the reverse;
7. a `SUMMARY.md` entry and the files disagree;
8. a link to a rule (`[...](…#r-ID)` or the preprocessor's rule-link form) names an ID that does not
   exist.

A second, gate-only mode (`REXX_SPEC_ORACLE=1`, as the other gates are gated) runs every example on
the oracle too. An example whose oracle output differs must belong to a `deviation.N` rule. This is how
the oracle stays evidence without being in charge.

## The preprocessor

`mdbook-rexx-spec`, a binary in the same crate, strips the front matter, turns each `r[ID]` line into
a linkable anchor showing the ID, renders a small header from the front matter (syntax, ANSI and
rexxref references, Deviations), and later renders data tables. Building the book is local for now;
CI runs only the checker.

## The pilot

* `spec/src/conventions.md`: what a string, a whole number, a length and a pad are; how arguments are
  counted and checked; how positions count from 1; how a rule names an error; what "returns" means.
  These are the `conv.` rules the builtin pages refer to by link instead of repeating.
* `spec/src/builtins/index.md` and one page per pilot builtin. Each page covers the result, every
  argument's checking and its error, defaults, and the edges (zero, empty string, past the end,
  and digits in arguments where they matter). A page links only to rules that exist; the checker
  resolves links to `r[...]` IDs across pages and fails on a dangling one.
* Every Deviation in the register that touches a pilot builtin gets its `deviation.N` rule on that
  page.
* Every rule has an example or a citing test. Where a rule is only testable through a corpus program
  or a Rust test, the pilot adds that test.
* The oracle gate mode is run once over the pilot, and every difference is either fixed in the crate,
  or brought to Moritz as a proposed Deviation. A new Deviation needs his ruling.
* The pilot is done when the checker passes in CI, and an agent given only `spec/` answers a set of
  ten pre-written questions about the pilot builtins exactly.
  * The controller (not the page writer) writes the questions and their answers before the pages,
    in `docs/superpowers/records/2026-11-05-spec-pilot/questions.md`, each answer checked on the crate.
    At least half ask about an edge or an error.
  * A fresh agent, given read access to `spec/` only and the question list, answers them.
  * "Exactly" means the same returned string byte for byte, or the same condition and code.
  * A wrong answer means the page is unclear or wrong: it is fixed and the agent rerun fresh.
  * This is the pilot's check of the format. Later chapters decide for themselves whether to repeat
    it.

## Decided by Moritz

* Markdown pages with structured front matter, not YAML documents (2026-11-05).
* The specification is interrexx's own rexxref, for engineers and LLMs to check precise semantics
  (2026-11-05).
* Own words; no CPL content (2026-11-05).
* No history archiving yet (2026-11-05).
* PRs are merged only with his explicit approval.
* The specification is normative in its own right; ooRexx and the ANSI standard are informative
  references (2026-11-05, "go ahead" on the proposal). Rules and examples are normative; a page's
  overview and its notes, set as block quotes beginning `**Note.**`, are informative, and the
  checker rejects any other prose after a page's first rule. The pages name the C++ implementation
  ooRexx; "the oracle" names only the test tool that runs it.

## My calls, to check

* `spec/` at the repository root, and a new `rexx-spec` crate.
* YAML inside the front matter.
* The ID scheme and prefixes above, and `….deviation.N` for Deviations.
* The pilot covers the string and word builtins in `SUBSTR`..`WORDPOS`. `SYMBOL`, `TIME`, `TRACE`,
  `TRUNC`, `USERID`, `VALUE` and `VAR` are left out: they depend on chapters (symbols, numbers,
  dates, tracing, the environment, variables) that do not exist yet.
* The crate is fixed to meet a rule, in the same PR; no known-failing rules. This follows your
  2026-10-19 ruling, but it makes the pilot's size depend on what the oracle pass finds.
* Builtin pages specify the function form only; the `String` methods come later and link to them.
* The register stays the source of Deviation numbers.
* The ten-question check: I write the questions, a fresh agent answers from `spec/` alone.
* Citations are found only inside comments, so code like `fn parse(spec: &str)` is not one.
* An example counts as covering its rule, so a rule needs no separate test when its example pins it.
* Error message texts stay out of the specification for now (anti-goals). Whether the specification
  can state message texts at all, given that the crate's texts come from CPL `rexxmsg.xml`, is a
  question for the conditions chapter.
* The book is not built in CI yet.

## Left to the builder

* The front matter schema's exact field types, the parser, and the preprocessor's rendering.
* How examples are run in process, reusing the corpus runner where it fits.
* The wording of every rule, within the anti-goals.
