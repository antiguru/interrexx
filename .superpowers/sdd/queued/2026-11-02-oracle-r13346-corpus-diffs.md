**OPEN, queued 2026-11-02 by PR 0 of Phase 10 (crate differences the r13346 oracle exposes; oracle diffs are reported, not gated).**

# Ten corpus programs disagree with the r13346 oracle

`REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` reports 10 of 1249: `lang/array_list_expression`
(`pair~at((1,))` is 98.975 on the oracle), `lang/array_index_refusals`, `lang/mutablebuffer_mutators`,
`lang/mutablebuffer_caseless`, `lang/library_string_argument`, `lang/library_callback_conditions`,
`lang/library_callback_strings`, `lang/trapped_condition_native_levels`, `lang/condition_object_origin`,
`lang/stream_boundary_report` (a condition object's PACKAGE and POSITION for a native boundary raise: the crate's
report line moved to the sending clause, the object's fields did not).
