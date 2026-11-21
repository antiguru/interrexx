//! Every number here was captured from `build/bin/rexxc`, one minimal program
//! per error, before any of this was written. Twenty distinct errors are in
//! scope and eighteen are reachable. The two that are not have their own test
//! recording what the oracle answers instead.

use crate::ast::{DirectiveKind, EndStyle, InstructionKind};
use crate::token::ParseError;
use crate::{Program, ProgramSource, SourceKind, parse_program};

fn parse(text: &str) -> Result<Program, ParseError> {
    parse_program(text.as_bytes().to_vec())
}

/// The program `text` builds, which must parse.
fn ok(text: &str) -> Program {
    parse(text).unwrap_or_else(|e| panic!("{text:?} failed to parse: {e:?}"))
}

/// The error `text` raises, as `(code, sub)`.
fn err(text: &str) -> (u16, u16) {
    match parse(text) {
        Ok(_) => panic!("{text:?} parsed but an error was expected"),
        Err(e) => (e.code, e.sub),
    }
}

/// The error `text` raises and the line it is reported on, as
/// `(code, sub, line)`.
fn err_at(text: &str) -> (u16, u16, usize) {
    let source = ProgramSource::new(text.as_bytes().to_vec(), SourceKind::Program);
    match parse(text) {
        Ok(_) => panic!("{text:?} parsed but an error was expected"),
        Err(e) => (e.code, e.sub, source.line_of(e.byte)),
    }
}

/// The keyword of each instruction of the main body.
fn names(program: &Program) -> Vec<&'static str> {
    program
        .main
        .instructions
        .iter()
        .map(|i| i.kind.keyword().unwrap_or("<other>"))
        .collect()
}

// ---- errors 7.1 and 7.2, from matchEnd and the membership check ----

#[test]
fn a_select_with_no_when_is_7_1_reported_against_the_select() {
    // Measured: the reported line is 3, the SELECT's, with the END on line 5.
    // This is the ONE error in the family that reports against the construct
    // rather than the offending clause, because the oracle passes the
    // construct's location where every neighbour passes the END's location.
    assert_eq!(err_at("nop\n\nselect\n\nend\n"), (7, 1, 3));
    // `SELECT CASE` counts the same way, and an OTHERWISE does not stand in for
    // a WHEN. Measured, reported on the SELECT's line again.
    assert_eq!(
        err_at("nop\n\nselect case 1\n\notherwise nop\n\nend\n"),
        (7, 1, 3)
    );
}

#[test]
fn a_non_when_directly_inside_a_select_is_7_2() {
    // Measured: reported on line 5, the offending clause, with line 3, the
    // SELECT's, as a substitution. The two lines differ, which is why the blanks are
    // here.
    assert_eq!(err_at("nop\n\nselect\n\nnop\n\nend\n"), (7, 2, 5));
    // An IF and a DO are control instructions, which reach the check by the
    // other branch of the add/flush split, so both directions of that split are
    // covered. Measured for each.
    assert_eq!(err("nop\n\nselect\n\nif 1 = 1 then nop\n\nend\n"), (7, 2));
    assert_eq!(err("nop\n\nselect\n\ndo\n\nend\n\nend\n"), (7, 2));
    assert_eq!(err("nop\n\nselect case 1\n\nnop\n\nend\n"), (7, 2));
    // An ELSE reaches the membership check before its own 8.2, so a SELECT
    // holding a bare ELSE is 7.2 and not 8.2. Measured both, and the pair is
    // what pins the ordering.
    assert_eq!(err("nop\n\nselect\n\nelse nop\n\nend\n"), (7, 2));
    assert_eq!(
        err("nop\n\nselect\n\nwhen 1 = 1 then nop\n\nelse nop\n\nend\n"),
        (8, 2)
    );
}

// ---- error 8.2 ----

#[test]
fn an_else_with_no_then_above_it_is_8_2() {
    assert_eq!(err_at("nop\n\nelse nop\n"), (8, 2, 3));
    // An ELSE separated from its IF by another instruction: the branch end was
    // already flushed, so there is nothing left to attach to. Measured.
    assert_eq!(err("nop\n\nif 1 = 1 then nop\n\nnop\n\nelse\n"), (8, 2));
}

// ---- errors 9.1 and 9.2 ----

#[test]
fn a_when_outside_a_select_is_9_1() {
    // Raised from the control stack, which is why it precedes every check the
    // assembler makes. Measured on line 3.
    assert_eq!(err_at("nop\n\nwhen 1 = 1 then nop\n"), (9, 1, 3));
    // A DO between the WHEN and the SELECT: the control-stack search stops at
    // the DO, because a DO is a block. Measured 9.1.
    assert_eq!(err("nop\n\ndo\n\nwhen 1 = 1 then nop\n\nend\n"), (9, 1));
    assert_eq!(
        err("nop\n\nselect\n\nwhen 1 = 1 then\n\ndo\n\nwhen 2 = 2 then nop\n\nend\n\nend\n"),
        (9, 1)
    );
    // An OTHERWISE is a block too, so a WHEN after one no longer finds the
    // SELECT. Measured 9.1 for both SELECT spellings.
    assert_eq!(
        err(
            "nop\n\nselect\n\nwhen 1 = 1 then nop\n\notherwise nop\n\nwhen 2 = 2 then nop\n\nend\n"
        ),
        (9, 1)
    );
    assert_eq!(
        err("nop\n\nselect case 1\n\nwhen 1 then nop\n\notherwise nop\n\nwhen 2 then nop\n\nend\n"),
        (9, 1)
    );
}

#[test]
fn an_otherwise_outside_a_select_is_9_2() {
    assert_eq!(err_at("nop\n\notherwise nop\n"), (9, 2, 3));
    assert_eq!(err("nop\n\ndo\n\notherwise nop\n\nend\n"), (9, 2));
}

// ---- error 10.x, the END family ----

#[test]
fn an_end_with_no_open_block_is_10_1() {
    assert_eq!(err_at("nop\n\nend\n"), (10, 1, 3));
}

#[test]
fn an_end_right_after_a_then_is_10_5_and_after_an_else_10_6() {
    // `r[instr.if.syntax]`: the END is refused as the branch's instruction,
    // on its own line, before the branch is flushed.
    for (source, line) in [
        ("nop\n\nif 1 = 1 then\n\nend\n", 5),
        ("nop\n\nif 1 = 1 then end\n", 3),
        ("nop\n\nselect\n\nwhen 1 = 1 then\n\nend\n", 7),
        ("nop\n\nselect\n\nwhen 1 = 1 then end\n", 5),
    ] {
        assert_eq!(err_at(source), (10, 5, line), "{source:?}");
    }
    for (source, line) in [
        ("nop\n\nif 1 = 1 then nop\n\nelse\n\nend\n", 7),
        ("nop\n\nif 1 = 1 then nop\n\nelse end\n", 5),
        // Inside an outer DO, the END would otherwise close the DO and
        // leave the ELSE with no instruction to take.
        ("nop\n\ndo\n\nif 1 = 1 then nop\n\nelse\n\nend\n\nend\n", 9),
        ("do; if 0 then nop; else; end\n", 1),
    ] {
        assert_eq!(err_at(source), (10, 6, line), "{source:?}");
    }
}

#[test]
fn a_mismatched_end_name_picks_its_number_from_what_it_failed_to_close() {
    // Four numbers from one condition, and the symbolic names do not say which
    // is which, so each was captured rather than derived. All reported against
    // the END, which is line 7 in each of these.
    assert_eq!(err_at("nop\n\ndo label a\n\nnop\n\nend b\n"), (10, 2, 7));
    assert_eq!(err_at("nop\n\ndo i = 1 to 3\n\nnop\n\nend j\n"), (10, 2, 7));
    // An unnamed block and any name at all.
    assert_eq!(err_at("nop\n\ndo\n\nnop\n\nend 1\n"), (10, 3, 7));
    assert_eq!(err("nop\n\nloop forever\n\nnop\n\nend 1\n"), (10, 3));
    assert_eq!(err("nop\n\ndo 3\n\nnop\n\nend 1\n"), (10, 3));
    // The same two cases under a SELECT, which has its own pair of numbers.
    assert_eq!(
        err_at("nop\n\nselect label a\n\nwhen 1 = 1 then nop\n\nend b\n"),
        (10, 4, 7)
    );
    assert_eq!(
        err_at("nop\n\nselect\n\nwhen 1 = 1 then nop\n\nend 1\n"),
        (10, 7, 7)
    );
    assert_eq!(
        err("nop\n\nselect case 1\n\nwhen 1 then nop\n\nend 1\n"),
        (10, 7)
    );
    // An OTHERWISE does not change which block the name is matched against:
    // measured 10.4 against the SELECT's label, not a number of its own.
    assert_eq!(
        err("nop\n\nselect label a\n\nwhen 1 = 1 then nop\n\notherwise nop\n\nend b\n"),
        (10, 4)
    );
}

#[test]
fn a_matching_or_omitted_end_name_is_accepted() {
    // The other direction of every arm above. Measured rc 0 for each.
    ok("nop\n\ndo label a\n\nnop\n\nend a\n");
    ok("nop\n\ndo label a\n\nnop\n\nend\n");
    // The control variable becomes the label when there is no LABEL clause.
    ok("nop\n\ndo i = 1 to 3\n\nnop\n\nend i\n");
    ok("nop\n\nselect label a\n\nwhen 1 = 1 then nop\n\nend a\n");
    ok("nop\n\nselect label a\n\nwhen 1 = 1 then nop\n\notherwise nop\n\nend a\n");
    // A name may be any symbol, including one that is a keyword elsewhere and
    // one that is a number. Measured rc 0 for all four spellings.
    for name in ["1", "loop", "a.", "a.1"] {
        ok(&format!("do label {name}\nnop\nend {name}\n"));
    }
}

// ---- error 14.x, from blockError ----

#[test]
fn an_unclosed_block_picks_its_number_from_the_block_kind() {
    // Six numbers, four of which the brief's list of thirteen omitted, all
    // captured. The reported line is the LAST INSTRUCTION's and not the block's,
    // which the blanks separate: here the DO is on line 3 and the last `nop` on
    // line 5.
    assert_eq!(err_at("nop\n\ndo\n\nnop\n"), (14, 1, 5));
    // A LABEL does not move a plain DO to the loop number, which is worth
    // pinning because the oracle's end style DOES distinguish it. Measured 14.1.
    assert_eq!(err("nop\n\ndo label a\n\nnop\n"), (14, 1));
    // Every other DO/LOOP form is one number, however it is spelled.
    for header in ["do while 1", "do 3", "loop", "loop forever", "do i over x"] {
        let source = format!("nop\n\n{header}\n\nnop\n");
        assert_eq!(err(&source), (14, 5), "{source:?}");
    }
    // Both SELECT spellings share one number.
    assert_eq!(err("nop\n\nselect\n\nwhen 1 = 1 then nop\n"), (14, 2));
    assert_eq!(err("nop\n\nselect case 1\n\nwhen 1 then nop\n"), (14, 2));
    // An OTHERWISE has its own, and its sub-number is 901 rather than the 6 that
    // would follow from its position in the table. Measured, not derived.
    assert_eq!(
        err("nop\n\nselect\n\nwhen 1 = 1 then nop\n\notherwise nop\n"),
        (14, 901)
    );
    assert_eq!(
        err("nop\n\nselect\n\nwhen 1 = 1 then nop\n\notherwise\n"),
        (14, 901)
    );
}

#[test]
fn a_then_or_else_with_nothing_after_it_is_14_3_or_14_4() {
    // Reported on the THEN's own line, which is 3 when it shares the IF's line.
    assert_eq!(err_at("nop\n\nif 1 = 1 then\n"), (14, 3, 3));
    // And on line 5 when the THEN is a clause of its own, which is what shows
    // the reported line is the THEN's rather than the IF's. Measured: line 5
    // reported, line 3 substituted.
    assert_eq!(err_at("nop\n\nif 1 = 1\n\nthen\n"), (14, 3, 5));
    // A WHEN's THEN shares the number with an IF's.
    assert_eq!(err("nop\n\nselect\n\nwhen 1 = 1 then\n"), (14, 3));
    assert_eq!(err_at("nop\n\nif 1 = 1 then nop\n\nelse\n"), (14, 4, 5));
    // A directive ends the body just as end of file does, so the same shapes
    // give the same numbers with a `::` clause after them. Measured all three.
    assert_eq!(err("nop\n\nif 1 = 1 then\n\n::routine r\n"), (14, 3));
    // 18.1 with a directive ending the body is reported against the DIRECTIVE's
    // line, not the IF's, because reading the next clause succeeded on the `::`
    // clause and moved the clause location there. Measured: line 5 reported,
    // line 3 substituted. At plain end of file the two coincide, which is the
    // row below.
    assert_eq!(err_at("nop\n\nif 1 = 1\n\n::routine r\nnop\n"), (18, 1, 5));
    assert_eq!(
        err_at("nop\n\nselect\n\nwhen 1 = 1\n\n::routine r\nnop\n"),
        (18, 2, 7)
    );
    assert_eq!(err_at("nop\n\nnop\n\nif 1 = 1\n"), (18, 1, 5));
    assert_eq!(err("nop\n\ndo\n\n::routine r\n"), (14, 1));
}

#[test]
fn the_last_instruction_is_what_an_unclosed_block_is_reported_against() {
    // Three instructions inside the DO, so the reported line can only be the
    // last one's. Measured line 7, with the DO on line 1 as the substitution.
    assert_eq!(err_at("do\n\nnop\n\nnop\n\nnop\n"), (14, 1, 7));
}

// ---- error 47.x, misplaced labels ----

#[test]
fn a_label_inside_a_block_picks_its_number_from_the_block_kind() {
    // Reported on the label's own line, 5 in each of these.
    assert_eq!(err_at("nop\n\ndo\n\nlab:\n\nnop\n\nend\n"), (47, 2, 5));
    assert_eq!(err("nop\n\nloop forever\n\nlab:\n\nnop\n\nend\n"), (47, 2));
    assert_eq!(err_at("nop\n\nif 1 = 1 then\n\nlab:\n\nnop\n"), (47, 3, 5));
    assert_eq!(
        err_at("nop\n\nselect\n\nlab:\n\nwhen 1 = 1 then nop\n\nend\n"),
        (47, 4, 5)
    );
    // After a WHEN's branch has closed, the branch-end frame is still a SELECT
    // context, so a label there is 47.4 rather than allowed. This is the arm
    // that separates a WHEN's branch end from an IF's, which IS allowed.
    assert_eq!(
        err("nop\n\nselect\n\nwhen 1 = 1 then nop\n\nlab:\n\nwhen 2 = 2 then nop\n\nend\n"),
        (47, 4)
    );
    // Inside an OTHERWISE, which is a block of its own.
    assert_eq!(
        err("nop\n\nselect\n\nwhen 1 = 1 then nop\n\notherwise\n\nlab:\n\nnop\n\nend\n"),
        (47, 4)
    );
}

#[test]
fn a_label_is_allowed_after_a_finished_then_unless_an_else_follows() {
    // Both directions, and this pair is why an IF's branch-end frame is absent
    // from every arm of the label check. Measured rc 0 for the first.
    ok("nop\n\nif 1 = 1 then nop\n\nlab:\n\nnop\n");
    // With an ELSE after it the label is rejected, but by the ELSE and not by
    // the label check, so it is reported on the ELSE's line, 7, and not the
    // label's, 5. Measured, and the message substitutes the string "ELSE"
    // because the C++ has no label name to hand at that point.
    assert_eq!(
        err_at("nop\n\nif 1 = 1 then nop\n\nlab:\n\nelse nop\n"),
        (47, 3, 7)
    );
}

// ---- 99.907 and 99.910, the must-be-first checks ----

#[test]
fn expose_and_use_local_must_be_the_first_instruction_of_their_body() {
    // NOT method-specific, which was measured rather than assumed: the main
    // program answers 99.907 too, on line 3.
    assert_eq!(err_at("nop\n\nexpose a\n"), (99, 907, 3));
    assert_eq!(err_at("nop\n\nuse local a\n"), (99, 910, 3));
    assert_eq!(err("::method m\nnop\nexpose a\n"), (99, 907));
    assert_eq!(err("::method m\nnop\nuse local a\n"), (99, 910));
    // A label counts as an instruction, so one in front is enough to break it.
    // Measured 99.907, which is what makes `at_body_start` test the chain rather
    // than a count of non-label clauses.
    assert_eq!(err("::method m\nlab:\nexpose a\n"), (99, 907));
    // The other direction, all measured rc 0.
    ok("expose a\n\nnop\n");
    ok("use local a\n\nnop\n");
    ok("::method m\nexpose a\nnop\n");
    ok("::method m\nuse local a\nnop\n");
}

#[test]
fn each_body_gets_its_own_must_be_first_check() {
    // The main body already has an EXPOSE, and the method body may still start
    // with one, because the last-instruction record is per block translation.
    // Measured rc 0, and this is the test that would fail if bodies shared an
    // assembler.
    ok("expose a\n::method m\nexpose b\n");
    // And a body's own second instruction is still rejected. Measured 99.907.
    assert_eq!(err("expose a\n::method m\nnop\nexpose b\n"), (99, 907));
}

// ---- 99.913, the exposed-variable table ----

#[test]
fn a_guard_when_expression_must_name_an_exposed_variable() {
    // Raised while parsing the GUARD rather than by the assembler, which is why
    // it fires in the main program with no method anywhere. Measured 99.913 for
    // each.
    assert_eq!(err_at("guard on when 1\n"), (99, 913, 1));
    assert_eq!(err("::method m\nexpose a\nguard on when b\n"), (99, 913));
    assert_eq!(err("::method m\nguard on when a\n"), (99, 913));
    // GUARD OFF WHEN is checked the same way, which the symbolic name does not
    // say. Measured 99.913.
    assert_eq!(err("::method m\nguard off when a\n"), (99, 913));
    // The other direction.
    ok("::method m\nexpose a\nguard on when a\n");
    // One exposed name anywhere in the expression is enough, so the walk has to
    // reach into a subexpression. Measured rc 0.
    ok("::method m\nexpose a\nguard on when a & b\n");
    // GUARD with no WHEN has no expression to check at all. Measured rc 0.
    ok("::method m\nguard on\n");
    ok("::method m\nguard off\n");
}

/// What a `GUARD WHEN` watches: each exposed variable its expression names,
/// once, a compound contributing its stem and its variable tails where they
/// are exposed.
#[test]
fn a_guard_when_watches_the_exposed_variables_it_names() {
    let watched = |text: &str| match &ok(text).main.instructions[1].kind {
        InstructionKind::Guard(guard) => guard
            .watched
            .iter()
            .map(|name| String::from_utf8_lossy(name).into_owned())
            .collect::<Vec<_>>(),
        other => panic!("{text:?}: {other:?}"),
    };
    assert_eq!(
        watched("expose a b.\nguard on when a + b.c + q + a = 1\n"),
        ["A", "B."]
    );
    assert_eq!(watched("expose i\nguard off when x.i = 1\n"), ["I"]);
}

#[test]
fn use_local_inverts_the_exposure_rule_and_seeds_five_names() {
    // With a USE LOCAL, every name EXCEPT the listed ones is exposed, which is
    // the opposite of what an EXPOSE means. Measured rc 0.
    ok("::method m\nuse local b\nguard on when a\n");
    // A name that IS listed is not exposed. Measured 99.913.
    assert_eq!(err("::method m\nuse local a\nguard on when a\n"), (99, 913));
    // The five names seeded as local are local even though no USE LOCAL listed
    // them, which is only observable through this check. Measured 99.913 for
    // SELF, and the other four are seeded by the same statement.
    assert_eq!(
        err("::method m\nuse local a\nguard on when self\n"),
        (99, 913)
    );
}

/// The compound cache, which is the one place a LATER guard is decided by an
/// EARLIER instruction.
#[test]
fn a_compound_of_an_exposed_stem_feeds_a_guard_wherever_else_it_is_referenced() {
    // `r[instr.guard.when.exposed]`: what the rest of the body does with the
    // compound does not matter, in a name slot or a variable slot, before the
    // GUARD or after it, in the same clause or in another GUARD.
    for source in [
        "::method m\nexpose a.\nsay a.1\nguard on when a.1\n",
        "::method m\nexpose a.\nguard on when a.1\nguard on when a.1\n",
        "::method m\nexpose a.\nguard on when a.1 & a.1\n",
        "::method m\nexpose i\nsay a.i\nguard on when a.i\n",
        "::method m\nuse local x\nsay a.1\nguard on when a.1\n",
        "::method m\nexpose a.\ndo label a.1\nleave a.1\nend a.1\nguard on when a.1\n",
        "::method m\nexpose a.\nsignal on syntax name a.1\nguard on when a.1\n",
        "::method m\nexpose a.\ndrop a.1\nguard on when a.1\n",
        "::method m\nexpose a.\ndrop (a.1)\nguard on when a.1\n",
        "::method m\nexpose a.\nparse var a.1 x\nguard on when a.1\n",
        "::method m\nexpose a.\nparse value 1 with a.1\nguard on when a.1\n",
        "::method m\nexpose a.\nuse arg a.1\nguard on when a.1\n",
        "::method m\nexpose a.\ndo a.1 = 1 to 2\nnop\nend\nguard on when a.1\n",
        "::method m\nexpose a.\nnumeric digits a.1\nguard on when a.1\n",
        "::method m\nexpose a.\ninterpret a.1\nguard on when a.1\n",
        "::method m\nexpose a.\na.1 = 1\nguard on when a.1\n",
        "::method m\nexpose a.\nif a.1 = 1 then nop\nguard on when a.1\n",
        "::method m\nexpose a.\na.1\nguard on when a.1\n",
        "::method m\nexpose a.\na.1~string\nguard on when a.1\n",
    ] {
        ok(source);
    }
    // Exposing the compound by its own name is still not enough.
    assert_eq!(
        err("::method m\nexpose a.1\nguard on when a.1\n"),
        (99, 913)
    );
}

#[test]
fn a_compound_guard_reference_contributes_its_stem_and_its_tail_pieces() {
    // The stem of `A.1` is `A.` and not `A`, so which of the two is exposed
    // decides the answer. Both directions measured, and this pair is what makes
    // the trailing period load bearing.
    ok("::method m\nexpose a.\nguard on when a.1\n");
    assert_eq!(err("::method m\nexpose a\nguard on when a.1\n"), (99, 913));
    // A tail piece that is a variable is looked up too, so exposing the tail is
    // also enough.
    ok("::method m\nexpose i\nguard on when a.i\n");
}

#[test]
fn the_indirect_expose_form_does_not_expose_the_name() {
    // `EXPOSE (a)` takes a different path in the variable-list parse and never
    // exposes nothing, so the name is not exposed as far as this check goes.
    // Measured 99.913, which is a quirk rather than a rule anyone would guess.
    assert_eq!(err("::method m\nexpose (a)\nguard on when a\n"), (99, 913));
}

// ---- the chain and the jump targets ----

#[test]
fn an_if_with_no_else_skips_to_the_instruction_after_its_branch() {
    // `nop`(0) `if`(1) `then`(2) `say`(3) `nop`(4).
    let program = ok("nop\nif 1 = 1 then say 1\nnop\n");
    assert_eq!(names(&program), ["NOP", "IF", "THEN", "SAY", "NOP"]);
    match &program.main.instructions[1].kind {
        InstructionKind::If { false_target, .. } => assert_eq!(*false_target, Some(4)),
        other => panic!("expected an IF, got {other:?}"),
    }
}

#[test]
fn an_if_at_the_end_of_a_body_has_no_false_target() {
    // Nothing follows the branch, so control falls out of the body and the
    // target is `None` rather than an index one past the end.
    let program = ok("if 1 = 1 then say 1\n");
    assert_eq!(names(&program), ["IF", "THEN", "SAY"]);
    match &program.main.instructions[0].kind {
        InstructionKind::If { false_target, .. } => assert_eq!(*false_target, None),
        other => panic!("expected an IF, got {other:?}"),
    }
}

#[test]
fn an_if_with_an_else_sends_its_false_path_to_the_else() {
    // `if`(0) `then`(1) `say`(2) `else`(3) `say`(4) `nop`(5): the false path
    // goes to the ELSE, and the true path resumes after the ELSE's branch.
    let program = ok("if 1 = 1 then say 1\nelse say 2\nnop\n");
    assert_eq!(names(&program), ["IF", "THEN", "SAY", "ELSE", "SAY", "NOP"]);
    match &program.main.instructions[0].kind {
        InstructionKind::If { false_target, .. } => assert_eq!(*false_target, Some(3)),
        other => panic!("expected an IF, got {other:?}"),
    }
    match &program.main.instructions[3].kind {
        InstructionKind::Else { then_exit } => assert_eq!(*then_exit, Some(5)),
        other => panic!("expected an ELSE, got {other:?}"),
    }
}

#[test]
fn a_dangling_else_binds_to_the_inner_if() {
    // `if`(0) `then`(1) `if`(2) `then`(3) `say`(4) `else`(5) `say`(6) `nop`(7).
    // The inner IF's false path is the ELSE; the outer IF's is past everything,
    // because the ELSE belongs to the inner one.
    let program = ok("if 1 = 1 then if 2 = 2 then say 1\nelse say 2\nnop\n");
    assert_eq!(
        names(&program),
        ["IF", "THEN", "IF", "THEN", "SAY", "ELSE", "SAY", "NOP"]
    );
    match &program.main.instructions[2].kind {
        InstructionKind::If { false_target, .. } => assert_eq!(*false_target, Some(5)),
        other => panic!("expected the inner IF, got {other:?}"),
    }
    match &program.main.instructions[0].kind {
        InstructionKind::If { false_target, .. } => assert_eq!(*false_target, Some(7)),
        other => panic!("expected the outer IF, got {other:?}"),
    }
    match &program.main.instructions[5].kind {
        InstructionKind::Else { then_exit } => assert_eq!(*then_exit, Some(7)),
        other => panic!("expected an ELSE, got {other:?}"),
    }
}

#[test]
fn an_else_branch_that_is_a_block_ends_at_its_own_end() {
    // `if`(0) `then`(1) `say`(2) `else`(3) `do`(4) `say`(5) `end`(6) `nop`(7).
    // The DO is a control instruction, so the ELSE frame survives until the END
    // closes the block, and only then does the THEN branch learn where to
    // resume.
    let program = ok("if 1 = 1 then say 1\nelse do\nsay 2\nend\nnop\n");
    assert_eq!(
        names(&program),
        ["IF", "THEN", "SAY", "ELSE", "DO", "SAY", "END", "NOP"]
    );
    match &program.main.instructions[3].kind {
        InstructionKind::Else { then_exit } => assert_eq!(*then_exit, Some(7)),
        other => panic!("expected an ELSE, got {other:?}"),
    }
}

#[test]
fn a_whens_false_path_is_the_next_when_and_its_exit_is_past_the_end() {
    // `select`(0) `when`(1) `then`(2) `say`(3) `when`(4) `then`(5) `say`(6)
    // `end`(7) `nop`(8).
    let program = ok("select\nwhen 1 = 1 then say 1\nwhen 2 = 2 then say 2\nend\nnop\n");
    assert_eq!(
        names(&program),
        [
            "SELECT", "WHEN", "THEN", "SAY", "WHEN", "THEN", "SAY", "END", "NOP"
        ]
    );
    match &program.main.instructions[1].kind {
        InstructionKind::When {
            false_target, exit, ..
        } => {
            assert_eq!(*false_target, Some(4));
            assert_eq!(*exit, Some(8));
        }
        other => panic!("expected the first WHEN, got {other:?}"),
    }
    // The last WHEN falls through to the END when it is false, and leaves past
    // the END when it is true.
    match &program.main.instructions[4].kind {
        InstructionKind::When {
            false_target, exit, ..
        } => {
            assert_eq!(*false_target, Some(7));
            assert_eq!(*exit, Some(8));
        }
        other => panic!("expected the second WHEN, got {other:?}"),
    }
    // And the SELECT knows all of it.
    match &program.main.instructions[0].kind {
        InstructionKind::Select {
            whens,
            otherwise,
            end,
            ..
        } => {
            assert_eq!(whens, &[1, 4]);
            assert_eq!(*otherwise, None);
            assert_eq!(*end, Some(7));
        }
        other => panic!("expected a SELECT, got {other:?}"),
    }
}

#[test]
fn a_whens_false_path_is_the_otherwise_when_there_is_one() {
    // `select`(0) `when`(1) `then`(2) `say`(3) `otherwise`(4) `say`(5) `end`(6).
    let program = ok("select\nwhen 1 = 1 then say 1\notherwise say 2\nend\n");
    assert_eq!(
        names(&program),
        ["SELECT", "WHEN", "THEN", "SAY", "OTHERWISE", "SAY", "END"]
    );
    match &program.main.instructions[1].kind {
        InstructionKind::When { false_target, .. } => assert_eq!(*false_target, Some(4)),
        other => panic!("expected a WHEN, got {other:?}"),
    }
    match &program.main.instructions[0].kind {
        InstructionKind::Select { otherwise, end, .. } => {
            assert_eq!(*otherwise, Some(4));
            assert_eq!(*end, Some(6));
        }
        other => panic!("expected a SELECT, got {other:?}"),
    }
}

#[test]
fn a_when_taken_as_another_whens_then_instruction_belongs_to_no_select() {
    // `r[instr.select.syntax]`: on the WHEN's own line, with the first WHEN's
    // condition true or false.
    assert_eq!(
        err_at("select\nwhen 1 = 1 then\nwhen 2 = 2 then nop\nend\n"),
        (9, 1, 3)
    );
    assert_eq!(
        err_at("select\nwhen 1 = 0 then\nwhen 2 = 2 then nop\nend\n"),
        (9, 1, 3)
    );
}

#[test]
fn a_block_and_its_end_point_at_each_other() {
    // `nop`(0) `do`(1) `nop`(2) `end`(3) `nop`(4). The DO is deliberately NOT
    // the first instruction: with the block at index 0 every assertion here
    // would still hold if the index were hard-wired to zero.
    let program = ok("nop\ndo\nnop\nend\nnop\n");
    assert_eq!(names(&program), ["NOP", "DO", "NOP", "END", "NOP"]);
    match &program.main.instructions[1].kind {
        InstructionKind::Do(body) => assert_eq!(body.end, Some(3)),
        other => panic!("expected a DO, got {other:?}"),
    }
    match &program.main.instructions[3].kind {
        InstructionKind::End { closes, .. } => {
            let closes = closes.expect("a matched END knows what it closed");
            assert_eq!(closes.block, 1);
            assert_eq!(closes.style, EndStyle::Do);
        }
        other => panic!("expected an END, got {other:?}"),
    }
    // And a nested pair, so the inner END is matched against the inner block
    // rather than the outer one. `do`(0) `do`(1) `nop`(2) `end`(3) `end`(4).
    let nested = ok("do\ndo\nnop\nend\nend\n");
    for (end, block) in [(3, 1), (4, 0)] {
        match &nested.main.instructions[end].kind {
            InstructionKind::End { closes, .. } => {
                assert_eq!(closes.expect("matched").block, block, "END at {end}");
            }
            other => panic!("expected an END, got {other:?}"),
        }
    }
}

#[test]
fn the_end_style_follows_the_block_it_closed() {
    // All six styles this parser can produce, each from the shape that produces
    // it. The C++ enum's seventh, `LABELED_SELECT_BLOCK`, is set by nothing, and
    // the last two rows are what pin that a label does not reach it.
    for (source, style) in [
        ("do\nnop\nend\n", EndStyle::Do),
        ("do label a\nnop\nend\n", EndStyle::LabeledDo),
        // Every loop form is one style, and a LABEL does not change it, which is
        // the opposite of the block form's rule.
        ("do i = 1 to 3\nnop\nend\n", EndStyle::Loop),
        ("do label a while 1\nnop\nend\n", EndStyle::Loop),
        ("loop forever\nleave\nend\n", EndStyle::Loop),
        ("select\nwhen 1 = 1 then nop\nend\n", EndStyle::Select),
        (
            "select\nwhen 1 = 1 then nop\notherwise nop\nend\n",
            EndStyle::Otherwise,
        ),
        (
            "select label a\nwhen 1 = 1 then nop\notherwise nop\nend\n",
            EndStyle::LabeledOtherwise,
        ),
        // A labelled SELECT with no OTHERWISE is still the plain SELECT style.
        (
            "select label a\nwhen 1 = 1 then nop\nend\n",
            EndStyle::Select,
        ),
    ] {
        let program = ok(source);
        let end = program
            .main
            .instructions
            .iter()
            .find_map(|i| match &i.kind {
                InstructionKind::End { closes, .. } => *closes,
                _ => None,
            })
            .unwrap_or_else(|| panic!("{source:?} has no matched END"));
        assert_eq!(end.style, style, "{source:?}");
    }
}

#[test]
fn an_end_on_an_otherwise_closes_the_select_behind_it() {
    // `select`(0) `when`(1) `then`(2) `nop`(3) `otherwise`(4) `nop`(5) `end`(6).
    // The END closes the SELECT and not the OTHERWISE, even though the OTHERWISE
    // was on top of the stack.
    let program = ok("select\nwhen 1 = 1 then nop\notherwise nop\nend\n");
    match &program.main.instructions[6].kind {
        InstructionKind::End { closes, .. } => {
            let closes = closes.expect("a matched END knows what it closed");
            assert_eq!(closes.block, 0);
            assert_eq!(closes.style, EndStyle::Otherwise);
        }
        other => panic!("expected an END, got {other:?}"),
    }
}

#[test]
fn a_block_inside_a_then_still_completes_the_pending_branch() {
    // `if`(0) `then`(1) `do`(2) `say`(3) `end`(4) `nop`(5). The DO is control so
    // the branch stays pending until the END closes it, which is what the
    // control flush at the end of the END arm is for.
    let program = ok("if 1 = 1 then do\nsay 1\nend\nnop\n");
    assert_eq!(names(&program), ["IF", "THEN", "DO", "SAY", "END", "NOP"]);
    match &program.main.instructions[0].kind {
        InstructionKind::If { false_target, .. } => assert_eq!(*false_target, Some(5)),
        other => panic!("expected an IF, got {other:?}"),
    }
}

// ---- labels, per body ----

#[test]
fn each_body_keeps_its_own_label_table() {
    let program = ok("top:\nnop\n::routine r\ntop:\nnop\n");
    // The main body's own label, at index 0 of the main body.
    assert_eq!(program.main.labels.get(b"TOP".as_slice()), Some(&0));
    match &program.directives[0].kind {
        DirectiveKind::Routine(routine) => {
            let body = routine.body.as_ref().expect("a routine body");
            // The same spelling, index 0 of a DIFFERENT chain. If the two bodies
            // shared a table the second occurrence would have been dropped,
            // because the first one wins.
            assert_eq!(body.labels.get(b"TOP".as_slice()), Some(&0));
            assert_eq!(body.instructions.len(), 2);
        }
        other => panic!("expected a routine, got {other:?}"),
    }
}

#[test]
fn a_duplicated_label_keeps_the_first() {
    // Measured, and accepted: two labels spelled `a:` is rc 0 and `signal a`
    // reaches the first.
    let program = ok("a:\nnop\na:\nnop\n");
    assert_eq!(program.main.labels.get(b"A".as_slice()), Some(&0));
}

// ---- directive bodies get their own control stack ----

#[test]
fn a_block_cannot_be_closed_across_a_directive() {
    // The DO belongs to the main body and the END to the routine's, so the DO is
    // unclosed. Measured 14.1, which is what proves the two bodies do not share
    // a stack.
    assert_eq!(err("do\nnop\n::routine r\nend\n"), (14, 1));
    // And the reverse: a body whose blocks balance is fine after another whose
    // blocks balance.
    ok("do\nnop\nend\n::routine r\ndo\nnop\nend\n");
}

#[test]
fn a_directive_body_is_assembled_and_kept() {
    let program = ok("nop\n::routine r\nif 1 = 1 then say 1\nnop\n");
    match &program.directives[0].kind {
        DirectiveKind::Routine(routine) => {
            let body = routine.body.as_ref().expect("a routine body");
            assert_eq!(
                body.instructions
                    .iter()
                    .map(|i| i.kind.keyword().unwrap_or("<other>"))
                    .collect::<Vec<_>>(),
                ["IF", "THEN", "SAY", "NOP"]
            );
            // An index inside a body indexes that body, so the IF's target is 3
            // and not an offset into the main chain.
            match &body.instructions[0].kind {
                InstructionKind::If { false_target, .. } => assert_eq!(*false_target, Some(3)),
                other => panic!("expected an IF, got {other:?}"),
            }
        }
        other => panic!("expected a routine, got {other:?}"),
    }
}

#[test]
fn an_external_routine_has_no_body_to_assemble() {
    // A shape whose body slot is `None` even though the kind can hold one, so
    // the assembler must not be handed the clauses after it. Measured: the parse
    // succeeds and the failure is 98.903 at run time, which is not a parse
    // error.
    let program = ok("nop\n::routine r external \"LIBRARY x\"\n");
    match &program.directives[0].kind {
        DirectiveKind::Routine(routine) => assert!(routine.body.is_none()),
        other => panic!("expected a routine, got {other:?}"),
    }
}

// ---- programs that must still be accepted ----

#[test]
fn the_shapes_the_oracle_accepts_are_accepted() {
    // Each measured rc 0. These are the sources whose block structure is legal
    // but unusual enough that a wrong stack would reject them.
    for source in [
        // A THEN on a line of its own, inside a SELECT: it must not reach the
        // membership check, which would reject it as a non-WHEN.
        "select\nwhen 1 = 1\nthen nop\nend\n",
        "do\nif 1 = 1\nthen nop\nend\n",
        // A nested SELECT inside a WHEN's branch.
        "select\nwhen 1 = 1 then\nselect\nwhen 2 = 2 then nop\nend\nend\n",
        // An OTHERWISE with its instruction on the same line, and on the next.
        "select\nwhen 1 = 1 then nop\notherwise nop\nend\n",
        "select\nwhen 1 = 1 then nop\notherwise\nnop\nend\n",
        // SELECT CASE with an OTHERWISE and more than one WHEN.
        "select case 1\nwhen 1 then nop\nwhen 2, 3 then nop\notherwise nop\nend\n",
        // Nested blocks, closed in order.
        "do\ndo\ndo\nnop\nend\nend\nend\n",
        // A label at the top level, which no open block forbids.
        "lab:\nnop\n",
    ] {
        ok(source);
    }
}

#[test]
fn catch_clauses_keep_their_owning_block_and_matchers() {
    use crate::ast::{CatchCode, ExprKind};

    let program = ok("do label attempt
  do
  catch syntax
  end
catch failure -2
catch errorInfo = syntax 42.3, error rcWanted, user failed
end attempt
select case 1
  when 1 then nop
  otherwise nop
catch any
end
loop 1
catch notready
end
");
    let mut handlers =
        program
            .main
            .instructions
            .iter()
            .filter_map(|instruction| match &instruction.kind {
                InstructionKind::Catch(handler) => Some(handler),
                _ => None,
            });
    assert_eq!(handlers.next().unwrap().block, Some(1));
    let failure = handlers.next().unwrap();
    assert_eq!(failure.block, Some(0));
    assert_eq!(failure.conditions[0].name.as_ref(), b"FAILURE");
    assert_eq!(
        failure.conditions[0].code,
        Some(CatchCode::Literal(b"-2".as_slice().into()))
    );

    let handler = handlers.next().unwrap();
    assert_eq!(handler.block, Some(0));
    assert!(
        matches!(handler.variable.as_ref().unwrap().kind, ExprKind::Variable(name)
        if program.symbols.name(name) == "ERRORINFO")
    );
    assert_eq!(handler.conditions[0].name.as_ref(), b"SYNTAX");
    assert_eq!(
        handler.conditions[0].code,
        Some(CatchCode::Literal(b"42.3".as_slice().into()))
    );
    assert!(
        matches!(handler.conditions[1].code, Some(CatchCode::Variable(name))
        if program.symbols.name(name) == "RCWANTED")
    );
    assert_eq!(handler.conditions[2].name.as_ref(), b"USER FAILED");

    for owner_keyword in ["SELECT", "LOOP"] {
        let handler = handlers.next().unwrap();
        assert_eq!(
            program.main.instructions[handler.block.unwrap()]
                .kind
                .keyword(),
            Some(owner_keyword)
        );
    }
    assert!(handlers.next().is_none());

    let image = crate::program_image(&program);
    let restored = crate::program_from_image(&image, program.source.parts().0)
        .expect("parsed image round-trips");
    assert_eq!(restored.main, program.main);
}

#[test]
fn catch_translation_errors_match_the_oracle() {
    let cases = [
        ("catch syntax", (10, 8)),
        ("do; if 1 then catch syntax; end", (10, 9)),
        ("do; if 1 then nop; else catch syntax; end", (10, 11)),
        ("do; catch; end", (20, 936)),
        ("do; catch any,; end", (21, 904)),
        ("do; catch user; end", (20, 915)),
        ("do; catch . = syntax; end", (31, 3)),
        ("do; catch potato; end", (25, 939)),
        ("do; catch syntax 42.1234; end", (35, 937)),
        ("do; catch syntax +42; end", (35, 1)),
        ("select; catch syntax; nop; end", (7, 1)),
        (
            "select; when 1 then nop; catch syntax; when 2 then nop; end",
            (9, 1),
        ),
        (
            "select; when 1 then nop; catch syntax; otherwise nop; end",
            (9, 2),
        ),
        ("do; catch syntax; x: nop; end", (47, 2)),
        ("do; catch syntax; nop", (14, 1)),
    ];
    for (source, expected) in cases {
        assert_eq!(err(source), expected, "{source}");
    }
}

#[test]
fn catch_can_still_name_a_variable_or_label() {
    assert_eq!(
        names(&ok("catch = 3; catch: say catch")),
        ["<other>", "<other>", "SAY"]
    );
}
