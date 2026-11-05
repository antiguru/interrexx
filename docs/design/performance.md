# Performance

## Measuring a change of our own

Compare two builds of interrexx by **retired instructions**, not wall clock. Instruction counts
reproduce to several significant figures; wall clock on a busy host moves several percent and has
inverted the sign of a 2% change.

1. Build both arms from committed trees into separate target directories, and compare the binaries'
   `sha256sum`. A clean `git status` says nothing about what is in `target/`.
2. Run `rust/bench-programs/callgrind.sh base=A head=B` (whole-process instruction counts per
   program, with and without libc; `-B PCT` fails a delta over PCT), or
   `perf stat -e instructions:u` with one event per run, so nothing is multiplexed.
3. Read a program the change cannot reach as a control. If it moves, the measurement is noise.
4. Check stdout is identical between arms; a number from a run that did different work compares
   nothing.
5. Before building an optimisation, ablate to its ceiling: make the thing free, unsoundly if need
   be, and measure. If the ceiling is small, stop.
6. Profile on the event you are bound by. The interpreter is instruction-bound, so sample with
   `perf record -e instructions:u`; cycle sampling ranks the wrong things. Use `cycles:pp` before
   blaming one instruction, and expand inlined frames before calling a profile flat.
7. To price a struct's size, pad it with a `[u8; N]` field and measure the slope.

A regression check is `callgrind.sh` against a pinned binary of the last accepted state; the
budget is a small whole-process percentage per program, so cumulative drift is visible.

## Measuring against ooRexx

* Compare in **cycles or wall clock**, never instructions: the two interpreters do different work
  per instruction (ooRexx allocates an integer object per operation; interrexx keeps small integers
  in the handle), so an instruction ratio is not a ratio of anything.
* ooRexx must be an **-O3 build** (CMake `Release`, as `scripts/setup-third-party.sh` makes) of the
  version interrexx tracks. An `-O0` build makes interrexx look 3x faster; `RelWithDebInfo` (`-O2
  -g`) is close enough to look plausible and is still not comparable. Check `parse version` and the
  build flags before trusting a ratio.
* Interleave the two interpreters run by run, from a fresh empty directory, with the same memory
  cap on both. `bench-programs/wallclock.sh -x PROGRAMS` adds an `oracle` arm;
  `rexx-bench-suite` counts cycles and instructions on the same runs as the wall times.
* Report a figure with its date, both builds' identities and the ooRexx optimisation level.

## The bench suite

`rust/bench-programs/` holds one program per dimension: `dispatch`, `dispatchclass`, `sendloop`,
`varlookup`, `compound`, `strings`, `arith`, `textnum`, `decloop`, `decrender`, `parse`, `alloc`,
`alloc4c`, `churn`, `heapshape` (full-GC pause), `nop` and `assign` (per-clause cost), `fibcall`,
`fibfunc`, `intcall`, `extcall` (a native extension call), `dirread`, `hashnew`, `pkglocal`,
`interpret`, `define`, `startup`, and `pingpong/` (message, semaphore and guard handoffs between
activities). `rust/bench-rexxcps/rexxcps.rex` is REXXCPS with fixed loop counts, so both sides do
identical work. Each program's output is deterministic, and the README says what each covers.

`rust/crates/rexx-bench` has the tools: `rexx-bench-suite` (interleaved interrexx against ooRexx
with counters), `rexx-arms` (two builds of interrexx, instruction counts), `rexx-time` (cold
start) and `rexx-bench-band` (run-to-run spread of one comparison).

## Current figures

Wall clock against an ooRexx 5.3 build, measured 2026-09-10 at `8565dcfa8`; interleaved, three
repetitions, median. Below 1.00x interrexx is faster.

| program | ratio | program | ratio |
|---|---:|---|---:|
| `emptyloop` | 0.61x | `dispatch` | 1.78x |
| `alloc4c` | 0.62x | `strings` | 1.80x |
| `compound` | 0.63x | `dispatchclass` | 1.81x |
| `varlookup` | 0.76x | `alloc` | 1.81x |
| `arith` | 1.17x | `rexxcps` (clauses/s) | 1.97x |
| `startup` | 1.73x | | |

The most recent run counting cycles on both sides (2026-08-20, at `d90de68e3`) gave cycle ratios of
0.57x `alloc4c`, 0.53x `compound`, 0.60x `emptyloop`, 0.71x `varlookup`, 1.15x `arith` and 1.23x
`strings`. Where interrexx is slower the gap is retired instructions in value operations
(allocation, string materialisation, `PARSE`, decimal loop control); it has fewer cache and branch
misses than ooRexx on every axis measured except L1i.

## Tried and rejected

* **A bytecode VM replacing the execution core**: ooRexx is a tree walker of the same family, and
  the VM spike's speedups were measured on a design that could not run `CALL`, `INTERPRET` or `DROP`.
* **Cutting op dispatch or fusing ops**: removing a third of all executed ops is worth under 2%.
* **Stripping per-clause bookkeeping**: bounded at 19% on `varlookup` and about 5% elsewhere, and it
  breaks `rexxcps`.
* **Cheaper temp rooting**: removing it outright is a 1.4% ceiling.
* **Pointer-based heap addressing** and a **columnar heap layout**: heap representation is low
  single digits of ordinary programs.
* **A chunked slot table** so slots never move: +370M instructions on `rexxcps` to save 6 MB of
  copying, and the generation check stays because slots are still recycled.
* **`smallvec` for decimal digits** and **`compact_bytes` for short strings**: a safe hand-rolled
  inline enum gives more inline capacity at the same width.
* **A side arena or out-of-line storage for long strings**: helps 7 of 12 million string creations
  on `rexxcps`.
* **Pooling `PARSE` buffers**: every probe about 3% slower.
* **Atomic quickening slots**: a cost per pass on `arith` for a `Sync` property `Chunk` does not
  have anyway.
* **Boxing a rarely populated `Activation` field**: it was hot, and boxing cost 2%; a better hasher
  for it won 8.8%.
* **`#[inline(never)]` on the byte-string constructors** to undo a layout shift: an out-of-line call
  on every string construction for a movement that was not attributable.
* **A C allocator (mimalloc, jemalloc)**: not pursued; it links C into a pure-Rust build.
