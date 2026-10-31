# Whole Phase 9d source review

## Verdict and scope

Source verdict: Changes required, with two Important correctness findings and no Critical finding. The review covers the net Phase 9d source range `71ab69cfe..82e10bb92` on `plan/rust-rewrite`, using the source companion and focused current-source inspection. `git diff --quiet 71ab69cfe 328fc0e28 -- rust` returned 0, confirming that the implementation plan's starting revision has the same Rust tree as the companion's baseline. The requirements are the Phase 9d design, parent Phase 9 sections 3 and 7 through 10, and the plan's global constraints and subsequent recorded rulings. This verdict concerns source correctness and does not establish completion of the fresh Task 10 gates.

## Important findings

* **F1: A traced constant result skips activation cleanup when rendering raises.** At `rust/crates/rexx-exec/src/install.rs:2827`, `self.result_text(value)?` returns directly from `eval_constant_expression` before `guard_off_at_end`, restoration of `call_context`, activation and slot pops, and restoration of the saved clause state. A caller with `SIGNAL ON SYNTAX` that loads a source package containing `::constant k (self)`, a class `DEFAULTNAME` raising 98.900, and `::options trace r` therefore misses its trap. The frozen interpreter exits 158 without the handler output, while the oracle exits 0 and prints `caught 98.900` followed by `after`. Fold the rendering failure into the local evaluation result and execute the existing common cleanup before returning either outcome. Evidence: `close/review-probes/constant-result/`, including the exact program, command metadata, and separate stdout, stderr, and status files.

* **F2: Untraced non-SYNTAX RAISE renders ADDITIONAL array items unnecessarily.** At `rust/crates/rexx-exec/src/run/condition/raise.rs:153`, every populated ADDITIONAL slot is passed to fallible `string_value_text`, even though substitution strings are used only for SYNTAX and the condition already retains the original ADDITIONAL object. An untraced `raise user zz additional (.array~of(.bad~new)) return`, with `BAD~DEFAULTNAME` raising 98.900, consequently raises SYNTAX instead of completing the USER raise. The frozen interpreter prints `caught 98.900`, while the oracle prints `after`, both exiting 0. Build substitution strings only where the condition requires them, and preserve actual trace rendering separately, including the sibling ARRAY arm's equivalent distinction. Evidence: `close/review-probes/raise-additional/`; the existing `raise_untraced_renders_nothing.rex` checks a bare object and does not cover an array containing one.

## Responsibility passes

* Parser and package installation: declaration ordering, partial scanning and error precedence, clause splitting, class dependency order, compiled-image routine names, resolver paths, constant execution, and security-manager retention.
* Numeric and conversion behavior: MAX/MIN argument conversion and LOSTDIGITS, DATATYPE W, integral powers, representation-independent conversions, and traced rendering failures.
* Conditions and execution: delayed traps, RAISE operands and propagation, TRACE destination failure delivery, internal and native frames, USE ARG aliases, assignment paths, and IR call-site retention.
* Objects and collections: class dictionaries and method identities, protected/private flags, executable identity storage, array indexes and sorting, hash callback traversal, selected-store lifetime, and removal behavior.
* Streams and scheduling: OpenFile buffering, standard-output aliases and metadata, transient reads, persistent stdin positions, command worker reservation, token-directed completion, native spawn failure, and activity roots.
* Gate and witness boundaries: suite denominator checks, empty group selection rejection, pure-Rexx extension ownership, refusal and Phase 9 labels, process input/output deadlines, and the existing regression fixtures adjacent to the two findings.

These were focused responsibility passes over the whole phase range, not a claim that every generated fixture or every unchanged function was inspected. The supplied companion excludes generated source-line goldens and raw measurement records, whose audits remain separate. No build, full test run, profile, mutation experiment, index change, or HEAD change was performed by this reviewer. Two concrete source doubts received bounded paired probes against the existing frozen release pin, whose SHA-256 and invocation details are retained with each probe. Only this report and those review artifacts were written.

## Prior review and retained decisions

The earlier `t8.8/final-review/review.md` covers Task 8.8, not the complete Phase 9d range. Its original stdout POSITION, buffered-alias position, and HASITEM storage findings were addressed in `t8.8/final-review/rereview.md`, and the resulting closed-alias EXISTS issue was addressed in `t8.8/final-review/r1-rereview.md`. The current query guards, retained alias buffering, and rooted HASITEM selected arrays were inspected without reopening those findings. The accepted performance residuals, Deviations 82 and 83, and Deviation 84 remain governed by their recorded user rulings. Neither finding above requests a new semantic departure.

## Considered behaviors declined to judge

* Fresh Task 10 gate outcomes and final closure records: owned by the controller and not established by this source review.
* Performance residual acceptance and historical stderr-queue diagnosis: retained user dispositions, with no new measurement or cause claim here.
* `defineMethods` bypassing an overridden SUPPLIER: a known queued gap separately identified by the close audit, now explicitly authorized for repair by the user, not a new review finding.
* Exhaustive reentrant HASHCODE behavior outside the inspected changed traversal paths and arbitrary descendant-process lifetime policy: no additional high-confidence phase regression established.

Phase 9d is not ready to close on this source verdict. The two confirmed failures need focused regression coverage and fixes before fresh closing gates can establish readiness. The known SUPPLIER gap is a separate authorized repair in the controller's consolidated work. This report supplies the complete new finding list for that repair wave.
