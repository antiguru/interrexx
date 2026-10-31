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
