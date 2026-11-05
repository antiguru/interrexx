# Benchmark programs

One `.rex` file per performance dimension, each sized to run roughly 0.5-2s under `build/bin/rexx` except
`startup.rex`, which is deliberately as close to instantaneous as a program can be.

| File | Covers |
|---|---|
| `dispatch.rex` | Tight method-send loop: one object, one instance method, 5,000,000 sends |
| `dispatchclass.rex` | The same dimension reached through a class object instead of an instance: one class-method send per pass, 4,000,000 sends. Runs on this crate, where `dispatch.rex` needs `~new` |
| `varlookup.rex` | Plain simple-variable read/write, no stems, 19,000,000 iterations |
| `compound.rex` | Stem/compound-variable access — 500 tails is inside the 100-10,000 range where a compound-variable memo pays off |
| `strings.rex` | `SUBSTR`/`POS`/`CHANGESTR`/concatenation, 3,000,000 iterations |
| `arith.rex` | Decimal arithmetic, alternating `NUMERIC DIGITS 9` and `NUMERIC DIGITS 20` every iteration so both settings are exercised throughout the run rather than only at startup |
| `alloc.rex` | Allocation churn: a fresh `.array` and `.string` every iteration, neither retained past it, sized to force multiple collections |
| `alloc4c.rex` | Allocation churn restricted to the 4c surface (no message sends): a new compound-variable tail and a concatenated string every iteration. Not the same axis as `alloc.rex` -- see its own header for what carries over and what does not |
| `parse.rex` | `PARSE` in the shapes a real program uses: a positional pattern, a **variable** pattern, and `PARSE VAR` with several targets. |
| `textnum.rex` | Values that arrive as **text** and are then used as numbers, with fresh handles each iteration so a handle-keyed cache cannot answer from the previous one. |
| `decloop.rex` | A **decimal-controlled** `DO` -- `do j=1.1 to 2.2 by 1.1`. `emptyloop.rex` and every other loop here use integer control, and the two are not the same axis: integer control is 1.22x the oracle and decimal control 1.63x |
| `decrender.rex` | The same decimal control plus the renderings a real program performs on the control value (`length(j)`, `j='foobar'`), which is `rexxcps`' inner-loop shape. **1.99x**, the worst ratio in the suite, and the construct that carries `rexxcps` above every other primitive it exercises |
| `nop.rex` | The fixed cost every clause pays: a counted loop whose body is 100 `nop` clauses, one per line, so the loop step is amortised over them. 1,000,000 passes |
| `assign.rex` | The same shape with 100 `x = y` clauses, `y` assigned once before the loop: the per-clause cost plus one variable read and one write. 1,000,000 passes |
| `fibcall.rex` | Recursive fib(22) by internal `CALL`, 30 times |
| `fibfunc.rex` | Recursive fib(22) by internal function call, 30 times |
| `sendloop.rex` | An empty `::METHOD` sent 5,000,000 times |
| `extcall.rex` | `liborxfunction`'s `TestIntArg` called 3,000,000 times; needs the library's directory on `LD_LIBRARY_PATH` |
| `dirread.rex` | Directory entry reads by message and by `[]`, and a `.local` entry read as an environment symbol: the store-backed read path, 200,000 passes. Sized against this crate (0.3s), not the oracle (0.05s) |
| `hashnew.rex` | Hash-collection construction: a `.directory~new`, `.stringtable~new` and `.table~new` per pass, 50,000 passes, none kept. |
| `pkglocal.rex` | An environment symbol read once the running package has a local entry: one `~local` write, then 2,000,000 reads of `.stdout`, which the search reaches after the package local misses. |
| `intcall.rex` | Internal package routines, which run no activation: `FILESPEC` of the REXX package and `SysFileExists` of rexxutil, 150,000 passes each. Sized against this crate (0.2s). The oracle takes 1.7s |
| `churn.rex` | Body-byte churn over a live set: 20,000 objects held while 100,000 strings of 10,000 bytes are made and dropped, so each collection marks the live set and the byte trigger sets how often one runs. Sized against this crate (0.1s). The oracle takes 2.1s |
| `interpret.rex` | `INTERPRET` of a fragment holding a literal longer than a handle carries inline, 20,000 passes, so each compiles a chunk that interns a heap string. Measured 0.45s on this crate and 0.09s on the oracle |
| `define.rex` | `Class~define` while instances made under each definition stay live, 3,000 defines with allocation between them: each define retires the class's instance behaviour and each freeing collection looks for the behaviours no live instance holds. Sized against this crate (0.3s) |
| `startup.rex` | `say 1` — cold-start timing (D2's gate), timed separately with `rexx-time`, not through criterion's statistical sampling |
| `pingpong/pingmsg.rex` | Message round trip: `~start` on a new activity and its `~result` awaited, 80,000 times. Recorded against the oracle; in a subdirectory, so the criterion harness does not run it; `callgrind.sh` runs it as `pingmsg` |
| `pingpong/pingsem.rex` | Semaphore post/wait: two activities hand a turn back and forth through two `EventSemaphore`s, 100,000 times each way. As `pingmsg.rex`; `callgrind.sh` runs it by its name |
| `pingpong/pingguard.rex` | GUARD WHEN handoff: two activities pass a turn through one object's variable, each in `GUARD ON WHEN`, 100,000 times each. As `pingmsg.rex`; `callgrind.sh` runs it by its name |
| `heapshape.rex` | Full-GC pause over a ~1M-object graph. It prints its own figures, so the suite reports those rather than timing the process. **The slot strings are wider than seven bytes on purpose**: a shorter one lives in the Rust handle and allocates nothing, which collapses the graph to ~1,001 objects -- see the program's own comment |

## Determinism

Same rule as `corpus/`: byte-identical output on every run of the same interpreter. No `DATE()`,
`TIME()`, process IDs, or unordered iteration. Every program in this directory was run under
`build/bin/rexx` and confirmed to exit 0 with identical output across repeated runs before being
committed.

`heapshape.rex` is the exception and is the reason `NOT_BENCHMARKED` exists: it calls `TIME()` and
prints wall figures, so its output varies between runs by construction. Only its `gc_forced=` line
is byte-stable.

## Sizing

Loop counts were tuned by timing each program directly under `build/bin/rexx` (not through the
criterion harness) and adjusting until each landed between 0.5s and 2s. See
`docs/design/performance.md`.

## Two things this corpus (see `../corpus/README.md`) learned the hard way, reconfirmed here

`say a"|"b` does not concatenate three values — `"|"b` is read as a **binary string literal** (the
`b` suffix binds to the preceding quote) and the program dies with error 15.4. None of these
programs use that form; string concatenation here uses explicit `||`.

`.integer` and `.rexxinfo` are not usable as environment classes (`.integer` is internal and
unexposed; `.rexxinfo` is an instance, not a class). None of these programs reference either.

## Instruction counts: `callgrind.sh`

`callgrind.sh NAME=BINARY NAME=BINARY...` runs every program here, `pingpong/*.rex` (named
`pingmsg`, `pingsem`, `pingguard`) and `../bench-rexxcps/rexxcps.rex` under callgrind on each binary and prints whole-process instruction
counts, which the budget judges (`-B PCT`), beside the counts less libc.so.6 and ld-linux
(`cgsum.py`). `layout-pad.py TREE COUNT` adds never-called functions to a tree's IR driver.
`wallclock.sh NAME=BINARY...` times the same programs, interleaved, or those `PROGRAMS` names
(`PROGRAMS="pingpong/pingmsg pingpong/pingsem pingpong/pingguard"`).
`cgdiff.py A B` prints per-function differences between two callgrind files.
