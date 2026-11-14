# Decisions

This page lists the standing decisions that shape interrexx, with the reason for each. A change
that would reverse one needs the project owner's ruling first; raise it on an issue rather than
working around it. Each entry names the issue or pull request that holds the discussion, where
there is one. The rules themselves live in the specification and in `docs/developing.md`; this
page records why they are the way they are.

## Semantics

* **The specification is normative; ooRexx is evidence** (2026-10-19). Derive a rule from the
  reference manual, the ANSI standard and the rest of the language, and follow it where ooRexx
  departs. ooRexx has shown artefacts of its implementation often enough (stream positions,
  selector order in `DROP` and `EXPOSE` lists, a `RAISE RETURN` with no result) that matching it
  without a derivation copies its accidents. See `docs/design/testing.md`.
* **A new Deviation needs the owner's ruling before it is written** (2026-10-19). That ooRexx is
  wrong decides what interrexx does; it does not license the difference. Implement the derived
  behaviour, then ask, citing the ruling in the entry.
* **A rule pins an `ADDITIONAL` item only where the program needs it** (2026-11-14, #675). Like a
  number parse error that carries the string that failed to parse, an item is stated only where it
  is needed to understand the error and the program cannot get it otherwise; the rest is
  unspecified under `r[conv.error.additional]`.
* **Errors are checked in a precise order** (2026-11-12, #395). Operands are checked left before
  right and a builtin checks its argument count before its arguments, from the left (ANSI 7.4,
  9.2.1), with ooRexx's order where ANSI is silent. Which fault a program sees is part of the
  contract. A relaxed order is a design for an optimising transpiler, which would declare its own
  Deviation; the interpreter keeps the precise one.
* **An error's contract is its code and subcode** (2026-11-14, #388, #672). The error number and,
  where the specification states one, the subcode are specified. The message text is not, and
  the items of `ADDITIONAL` for an error the interpreter raises are not unless a rule states one.
  Java, Python and Ruby draw the line in the same place: the exception's class and subclass are
  the contract, its message is not, and structured data on it is specified only through a
  documented accessor. Objects a program supplies to `RAISE` may be specified to reach the
  condition unchanged; the array the interpreter builds around them is not. Comparisons check
  only what is specified (`docs/design/testing.md`).
  Whether a built-in method is a level of a traceback is unspecified, because a build may not
  have it (2026-11-14, #388); comparisons drop those frame lines.
* **Method lookup is live** (2026-11-13, #141, #624, Deviation 110). An object sees every later
  change to its class, superclasses and mixins, as in Smalltalk, from which ooRexx's object model
  descends.
* **Hash collections are to iterate in insertion order** (2026-11-12, #36). This is not implemented
  yet; #36 tracks it. The hash function is to be seeded against flooding, with the seed invisible to
  programs. Python reached the same pair of decisions after randomised order proved noisy and
  users came to rely on insertion order.
* **When a finalizer runs is unspecified** (2026-09-01). Anything that depends on the order of
  garbage collection only has to be one of the possible orders.
* **Behaviour when memory runs out is best effort** (2026-08-14). ooRexx's limits depend on the
  platform and the allocator, so they are not a target, and the ordinary path pays nothing to
  match them. A failure is still reported rather than aborting where that costs nothing.

## Licence and sources

* **No text under ooRexx's licence enters the repository** (2026-11-05, enforced by #216).
  interrexx is MIT OR Apache-2.0. The specification is written in our own words and cites the
  reference manual and the ANSI standard by section. A short name with no other natural wording
  may coincide with ooRexx's (2026-11-08).
* **Error message texts are interrexx's own data** (2026-11-14, #388). The texts live in the
  inventory crate's message files. A message that states a fact mechanically, or that comes from
  the ANSI standard, may keep its wording; distinctive prose is written fresh. Tools that compared
  texts against ooRexx's catalogue while the texts were written do not stay in the source.
* **A regular build does not read `third_party/`** (2026-11-13, #388). Tables derived from ooRexx
  are checked in, with a CI job that checks them against the pinned revision. The C and C++
  headers in `include/` are generated from the Rust layout definitions, in a per-platform layout
  that holds Unix content only so far.
* **C gets declarations, C++ gets contexts** (2026-11-14, #661). C code may use the headers'
  declarations, entry tables and routines that need no context; a library that calls the
  interpreter through a context is written in C++. ooRexx's own headers already stop C at the
  same point.
* **The headers fix three macros that are broken in ooRexx's** (2026-11-14, #691).
  `SET_REXX_VALUE` sets the type code and the value, `SSIZE_MIN` is the least `ssize_t`, and the
  routine and method macros cast each descriptor value to the parameter type, so a C method can
  take `OSELF`. In ooRexx's headers the first does not compile, the second is 0 and the third
  fails in C. Code that builds against ooRexx's headers keeps its meaning, except code that took
  `SSIZE_MIN` to be 0. These are fixes, not Deviations.

## Performance

* **The budget guards obvious regressions** (2026-10-19). Small drift is reported, not fixed for
  its own sake; a targeted performance round handles it.
* **The speed of a traced program is not a target** (2026-10-19). What counts is what tracing
  support costs a program that does not trace.

## Code and pull requests

* **Comments cite the specification** (#389, lint #415). A comment states what the code does and
  why, by rule id. ooRexx is named only in comparison tests, fixtures, Deviation and defect
  entries, and where the specification has a gap.
* **No measurements in code comments** (#594). A comment states the invariant and why it holds;
  numbers and history go in the pull request body, which becomes the squash commit.
* **Pull requests are drafts, described as merged.** The body describes what the change contains,
  not how it came about, and carries no session link. Pull requests that touch the same files
  form one linear stack, so each can merge without conflicting with the next.
* **A toolchain migration also builds on Rust 1.98** (2026-11-12). CI follows stable, and people
  build with older toolchains.
* `unsafe` and macros beyond small mechanical code need the owner's approval
  (`docs/developing.md`).
