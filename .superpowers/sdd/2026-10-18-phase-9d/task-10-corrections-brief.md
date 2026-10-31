# Task 10 consolidated close corrections

## Scope and authority

Finish the accepted SUPPLIER correction and the two Important findings in the final whole-phase review before freezing the close source. The binding report is `phase-9d-final-review.md` in this directory, with exact paired witnesses under `docs/superpowers/records/2026-10-18-phase-9d/close/review-probes/`. Moritz accepted the queued SUPPLIER item with "fix now." on 2026-10-31, recorded in Task 10 of the parent plan and the inventory. This is one consolidated correction wave, followed by a scoped review of these findings and fix-induced breakage.

## Required corrections

* F1: A fallible constant result rendering must return through normal constant-activation cleanup, restoring guards, call context, activations, roots and clause state on failure as on success. Witness a caller syntax trap handling DEFAULTNAME failure while installing a traced constant, then continuing correctly. Preserve successful traced and untraced constant behavior.
* F2: Non-SYNTAX RAISE retains its ADDITIONAL object without converting its array elements into catalogue substitution strings. Keep SYNTAX substitution conversion and genuine TRACE operand rendering separate. Cover both ADDITIONAL and ARRAY forms, untraced failure-sensitive elements and their adjacent traced/SYNTAX behavior as required by the actual contract. Preserve approved Deviation 83 DESCRIPTION conversion timing.
* SUPPLIER: Class~defineMethods must observe a collection subclass's SUPPLIER method, including user sends and security-manager observations inside that method. Derive the rule from reference documentation and implementation, and test adjacent plain collections and supplier-protocol objects. Inspect shared method_table_pairs callers before choosing the narrow layer, retain existing validation/error ordering and method installation atomicity, and cover reentrant/security interactions that the original Task 6.12 review identified. No new semantic departure may be silently introduced.

## Sequence and verification

1. Verify existing linked worktree and starting HEAD, inspect evidence and root causes, then make early pure-move responsibility splits in unwieldy touched files. Preserve every true comment and public surface. Keep move commits separate, proving extracted text identity before behavior changes. Avoid unrelated extractions.
2. Add meaningful regression tests and corpus witnesses with committed tests before their red execution. Use existing test helpers and run under required caps. A witness not covered by an existing program needs a sourceline oracle sidecar via its official generator and phase-9.txt registration. No ooTest edits.
3. Commit minimal implementation before executing its builds/verification. Run focused covering tests, policy/refusal/closed-label checks, formatting and touched-crate lint. Run actual paired oracle/crate witness probes and preserve separate stdout/stderr/status. The root's fresh full workspace, corpus, suite, TSan and all-program performance close runs follow the scoped review; do not redundantly execute them in this wave.
4. Re-derive refusal-sites.tsv if constructor source positions change, using the official refresh path and a verifying no-refresh run. Keep all dispositions and existing allowed sets unchanged except closing the accepted SUPPLIER queue/inventory when proof exists.
5. Write `task-10-corrections-report.md` here, with source commits, exact red/green commands/statuses, witness outputs, move proof, remaining limitations and current source identity. Durable evidence belongs under `docs/superpowers/records/2026-10-18-phase-9d/close/corrections/`. No running child remains at handoff.

## Environment and constraints

Use CARGO_INCREMENTAL=0, four Cargo jobs and the existing /tmp/codex-p9d-t88-target cache for focused checks. Default sandbox cannot create a memory cgroup: launch outer memcap through approved escalation, inherit normal environment and sanitize only interpreter children. Builds and tests memcap8G, interpreter memcap2G, each foreground command under ten minutes. Keep raw channels verbatim and output names exclusive, do not overwrite failures. No sudo/systemd-run/pkill or pgrep -f, git reset/checkout/add-A, pushes, new dependencies, unsafe or source edits outside this scope. Preserve the user's ZIP and previous caches/profiles.
