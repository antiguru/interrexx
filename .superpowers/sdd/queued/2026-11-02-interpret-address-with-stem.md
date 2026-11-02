**OPEN, queued 2026-11-02 by PR 0 of Phase 10 (crate defect, not caused by the oracle bump).**

# ADDRESS WITH set in INTERPRET names the fragment's symbols

`ooTest` r13347's `ADDRESS::test_with_interpret_sets_routine_config`. `Redirection::Stem(SymbolId)` is stored
in the activation's `io_configs` and read later against the body running the command, so an id from an
INTERPRET fragment's symbol table resolves in the wrong table. Measured: after
`interpret "address value address() with output stem b."`, `"echo x"` leaves `b.0` at 0 where the oracle has 1.
Exempted in `rust/corpus/keyword-exempt.txt`. The stored configuration needs names, or the fragment's code.
