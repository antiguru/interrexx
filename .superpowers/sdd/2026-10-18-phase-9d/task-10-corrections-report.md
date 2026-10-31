# Task 10 consolidated correction report

## Result and source identity

The accepted SUPPLIER correction and Important findings F1 and F2 are implemented at `ad0741664719bf6b01c063079c2c61a715999dff`. Its Rust tree is `783b7378c80b0fba36f14912e0179dbe7c70cfe2`. Seven focused regressions pass under ordinary execution and collection on every allocation, and fourteen paired programs match stdout, stderr and exit status independently. These results establish the correction wave's focused verification, while the controller retains the fresh Task 10 workspace, corpus, suite, TSan and performance gates.

The starting linked worktree was `plan/rust-rewrite` at `5a3d5a52d`. Pure moves extracted constant evaluation and method-table installation before behavior changes, with visibility and module-relative path adjustments only. `close/corrections/move-proof-verification.json` compares both extracted blocks against the starting source and confirms their normalized byte identity at `81e870fe6`. Every original comment in the extracted blocks was retained, and `close/corrections/commits.txt` lists the complete source and test commit sequence.

## Corrected behavior

F1 now folds fallible constant-result rendering into the local evaluation result before common activation cleanup. Both outcomes restore guards, call context, activation and slot stacks, and saved clause state through the existing cleanup path. The regression loads a traced constant whose DEFAULTNAME raises 98.900, catches it in the caller and prints `caught 98.900` followed by `after`. Paired existing programs also cover successful intermediate tracing, invocation-only tracing and untraced constant evaluation.

F2 retains non-SYNTAX ADDITIONAL values without rendering their elements for catalogue substitutions. ARRAY argument rendering uses `intermediate_text`, so TRACE R does not evaluate a DEFAULTNAME for an argument line it will not emit. SYNTAX substitutions render after RESULT evaluation, using a separately rooted original array handle and live per-index reads so RETURN callbacks can replace its contents safely. Deviation 83 DESCRIPTION conversion and the existing requestArray validation point remain unchanged, and the existing conversion-boundary tests pass.

SUPPLIER dispatch now runs normally for collections with a hash store, including Directory and StringTable subclasses. The shared callers are `native_define_methods`, `enhance_class_methods` and `native_enhanced`, all corresponding to reference `RexxClass::createMethodDictionary`, which sends SUPPLIER before walking a real Supplier. Method-table validation still precedes `define_method_table`, and the invalid-method witness confirms installation remains atomic. The tests observe DEFINE and recursive DEFINEMETHODS security checkpoints inside the supplier and separately exercise a copied protected primitive SUPPLIER intercepted by the manager.

Native-directory compatibility retains the previous direct walk only for a Body::Native table resolving an unrestricted primitive SUPPLIER. These directories lack the hash store required by the current native supplier, so removing the compatibility walk would break existing `.methods` callers. `write_object_method` permits scoped method installation only for Instance and Class receivers, and primitive classes remain locked, so an object-specific public native override cannot reach this shortcut through the current supported mutation surface. The existing native-directory refusal boundary is retained without expanding the native-directory protocol or adding a Deviation.

## Authority and coverage

The documentation authority is `oodocs/rexxref/en-US/fundclasses.xml`, section `mthClassDefineMethods`, lines 376 through 390. It defines a collection of method names and method bodies, uppercase names, replacement behavior and unavailable entries. The implementation authority is `interpreter/classes/ClassClass.cpp`, `defineMethodsRexx` and `createMethodDictionary`, plus `interpreter/instructions/RaiseInstruction.cpp` and `interpreter/concurrency/Activity.cpp` for operand evaluation and message substitution. The original Task 6.12 re-review, accepted queue entry, final whole-phase review and correction brief supplied the specific failure and security witnesses.

The seven new corpus programs are registered in `rust/corpus/phase-9.txt`. Their sourceline sidecars came from the official driver embedded in `rexx-parse/tests/sourceline_oracle.rs`, running copied programs in fresh scratch directories. `close/corrections/oracle-driver.py` preserves those oracle channels and does not instantiate repository files. The existing `define_methods_supplier.rex` was restored byte for byte after a filename collision during test setup, with the new subclass witness stored separately as `define_methods_supplier_override.rex`.

* `constant_result_cleanup`: caller trap and continuation after traced constant failure.
* `raise_additional_items`: untraced USER ADDITIONAL and ARRAY elements with failing DEFAULTNAME.
* `raise_additional_modes`: USER and SYNTAX, ADDITIONAL and ARRAY, each under OFF, R and I, including exact oracle stderr.
* `raise_additional_return_mutation`: RETURN replaces `old` with `new` before SYNTAX substitution.
* `define_methods_supplier_override`: subclass supplier, user sends, recursive installation, security-manager observations, plain Directory and protocol object.
* `define_methods_protected_supplier`: copied protected primitive supplier and manager-provided Supplier.
* `method_table_supplier_boundaries`: atomic failure, subclass class methods, enhanced objects and plain StringTable.

## Commands and observed statuses

All Cargo commands used `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/codex-p9d-t88-target`, four jobs and an outer `memcap 8G`. Individual foreground commands used a 580-second timeout where applicable, with separate stdout, stderr and status files. Oracle children ran from fresh scratch directories with the prescribed 1 GiB address-space limit, while paired interpreter runs had an outer `memcap 2G` and individual 30-second timeouts. No full close gates, profiling runs or ooTest source edits occurred in this correction wave.

| Evidence prefix under `close/corrections/` | Command or purpose | Observed status |
| --- | --- | --- |
| `red` | `cargo test -p rexx-exec --test close_corrections -j4`, first module relocation build | 101, relative collection path did not compile |
| `red-2` | Same command after committed module-path correction | 101, all three original semantic regressions failed |
| `green` | Same command after initial fixes | 0, three regressions passed |
| `protected-red` | Same test target, filter `a_copied_protected_supplier` | 101, manager interception bypassed |
| `green-2` | Close regressions and `semantic_boundaries` | 0 |
| `adjacent-red` | Close regressions including twelve-mode RAISE matrix | 101, USER ARRAY under TRACE R rendered an invisible argument |
| `focused-green` | Close regressions, semantic boundaries, collection policy, refusal dispositions and closed-phase labels | 0 |
| `trace-order-red` | Close regressions, filter `raise_substitutions` with oracle stderr assertion | 101, SYNTAX substitution preceded ARRAY/RESULT trace |
| `final-green` | Close regressions, semantic boundaries and refusal sites after trace correction | 0 |
| `mutation-red` | Filter `syntax_substitution_reads`, first witness spelling | 101, both interpreters rejected unparenthesized RETURN syntax, oracle 221 |
| `mutation-red-2` | Same filter after parenthesized RETURN witness correction | 101, crate printed `old.` and oracle printed `new.` |
| `final-green-2` | `cargo test -p rexx-exec --test close_corrections --test semantic_boundaries --test refusal_sites -j4` | 0, seven close regressions, five semantic tests and eight refusal tests |
| `refusal-refresh` | `REXX_REFUSAL_SITES_REFRESH=1 cargo test -p rexx-exec --test refusal_sites the_table_holds_every_constructor_the_source_defines -j4 -- --test-threads=1` | 0, no table changes |
| `refusal-verify-final` | Refusal-site tests without refresh | 0 |
| `sourceline-verify-final` | `cargo test -p rexx-parse --test sourceline_oracle -j4` | 0 |
| `fmt-before` | Formatting check before formatting commit | 1, formatting differences |
| `fmt-final-final` | `cargo fmt --all --check` on final source | 0 |
| `clippy-touched-final` | `cargo clippy -p rexx-exec --all-targets -j4 -- -D warnings` | 0 |
| `build-cli-final` | `cargo build -p rexx-exec --bin rexx -j4` | 0 |

Raw channels retain every failed attempt under exclusive names. The first SUPPLIER oracle witness also failed because its RESULT variable was cleared by a send, and the corrected witness is retained under `define_methods_supplier_override-v2`. The first paired matrix run retains its stderr mismatch under `paired`, while `paired-final` contains the corrected fourteen-program comparison. No earlier failure output was overwritten or treated as semantic regression proof when it was a compile or witness-syntax error.

## Paired evidence and handoff

`close/corrections/paired-final` contains fourteen program directories with independent crate and oracle stdout, stderr and exit-status files. Every `comparison.json` has all three comparison fields true, and each invocation JSON records the source commit, source hash, binary hash, command and working directory. The successful existing programs include native `.methods` installation, supplier subclasses, SYNTAX ADDITIONAL conversion and traced and untraced constants. `close/corrections/completion.json` summarizes these comparisons and all command statuses.

The tested crate binary SHA-256 is `dbf73f30724018918625539592f974ca8d5ca5ba1a078dc105081fc29216f513`. The oracle binary SHA-256 is `bb5bb8ccbb96c376e329b91aafdad891f975ba06c941dbceba82c3848fa13019`. The accepted SUPPLIER queue entry and inventory row now point to the verified correction source, while all other dispositions and allowed sets remain unchanged. All foreground children exited, and `remaining-processes.txt` records no remaining Cargo, rustc or rexx process at handoff.

The final whole-phase review and its original review probes are retained alongside the correction evidence. The user's ZIP and previous caches and profiles were untouched. The native-directory protocol boundary above remains a pre-existing limitation rather than a newly approved departure. Fresh full closure evidence and the scoped review verdict remain the controller's responsibility after this source handoff.

## Guard and seam follow-up

The fresh release close at `1ef21db99` exposed two fix-induced failures, retained in `close/attempt-1/G4-test-release.stdout` with status 101. The `.local` refusal assertion failed because the correction confined `unbuilt_collection_owner` to the Body::Native compatibility branch, while `.local` and `.environment` are hash-backed Body::Instance directories. The seam assertion failed because the pure move introduced a clearance consumer in a file missing from its exact allowlist. The controller released the source freeze and authorized this narrow second round after archiving the negative attempt at `4bd2e0d70`.

Source commit `988026178ba00de5f2fa7c737114e68743333b55` restores the existing whole-collection guard before either supplier path and preserves its original reason comment and exact refusal assertion. `unbuilt_collection_owner` recognizes only the actual environment and local directory identities, so collection subclasses and protected suppliers retain normal dispatch. The helper contract now distinguishes supplier order from the native compatibility path's sorted entries. The seam allowlist adds only `src/dispatch/class_protocol/method_table.rs`, with an unlisted sibling-path negative control and no broader prefix allowance.

The follow-up verification ran from `rust/` under `memcap 8G`, with `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/codex-p9d-t88-target`, four Cargo jobs and a 580-second timeout per command. The whole library passed all 1,239 tests, and the six requested integration targets passed another 37 tests. Formatting and touched-crate Clippy both returned 0. Separate raw channels, exact invocation metadata and the replay driver are retained under `close/guard-followup/`.

| Evidence prefix under `close/guard-followup/` | Exact command | Status |
| --- | --- | --- |
| `tests` | `cargo test -p rexx-exec --lib --test dispatch_seam --test close_corrections --test semantic_boundaries --test refusal_sites --test refusal_dispositions --test closed_phases -j4` | 0 |
| `fmt` | `cargo fmt --all --check` | 0 |
| `clippy` | `cargo clippy -p rexx-exec --all-targets -j4 -- -D warnings` | 0 |

`preservation.json` confirms the F1 constant-evaluation file, F2 RAISE runtime file, seven original corpus fixtures and existing exact refusal assertion file are byte-unchanged from `4bd2e0d70`. Only the method-table helper and dispatch-seam test changed in this follow-up's Rust diff. Every verification command finished in the foreground, and no child remains from this work. The controller retains the scoped follow-up review and fresh full closing gates.

## Sanitizer compatibility and reader lifetime follow-up

The third scoped round addresses the retained second close's actual sanitizer failures without changing language semantics or expanding the deferred native-call ownership work. The collector responsibility moved verbatim at `8ad0a33f8`, with normalized identity independently checked in `close/tsan-corrections/static-proof.json`. Commit `bfe213064` implements the reader lifetime and test-harness corrections, `69809073e` corrects the copied helper's obsolete skip-count description, and `9219362a4` records the official refusal-location refresh. Final focused verification used the same Rust tree at `088fe9389`, namely `d0ffe4ddf33aae12cd43f3657029d8da5e0170c7`.

The collector keeps both reader threads, declares their result vectors outside `thread::scope`, and immediately drops both scoped handles. Each reader catches its own panic and returns an empty vector, preserving the previous `join().unwrap_or_default()` behavior without allowing a detached reader panic to escape through the scope. The scope continues to protect the borrowed result storage until both readers complete normally, while HALT retains the existing ability to abandon the busy worker promptly. Actual-collector tests cover callback panics in both reader directions and 200,000 bytes from a live pipe after its peer reaches real EOF.

The isolated resource-test launcher omits RLIMIT_AS only when `REXX_TEST_THREAD_SANITIZER=1` is explicitly supplied by an instrumented test driver. Its ordinary 8 GiB hard address-space cap, open stdin pipe, 25-second deadline and huge-stack worker-reservation failure preflight remain intact. The helper stays behind `#[cfg(test)]`, and neither production behavior nor TSAN_OPTIONS detection was added. The selector still asserts equality with canonical `CARGO_BIN_EXE_rexx`, removing only the invalid current-executable parent-layout assertion.

The current-plan `close/tsan.sh` preserves the historical helper's commands, options, suppression file and original exclusions. Its only execution changes are the explicit driver marker and the exact additional DEPTH entry `eval::tests::a_parse_pattern_and_an_expression_share_the_eval_depth_limit`, under the controller's recorded compatibility ruling. The historical helper remains byte-unchanged, with its full original copy, SHA-256 and precise diff retained under `close/tsan-corrections/`. The runner now points to the current-plan copy, and the unchanged full 100,000-depth test passes in ordinary debug and release.

| Evidence under `close/tsan-corrections/` | Command or identity | Observed result |
| --- | --- | --- |
| `normal` | Whole library and affected integration targets | 101 only for the extracted `escalated` constructor's stale line number, 464 versus 466 |
| `refusal-refresh` | `REXX_REFUSAL_SITES_REFRESH=1 cargo test -p rexx-exec --test refusal_sites the_table_holds_every_constructor_the_source_defines -j4 -- --test-threads=1` | 0, only that source position changed |
| `normal-final` | `cargo test -p rexx-exec --lib --test rexx_command --test signals --test stdin_contention --test program_end --test concurrency_tests --test close_corrections --test dispatch_seam --test environment_seam --test refusal_sites --test refusal_dispositions --test closed_phases -j4` | 0, 1,241 library tests and 150 integration tests |
| `depth-release-final` | `cargo test -p rexx-exec --release --lib eval::tests::a_parse_pattern_and_an_expression_share_the_eval_depth_limit -j4 -- --exact` | 0, full unchanged depth-boundary test |
| `unsafe-final` | `cargo test -p rexx-core --test unsafe_sites -j4` | 0 |
| `fmt-final`, `clippy-final` | `cargo fmt --all --check`; `cargo clippy -p rexx-exec --all-targets -j4 -- -D warnings` | Both 0 |
| `tsan-build` | `cargo +nightly test -Zbuild-std --target x86_64-unknown-linux-gnu -p rexx-exec --lib --test signals --test concurrency_tests --no-run --message-format=json -j4` | 0 |
| `tsan-lib-0` through `tsan-lib-3` | Four exact stdin parent launchers, marker 1 and `REXX_POOL_FAILURE_CHILD` unset | All 0, parent and child test identities recorded, no reports |
| `tsan-lib-4` through `tsan-lib-7` | Four exact scheduler spawn-failure tests | All 0, no reports |
| `tsan-lib-8`, `tsan-lib-9` | Exact collector panic and real-EOF controls | Both 0, no reports |
| `tsan-signal-0` through `tsan-signal-2` | Previously failing nested-redirection, raised-target-condition and partial-command-output signal tests | All 0, no reports |
| `tsan-selector` | Exact Cargo-owned binary selector test under the instrumented build layout | 0, no reports |

Each Cargo command used four jobs, `CARGO_INCREMENTAL=0`, an outer `memcap 8G` and an individual bound below ten minutes. Ordinary checks used `/tmp/codex-p9d-t88-target`, while focused instrumented checks reused the prescribed second-close TSan cache. Instrumented execution preserved `suppressions=rust/tsan.supp allocator_may_return_null=1 second_deadlock_stack=1`, changing only each exclusive log path. The committed `normal.py`, `sanitized.py` and `readers.py` drivers provide replay commands, exact selected test names, raw channels and invocation metadata.

The old instrumented CLI reproduces finished-thread reports at status 66 for a no-op stderr reader and for real stderr EOF before live stdout. Real stdout EOF before live stderr is a passing old-code control, status 0 with no report, because the old collector joins stdout first. The initial probe driver's final assertion incorrectly required all three old cases to emit reports, and `before-driver-failure.json` retains that distinct driver failure without misclassifying the control. All three corrected-CLI cases return 0 without report files, preserve exact stdout `a\nb -4 1 first\nhalted 5\n` and empty stderr, and finish in approximately 1.7 seconds.

The corrected instrumented CLI SHA-256 is `58ec6da7981da19e5fded9ba2d7fa9950231ae33eea184b338576a556cf497b5`. `completion.json` records hashes for all selected instrumented artifacts, while `preservation.json` proves the F1/F2/guard files, seven witnesses, unchanged depth-test source and suppression bytes against the earlier source. The original whole-library crash lacks exact test-name attribution, so this report does not claim that attribution has become certain. No broad workspace, suite or profile rerun occurred here, every foreground child exited, and the controller retains scoped approval and all fresh closing gates.
