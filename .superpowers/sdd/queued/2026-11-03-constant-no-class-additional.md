# 99.906 for a ::CONSTANT outside a class has no ADDITIONAL

Found by the Rexx Parser's `SyntaxErrors` suite (`TEST_ERROR_99_DIRECTIVES`), Phase 10 PR 5.

`.Package~new("x", "::Constant c (1+2)")` raises 99.906 on both sides with the same message. On the
oracle, `condition('o')~additional` is `("C")`: the constant's name, which the message text does not
use. Here it is empty, because `rust/crates/rexx-parse/src/directive.rs`
(`return Err(self.error(E99_906, []))`) raises with no insert, and `E99_906` is a `Msg<0>` in the
generated table.

The fix needs a way to carry an ADDITIONAL the message text does not use. Test: a corpus program
printing the condition's ADDITIONAL, with the oracle's output.
