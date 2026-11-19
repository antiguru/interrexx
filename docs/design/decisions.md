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
* **Evaluation order is specified; error precedence is not** (2026-11-16, #395, #757). An operator
  evaluates its operands left before right, both before it checks either, and a call evaluates its
  arguments from the left (ANSI 7.4, 9.2.1); each control expression of a DO is evaluated and
  checked in its turn, in the written order, as its own operation (ANSI 8.3). An error raised while
  an operand or an argument is evaluated is raised at that point. These stay specified because Rexx
  side effects are observable: a function call in an operand or an argument runs or does not run.
  When the checks of one operation (one operator, one builtin or method invocation, one
  instruction's checks) find more than one fault, which of their errors is raised is unspecified;
  it is the error of one of the faults. The faults include a wrong argument count, an omitted
  required argument and a `STRING` answer that has no string value. A builtin's conversion of its
  arguments by `STRING` is part of its checks: a call with no fault sends `STRING` to each argument
  from the left before the other checks (`r[bif.index.call.convert]`), and for a call with a fault, whether and
  how many `STRING` sends happen is unspecified. Whether the checks after the first fault are made,
  and so whether the messages they send are sent, is unspecified too; and when an operation finds a
  fault that raises a condition, such as LOSTDIGITS, and one that raises an error, which comes
  first is unspecified, so Deviation 102 is withdrawn. This replaces the precise order of
  2026-11-12, which took ANSI's results (ANSI 4.1 makes them normative) and ooRexx's order where
  ANSI is silent. ooRexx follows ANSI's evaluation order but not its precedence: `LEFT`, `VERIFY`,
  `X2D`, `FORMAT` and `STRIP` check a later argument first, several methods and RexxUtil functions
  check arguments before their count, and it does not keep ANSI's subcodes either, raising 41.1 for
  `1 + 'b'` where ANSI gives 41.2. Java, Python and Rust libraries, Haskell's imprecise exceptions
  and SQL do not specify which of several faults is reported. interrexx keeps ooRexx's precedence
  where it has it, so that comparisons stay quiet, but that is a convenience, not part of the
  contract. Together with the entry below, the contract of an error ends here: a fault is reported,
  by the code and subcode of a fault that is present.
* **An error's contract is its code and subcode** (2026-11-14, #388, #672). The error number and,
  where the specification states one, the subcode are specified. The message text is not, and
  the items of `ADDITIONAL` for an error the interpreter raises are not unless a rule states one.
  Java, Python and Ruby draw the line in the same place: the exception's class and subclass are
  the contract, its message is not, and structured data on it is specified only through a
  documented accessor. A fault that is present is reported by its code and subcode; where an
  operation has several, which of them is reported is unspecified (2026-11-16, see above).
  Objects a program supplies to `RAISE` may be specified to reach the condition unchanged; the
  array the interpreter builds around them is not. Comparisons check only what is specified
  (`docs/design/testing.md`).
  Whether a built-in method is a level of a traceback is unspecified, because a build may not
  have it (2026-11-14, #388); comparisons drop those frame lines.
* **Standard output is buffered, with a switch to write it unbuffered** (2026-11-15, #706,
  #721). A standard output that is not a terminal is handed over in blocks, and its order against
  what native code writes itself is unspecified (Deviation 40). `REXX_UNBUFFERED` or `rexx -u`
  writes each write as it is made, which is ooRexx's order. Python draws the line in the same
  place, with `PYTHONUNBUFFERED` and `-u`. Handing the buffer over at every entry into native
  code and every return from it would keep ooRexx's order unasked, but costs a write per call,
  and per clause where an RXHLT or RXTRC exit is registered.
* **Comparing a `DateTime`, `TimeSpan` or `File` with an object of another class** (2026-11-15,
  #74, Deviation 126). `compareTo` and the ordering operators raise `SYNTAX 88.914`, since any
  order between the two would be arbitrary. The equality operators answer not equal, as ooRexx
  does: a hash collection compares the indexes whose hash codes collide with `==`, so a raise there
  would make holding such an object beside one of another class fail by chance.
* **A collection finds the very object it stores without asking `==`** (2026-11-15, #74,
  Deviation 78 withdrawn). A lookup or a search compares identity first and sends `==` only to
  another object, as ooRexx does, and as Python's containers and Java's `HashMap` do: an object
  is taken to equal itself, whatever its `==` answers.
* **A builtin's conversion of its arguments is part of its checks** (2026-11-16, Deviation 170
  withdrawn). A call with no fault sends `STRING` to each object argument from the left before the
  other checks, as ooRexx does; a sweep of the no-fault calls found no difference in the sends.
  For a call with a fault, whether and how many sends happen stays unspecified.
* **Method lookup is live** (2026-11-13, #141, #624, Deviation 110). An object sees every later
  change to its class, superclasses and mixins, as in Smalltalk, from which ooRexx's object model
  descends.
* **Hash collections iterate in insertion order** (2026-11-12, #36, Deviation 33). Every hash
  collection yields its entries in the order they were first put: a `put` that replaces an item
  keeps its entry in place, and an entry removed and put again comes last. Where several entries
  share an index or hold equal items, lookups and removals act on the oldest, and lists of them
  answer oldest first. ooRexx's order follows its bucket layout; it acts on the newest of several
  entries at one index and searches items in its storage order. The order is specified rather than
  left to the implementation, because a deterministic order that ships becomes a contract in
  practice, as Python's dict did between 3.6 and 3.7. A string's hash code is a keyed hash of its
  characters under a seed drawn for each run, and a stem's tail map is keyed from the same seed, so
  a program cannot choose many string keys of a hash collection, or many tails of a stem, that share
  one probe path. The other maps keyed by names a program picks, the variables that `INTERPRET` and
  `VALUE` create and a stem's exposed tails, are not keyed. The seed shows only in `hashCode`
  answers, never in an order or in which entry a lookup finds. For objects that break the `hashCode`
  contract, whether a lookup finds an entry is unspecified (2026-11-16), and so is which entry a
  search acts on when an `==` or `hashCode` it sends changes the collection.
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
  inventory crate's message files, written to `STYLE.md`. A text keeps ooRexx's wording only where
  it is ANSI's, is one of the canonical phrases with names filled in, or is a plain statement a
  writer given only the code's meaning would produce nearly word for word; every other text is
  written fresh, even where ooRexx's sentence reads well. Kinds of fault that several codes share
  use one phrase, even where ANSI or ooRexx word them differently. The tools that compared texts
  with ooRexx's catalogue while the texts were written do not stay in the source, and no tool
  compares them afterwards: a one-time scan found the remaining quotations, and its method and
  limit are in `docs/developing.md`.
* **Recorded oracle runs hold no text of ooRexx's** (2026-11-14, #388). An error line in a recorded
  run keeps its code and drops the text after it, which is the part the comparisons ignore.
  Comparisons cut the same parts of a live run by structure, never by matching ooRexx's wording.
* **A regular build does not read `third_party/`** (2026-11-13, #388). Tables derived from ooRexx
  are checked in, with a CI job that checks them against the pinned revision. The C and C++
  headers in `include/` are generated from the Rust layout definitions, in a per-platform layout
  that holds Unix content only so far. The `build`, `shard`, `doctest` and `clippy` CI jobs
  run on a checkout without the submodule. `build`, `doctest`, `clippy` and `msrv-standalone` keep
  a cargo cache that no job with the submodule saved, and `shard` keeps none, so a build script
  that reads the tree fails there. Tests that need the tree are held out of that run by a filter
  that names each one with its reason, not skipped at run time.
* **`include/` declares the names of ooRexx's headers, and no others** (2026-11-14, #650, #712).
  A CI job diffs every name both ways and fails on a difference that the script does not list with
  a reason. The eight exports the pinned headers leave undeclared stay undeclared in `include/`;
  the libraries export them, and `interrexx-capi/tests/link.rs` finds them by symbol, since the ABI
  dump has no declaration to measure. Declaring them would add names that ooRexx's headers lack,
  which the diff exists to prevent. `RexxConditionData` is declared, with a `repr(C)` twin that the
  ABI dump measures.
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
* **The language level is the one interrexx implements, not the pinned one** (2026-11-17, #796).
  `REXX_CURRENT_LANGUAGE_LEVEL`, `LanguageLevel`, `.RexxInfo~languageLevel` and `PARSE VERSION`
  answer 6.06 while the pinned ooRexx answers 6.07, whose CATCH and FINALLY clauses interrexx
  does not have, so a program that asks for the level is not told features are there that are
  not. `include/` declares `REXX_LANGUAGE_6_07`, which only names the level, and
  the names diff lists the differing current level with this reason. The level moves when the
  features of the next level are in.

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
