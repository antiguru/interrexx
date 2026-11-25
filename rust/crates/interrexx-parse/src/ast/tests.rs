//! Splitting a compound variable's name into its stem and tail pieces.

use super::{Tail, compound_parts};

#[test]
fn a_compound_name_splits_at_the_first_period_and_keeps_it_on_the_stem() {
    // Matches the oracle, which builds the stem as `start` up to and including
    // the period it stopped on.
    assert_eq!(
        compound_parts("A.B.C"),
        ("A.", vec![Tail::Variable("B"), Tail::Variable("C")])
    );
    assert_eq!(
        compound_parts("A.B.C.D"),
        (
            "A.",
            vec![
                Tail::Variable("B"),
                Tail::Variable("C"),
                Tail::Variable("D")
            ]
        )
    );
}

#[test]
fn a_tail_piece_that_cannot_be_a_variable_name_is_a_constant() {
    // Such a piece is empty or starts with a digit. Measured with
    // `build/bin/rexx` and `b = 2`: `say a.1.b` prints `A.1.2`, so `1` stood for
    // itself where `B` was looked up, and `say a..b` prints `A..2`.
    assert_eq!(
        compound_parts("A.1.B"),
        ("A.", vec![Tail::Constant("1"), Tail::Variable("B")])
    );
    assert_eq!(
        compound_parts("A..B"),
        ("A.", vec![Tail::Constant(""), Tail::Variable("B")])
    );
    // A trailing period gives a final empty piece rather than dropping one.
    assert_eq!(
        compound_parts("A.B."),
        ("A.", vec![Tail::Variable("B"), Tail::Constant("")])
    );
    // A digit anywhere but the front does not make a constant: `1B` cannot be
    // a symbol, but `B1` can.
    assert_eq!(compound_parts("A.B1"), ("A.", vec![Tail::Variable("B1")]));
}

#[test]
fn the_first_period_ends_the_stem_however_many_follow() {
    // A whole-name split would give the wrong stem for `A.B.C`: the stem is
    // `A.` and not `A.B.`.
    let (stem, tails) = compound_parts("STEM.I.J.K");
    assert_eq!(stem, "STEM.");
    assert_eq!(tails.len(), 3);
}
