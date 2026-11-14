# Task 4c report

Status: DONE. Commits 184cd56aa (#683), 820d6102c (stdout order).

## #683: cause and fix

* Cause: `run_package_unloaders` (dispatch/library.rs) calls `library.close()`, which dropped the `libloading::Library` (dlclose). `Mapping` counts only calls the interpreter makes in, not code an attached thread runs on its own, so a thread returning from `DetachThread` ran into unmapped code.
* Fix: `Mapping::close` now hands the handle to `keep_mapped` (rust/crates/interrexx-api/src/load.rs:~176) instead of dropping it. Rows and hooks still stop calling in (`open` flag), unloaders still run. The spec has no rule on unmapping; `r[native.threads.end]` is cited for why the thread is not waited for.
* Repeated interpreters: the same file dlopens to the same handle, so `keep_mapped` records handle addresses in `KEPT_MAPPED` and drops (dlclose, decrementing) a handle already kept. One reference and one entry per distinct library. Unit test `closing_the_same_library_repeatedly_keeps_one_entry` (51 load/close cycles of libm, list length constant). Integration test `a_closed_library_stays_mapped` (tests/load.rs) checks /proc/self/maps; it is weak, since other tests in that binary may map the same file.
* Stress test: `a_running_callback_holds_program_end_a_hundred_times` (tests/attached_threads.rs:468), `#[ignore]`; the attached-stress job already runs `--run-ignored ignored-only` for that binary, so test.yml is unchanged.

## Evidence

* Before (HEAD load.rs, 3 x 100 runs, one process at a time): attempt 2 failed with SIGSEGV (rc 139, after about 115 s, so around run 50); attempts 1 and 3 passed. 1 crash in 300 runs, a lower rate than the 1-in-15-to-40 seen under parallel load.
* After (3 x 100): all passed, about 235 s each.
* Also run: `cargo nextest run -p interrexx --test instance_identity --test attached_threads` 127 passed, 2 skipped (ignored); interrexx-api load tests and `unsafe_sites` pass; `cargo clippy --workspace --all-targets` clean.

## Stdout-order flake

* Cause: the driver's stdout is already unbuffered (`setvbuf _IONBF`), so the guessed cause was wrong. The interpreter buffers `say` output (`interp.out`) and hands it over only before a read, an idle or a command, or at the buffer bound. The instance thread hands it over after `CallProgram` has returned, racing the driver's `printf("data set")`.
* Fix: tests/instance_identity/inst.rex ends with `.stdout~flush`, which hands the output over before the program returns. The driver cannot do this, so the fixture change is in the Rexx program, the assertion is unchanged.
* Before: 6 of 30 parallel runs failed (`"data set\nsame\n"`). After: 30/30, 30/30, 30/30.

## Commands

Test binary run 30-wide as `memcap 8G $BIN application_data_reaches_a_started_activity --exact`; CARGO_TARGET_DIR=/home/moritz/dev/repos/interrexx-work/t-attach, CARGO_INCREMENTAL=0.
