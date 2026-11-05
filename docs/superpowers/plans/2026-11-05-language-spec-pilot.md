# Language Specification Pilot Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `spec/` exists as interrexx's own rexxref, with its checker in CI, an mdBook preprocessor,
and a pilot of nine builtins whose every rule is tested.

**Architecture:** A dependency-light crate `rexx-spec` parses the pages (front matter, rules,
examples, rule links) and serves both the checker and the preprocessor. The checker is an
integration test in `rexx-exec` (`tests/spec.rs`), so it can run examples in process and reuse the
oracle helpers in `tests/support/`. The pages are Markdown under `spec/src/`.

**Tech Stack:** Rust 2024, `serde_json` (preprocessor protocol), `walkdir`, existing `rexx-exec`
test support. No new dependencies beyond the workspace's.

**Spec:** `docs/superpowers/specs/2026-11-05-language-spec-design.md`

## Global Constraints

* No CPL content (Moritz 2026-11-05: "I don't think we should have CPL content in the repo, so own
  words."). Page writers may read rexxref and the ANSI draft (fetched to a scratch directory, never
  committed) to derive semantics, and cite them by section. No sentence, table or example is copied
  or closely paraphrased. Error message texts never appear; error cases name condition and code.
* Rules state derived semantics, not whatever the crate does (Moritz 2026-10-19: "Always try to
  derive the semantics, and follow what you could model. The oracle doesn't always get it right.").
* No known-failing rules. A crate fix for a rule lands in the same PR. A fix bigger than a day's work
  stops for Moritz.
* A new numbered Deviation needs Moritz's ruling first; it takes the next number in
  `docs/superpowers/plans/phase-4-exclusions.txt` and adds the register entry in the same PR.
* Never read or modify `third_party/`. The interpreter runs under `memcap`.
* Existing corpus `.rex` files are not edited (their bytes and line numbers are recorded).
* PRs are merged only with Moritz's explicit approval. Run `cargo fmt --all --check` and
  `cargo clippy --locked --workspace --all-targets -- -D warnings` before every push.
* Pilot builtins: `SUBSTR`, `SUBWORD`, `TRANSLATE`, `UPPER`, `VERIFY`, `WORD`, `WORDINDEX`,
  `WORDLENGTH`, `WORDPOS`. Function form only.

## Decisions this plan adds

* **Front matter is a strict YAML subset**, parsed by `rexx-spec` itself: one `key: value` per line,
  values plain or double-quoted scalars, or an inline list `[a, b]`; `#` comments. Anything else is a
  checker error. It stays valid YAML, and no YAML crate is added.
* **Keys:** `element` (required), `kind` (required: `builtin`, `instruction`, `directive`, `class`,
  `method`, `concept`, `chapter`), `syntax`, `ansi`, `rexxref` (optional strings), `deviations`
  (optional list of integers, default empty). Unknown keys are errors.
* **Every page has front matter.** Chapter index pages and `conventions.md` use `kind: chapter` or
  `concept`.
* **Chapter prefix** is fixed by directory: `builtins/` → `bif.`, `instructions/` → `instr.`,
  `expressions/` → `expr.`, `parsing/` → `parse.`, `conditions/` → `cond.`, `classes/` → `class.`,
  `spec/src/conventions.md` → `conv.`. A rule ID matches `^[a-z][a-z0-9]*(\.[a-z0-9][a-z0-9-]*)+$`.
* **Rule links** are written `[text](r:ID)`. The preprocessor rewrites them to the page and anchor;
  the checker resolves them.
* **Example blocks:** a fenced block with info string `rexx` followed (after blank lines only) by one
  with info string `output` or `output rc=N`. A `rexx` block not followed by `output` is not an
  example and is not run. An example belongs to the nearest `r[...]` above it on the page; an example
  before any rule is an error.
* **Citations:** `spec:` followed by IDs, inside a `//` or `/* */` comment in `.rs` and `.rex` files,
  or a `#` comment line in files under `tests/**/testdata/` or `*cases*` directories, in tracked files
  under `rust/`. Found through `git ls-files`.

## Review Focus

1. A page with Windows line endings or a trailing space after `r[ID]`: parsed the same as a clean
   one, or reported, never silently dropping the rule. Owner: Task 1.
2. An example whose program reads from standard input or prints nothing: runs with empty stdin and
   compares an empty output block, never hangs. Owner: Task 2.
3. A rule ID cited only from a file outside `rust/` or from an untracked file: not counted as
   covered. Owner: Task 2.
4. An output block whose expected text lacks a final newline while the program prints one (or the
   reverse): the comparison rule is fixed and stated (output block content plus one `\n`, exactly),
   and a mismatch is reported with both sides shown. Owner: Task 2.
5. A rule ID that appears twice across two pages: reported with both locations. Owner: Task 1.

---

### Task 1: `rexx-spec` parser

**Files:**
- Create: `rust/crates/rexx-spec/Cargo.toml`, `rust/crates/rexx-spec/src/lib.rs`,
  `rust/crates/rexx-spec/src/front_matter.rs`, `rust/crates/rexx-spec/src/page.rs`
- Test: `rust/crates/rexx-spec/tests/parse.rs`

**Interfaces:**
- Produces:
  - `pub struct FrontMatter { pub element: String, pub kind: Kind, pub syntax: Option<String>, pub ansi: Option<String>, pub rexxref: Option<String>, pub deviations: Vec<u32> }`
  - `pub enum Kind { Builtin, Instruction, Directive, Class, Method, Concept, Chapter }`
  - `pub struct Rule { pub id: String, pub line: usize, pub text: String }`
  - `pub struct Example { pub rule: String, pub line: usize, pub program: String, pub output: String, pub rc: i32 }`
  - `pub struct RuleLink { pub id: String, pub line: usize }`
  - `pub struct Page { pub path: PathBuf, pub front: FrontMatter, pub rules: Vec<Rule>, pub examples: Vec<Example>, pub links: Vec<RuleLink> }`
  - `pub struct Problem { pub path: PathBuf, pub line: usize, pub message: String }`
  - `pub fn parse_page(path: &Path, text: &str) -> Result<Page, Vec<Problem>>`
  - `pub fn load(root: &Path) -> (Vec<Page>, Vec<Problem>)` — every `.md` under `root/src` except `SUMMARY.md`, plus the design's checks 1, 2 and 7 (front matter, IDs incl. duplicates across pages and `spec/withdrawn.txt`, prefix by directory, `SUMMARY.md` against the files).
  - `pub fn prefix_for(relative: &Path) -> Option<&'static str>`

- [ ] **Step 1: Write the failing tests** in `tests/parse.rs`, using inline page texts:
  - `a_page_yields_its_rules_examples_and_links`: a page with front matter, two rules, one example
    (`say substr('abc', 2)` / `bc`), one `[x](r:bif.substr.result)` → 2 rules, 1 example with
    `rule == "bif.substr.result"`, `output == "bc\n"`, `rc == 0`, 1 link.
  - `output_rc_is_read`: info string `output rc=1` → `rc == 1`.
  - `crlf_and_trailing_space_parse_the_same`: the same page with `\r\n` and `r[bif.x.y] ` gives the
    same rules as the clean one.
  - `bad_front_matter_is_reported`: missing `element`, unknown key `foo`, `kind: banana`, a nested
    mapping → one `Problem` each, with its line.
  - `example_before_any_rule_is_reported`, `malformed_id_is_reported` (`r[Bif.X]`, `r[bif]`).
  - `load_reports_duplicates_withdrawn_prefix_and_summary`: a temp tree with two pages sharing
    `bif.a.b`, a withdrawn ID, a `bif.` rule in `conventions.md`, and a `SUMMARY.md` naming a missing
    file and omitting an existing one → one `Problem` per defect, the duplicate naming both paths.

- [ ] **Step 2: Run them to see them fail**

Run: `cd rust && cargo test -p rexx-spec --test parse`
Expected: compile failure (crate empty).

- [ ] **Step 3: Implement** the types and functions above. Line-based parsing; fenced blocks are
  ```` ``` ```` lines with an info string. Output block text is its lines joined with `\n` plus a
  final `\n`.

- [ ] **Step 4: Run the tests**

Run: `cd rust && cargo test -p rexx-spec`
Expected: all pass.

- [ ] **Step 5: Commit** (`rexx-spec: parse specification pages`)

### Task 2: the checker and the `spec/` skeleton

**Files:**
- Create: `spec/book.toml`, `spec/src/SUMMARY.md`, `spec/src/introduction.md`, `spec/withdrawn.txt`,
  `rust/crates/rexx-exec/tests/spec.rs`
- Modify: `rust/crates/rexx-exec/Cargo.toml` (dev-dependency `rexx-spec`)

**Interfaces:**
- Consumes: Task 1's `load`, `Page`, `Problem`.
- Produces: `fn citations(repo: &Path) -> BTreeMap<String, Vec<(PathBuf, usize)>>` (in `spec.rs`);
  the test `the_specification_holds`.

`introduction.md` (`kind: chapter`) says what the specification is, who it is for, that it is the
authority, how to read a rule and an example, and that rexxref and ANSI are cited, not copied. It
holds no rules.

- [ ] **Step 1: Write the failing tests** in `spec.rs`:
  - `the_specification_holds`: `load(repo/spec)` plus checks 3 (every rule has an example or a
    citation), 4 (every citation exists), 5 (every example's stdout equals its output block and its
    exit code equals `rc`, run through `rexx_exec::run_program` with empty stdin and the watchdog
    bound the corpus uses), 6 (`deviations` ↔ `….deviation.N` rules, and each `N` has an entry
    ` N. ` in `phase-4-exclusions.txt`), 8 (links resolve). Collects every `Problem` and fails once,
    listing them all.
  - `a_seeded_defect_is_caught`: copy `spec/` to a temp dir, add a page with an uncited rule, an
    example whose output is wrong, and a link to `bif.nope`; the same checks report exactly those three.
  - `citations_are_comments_in_tracked_files`: a cited ID in a `// spec:` comment counts; `spec:` in
    code (`fn f(spec: &str)`) does not; a citation in an untracked file does not.
  - `an_example_reading_stdin_ends`: an example `parse pull x; say '['x']'` / `[]` passes.

- [ ] **Step 2: Run them to see them fail**

Run: `cd rust && memcap 8G cargo test -p rexx-exec --test spec`
Expected: compile failure.

- [ ] **Step 3: Implement** `spec.rs` and the skeleton. A mismatch message shows the page, line,
  expected and actual output.

- [ ] **Step 4: Run**

Run: `cd rust && memcap 8G cargo test -p rexx-exec --test spec`
Expected: 4 passed.

- [ ] **Step 5: Commit** (`spec: the checker and the skeleton`)

### Task 3: the oracle gate mode

**Files:**
- Modify: `rust/crates/rexx-exec/tests/spec.rs`

**Interfaces:**
- Consumes: `support::oracle::locate()`, `Oracle::run_with`, `CppOutcome`.
- Produces: the test `examples_agree_with_the_oracle_or_are_deviations`, gated on
  `REXX_SPEC_ORACLE` (set, non-empty, not `0`), otherwise passing at once with a note on stderr.

- [ ] **Step 1: Write the failing test**: under the gate, every example is written to a temp file
  and run on the oracle under the private runtime directory helper (`support/oracle_runtime.rs`).
  An example whose oracle stdout or exit code differs fails unless its rule ID contains
  `.deviation.`. A self-test seeds one non-deviation example the oracle answers differently from its
  output block (`say 1/3` with output `x`) in a temp copy and asserts it is reported.

- [ ] **Step 2: Run to fail, Step 3: implement, Step 4: run**

Run: `cd rust && REXX_SPEC_ORACLE=1 memcap 8G cargo test -p rexx-exec --test spec`
Expected: all pass (no pilot pages yet).

- [ ] **Step 5: Commit** (`spec: run the examples on the oracle under a gate`)

### Task 4: the mdBook preprocessor

**Files:**
- Create: `rust/crates/rexx-spec/src/bin/mdbook-rexx-spec.rs`, `rust/crates/rexx-spec/src/render.rs`
- Modify: `spec/book.toml` (`[preprocessor.rexx-spec]`, `command = "mdbook-rexx-spec"`)
- Test: `rust/crates/rexx-spec/tests/render.rs`

**Interfaces:**
- Consumes: Task 1's `parse_page`.
- Produces: `pub fn render(text: &str, pages: &BTreeMap<String, PathBuf>, here: &Path) -> String`,
  where `pages` maps each rule ID to its page path.

The binary speaks mdBook's preprocessor protocol with `serde_json::Value` only: `supports <renderer>`
exits 0; otherwise read `[context, book]` from stdin, rewrite every `Chapter.content` with `render`,
write the book to stdout. No `mdbook` crate.

`render`: drops the front matter; puts a header of the syntax, the ANSI and rexxref references and
the Deviations; turns `r[ID]` into `<div class="rule" id="r-ID"><a href="#r-ID">[ID]</a></div>`;
rewrites `(r:ID)` links to the relative page path plus `#r-ID`.

- [ ] **Step 1: Write the failing tests**: `render` on a sample page gives the anchor `id="r-bif.substr.result"`,
  a cross-page link `../builtins/substr.md#r-bif.substr.result`, and no `---` front matter; the
  binary, given a two-chapter book JSON on stdin, returns JSON whose contents are rendered.
- [ ] **Step 2: Run to fail, Step 3: implement, Step 4: run**

Run: `cd rust && cargo test -p rexx-spec`
Expected: all pass.

- [ ] **Step 5: Commit** (`rexx-spec: the mdBook preprocessor`)

### Task 5: the ten questions (controller)

**Files:**
- Create: `docs/superpowers/records/2026-11-05-spec-pilot/questions.md`

Written by the controller, not a page writer, before Tasks 6–9. Ten questions about the pilot
builtins, at least five about an edge or an error, each with its exact answer (a returned string in
quotes, or a condition and code) checked by running it on the crate under `memcap`. Where the crate
and the derived semantics disagree, the answer is the derived one, and the question is marked as
expecting a crate fix.

- [ ] **Step 1: Write and check the questions. Step 2: Commit** (`spec pilot: the ten questions`)

### Task 6: conventions and the builtins chapter

**Files:**
- Create: `spec/src/conventions.md`, `spec/src/builtins/index.md`
- Modify: `spec/src/SUMMARY.md`

`conv.` rules for what the pilot needs: strings and characters; positions counting from 1; a whole
number argument and the checks it gets (non-numeric, not whole, out of range, each with its
condition and code); a length; a pad (exactly one character); an omitted versus an empty argument;
too many arguments; how a rule names an error. `builtins/index.md` (`kind: chapter`) holds the rules
shared by every builtin call (argument count and omission, the error for a call with too many
arguments), linking to `conv.` rules.

Every rule gets an example or a cited test, run on the crate and, through Task 3's gate, on the
oracle.

- [ ] **Step 1: Write the pages. Step 2: run** `memcap 8G cargo test -p rexx-exec --test spec` and
  `REXX_SPEC_ORACLE=1 ...`; **Step 3:** fix the crate for any failing rule (with its own test),
  or stop for Moritz with the proposed Deviation; **Step 4: Commit** (`spec: conventions and builtins chapter`)

### Task 7: `SUBSTR` and `SUBWORD`

**Files:** Create `spec/src/builtins/substr.md`, `spec/src/builtins/subword.md`; modify
`SUMMARY.md`; crate fixes and their tests as needed.

Each page: front matter with `syntax`, `ansi`, `rexxref`; rules for the result, each argument's
default, check and error, and the edges (`n` past the end, `length` 0, empty `string`, the pad);
an example per rule where a one-line program can show it. Every register Deviation that touches the
builtin (search `phase-4-exclusions.txt` for its name) gets its `deviation.N` rule.

- [ ] **Step 1: Write the pages. Step 2: run both modes. Step 3: fix the crate or stop for Moritz.
  Step 4: Commit** (`spec: SUBSTR and SUBWORD`)

### Task 8: `WORD`, `WORDINDEX`, `WORDLENGTH`, `WORDPOS`

As Task 7, for `spec/src/builtins/{word,wordindex,wordlength,wordpos}.md`. The word-splitting rule
(which characters separate words) is stated once in `builtins/index.md` and linked.

- [ ] **Steps as Task 7. Commit** (`spec: the word builtins`)

### Task 9: `TRANSLATE`, `UPPER`, `VERIFY`

As Task 7, for `spec/src/builtins/{translate,upper,verify}.md`.

- [ ] **Steps as Task 7. Commit** (`spec: TRANSLATE, UPPER and VERIFY`)

### Task 10: the close

- [ ] **Step 1:** Full checker in both modes; `cargo test --workspace --no-fail-fast` under memcap;
  fmt and clippy.
- [ ] **Step 2:** The ten-question check: a fresh agent given only `spec/` and `questions.md`'s
  questions (not the answers) answers them. Compare exactly. Each wrong answer: fix the page, rerun a
  fresh agent. Record the runs in `docs/superpowers/records/2026-11-05-spec-pilot/`.
- [ ] **Step 3:** Open the PR with the design, this plan and the pilot. Report to Moritz and wait for
  his approval to merge.
