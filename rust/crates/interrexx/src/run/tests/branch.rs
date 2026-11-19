use super::*;

// ---- LABEL and NOP ----

#[test]
fn a_label_is_a_traced_no_op() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"here: say 'hit'"),
        b"hit\n".to_vec(),
        "the label itself produces no output of its own"
    );
}

#[test]
fn nop_is_a_no_op() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"nop\nsay 'after'"),
        b"after\n".to_vec()
    );
}

// ---- IF/THEN/ELSE ----

/// The discriminating shape for a wrong `false_target`/`then_exit`: a
/// true condition whose `ELSE` branch has a **different** side effect
/// than the `THEN` branch. A version that lets the true path fall
/// through into the `ELSE` marker without skipping it (the naive
/// fallthrough this task's whole design exists to avoid -- see
/// `run_bounded`'s doc comment) runs *both* assignments and prints
/// `abXc`. A version that never runs the `ELSE` branch on the false path
/// at all prints `aXc` for the companion case below. Only the correct
/// wiring prints `abc` and `aXc` respectively.
#[test]
fn if_then_else_runs_exactly_one_branch() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"a = 'a'\nif 1 = 1 then a = a || 'b'\nelse a = a || 'X'\na = a || 'c'\nsay a"
        ),
        b"abc\n".to_vec(),
        "true: runs the THEN branch and skips the ELSE branch entirely"
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"a = 'a'\nif 1 = 0 then a = a || 'b'\nelse a = a || 'X'\na = a || 'c'\nsay a"
        ),
        b"aXc\n".to_vec(),
        "false: runs the ELSE branch and skips the THEN branch entirely"
    );
}

#[test]
fn if_then_with_no_else_falls_through_on_false() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"a = 'a'\nif 1 = 0 then a = a || 'b'\na = a || 'c'\nsay a"
        ),
        b"ac\n".to_vec()
    );
}

/// The `ast.rs`/`block.rs`-derived discriminating case: an `IF`/`ELSE
/// IF`/`ELSE` chain (no `DO`) where each link's
/// `false_target` is the next link's own condition and the whole
/// chain's resume point sits after all of them. Run once per branch so
/// a wrong `false_target` (skipping or re-testing a link) and a wrong
/// `then_exit`/resume (landing inside a later link, matching
/// `if_else_chain.rex`'s own framing) are both visible.
#[test]
fn if_else_if_chain_takes_exactly_one_link() {
    let source = |n: &str| -> Vec<u8> {
        format!(
            "n = {n}\na = ''\nif n = 1 then a = a || 'one'\nelse if n = 2 then a = a || 'two'\nelse if n = 3 then a = a || 'three'\nelse a = a || 'other'\na = a || '-after'\nsay a"
        )
        .into_bytes()
    };
    for (n, expected) in [
        ("1", "one-after\n"),
        ("2", "two-after\n"),
        ("3", "three-after\n"),
        ("4", "other-after\n"),
    ] {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, &source(n)),
            expected.as_bytes().to_vec(),
            "n = {n}"
        );
    }
}

#[test]
fn if_condition_that_is_not_0_or_1_raises_34_1() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"if 'x' then nop").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (34, 1));
    assert_eq!(raised.additional, vec![b"x".to_vec()]);
}

/// A comma list is 34.6 regardless of which element fails, never 34.1.
/// Measured against the oracle (brief's own transcript).
#[test]
fn if_condition_that_is_a_comma_list_raises_34_6_not_34_1() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"if 'x', 1 then nop").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (34, 6));
    assert_eq!(raised.additional, vec![b"x".to_vec()]);
}

#[test]
fn a_true_comma_list_condition_is_an_and_of_its_parts() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"if 1, 1 then say 'hit'\nelse say 'miss'"),
        b"hit\n".to_vec()
    );
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"if 1, 0 then say 'hit'\nelse say 'miss'"),
        b"miss\n".to_vec()
    );
}

// ---- SELECT / WHEN ----

#[test]
fn select_when_runs_exactly_the_first_matching_when() {
    let source = |n: &str| -> Vec<u8> {
        format!(
            "n = {n}\nr = ''\nselect\n  when n = 1 then r = r || 'one'\n  when n = 2 then r = r || 'two'\n  when n = 3 then r = r || 'three'\n  otherwise r = r || 'other'\nend\nr = r || '-after'\nsay r"
        )
        .into_bytes()
    };
    for (n, expected) in [
        ("1", "one-after\n"),
        ("2", "two-after\n"),
        ("3", "three-after\n"),
        ("4", "other-after\n"),
    ] {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, &source(n)),
            expected.as_bytes().to_vec(),
            "n = {n}"
        );
    }
}

/// The `select_when_bodies.rex` discriminating shape, reproduced without
/// `DO`: each `WHEN`'s consequence is a nested `SELECT`
/// spanning several flat instructions of its own
/// (`Select`/`When`/`Then`/two assignments/`End`), so a wrong exit that
/// lands even one instruction into a *later* `WHEN`'s span, rather than
/// cleanly past the whole outer `SELECT`, shows up as an extra or
/// missing accumulator update. Run for every `n` so a wrong exit wired
/// to (for instance) always resume after the *first* `WHEN` would still
/// be caught on `n = 2` or `n = 3`.
#[test]
fn select_when_wrong_exit_would_land_in_a_later_whens_multi_instruction_body() {
    let source = |n: &str| -> Vec<u8> {
        format!(
            "n = {n}\nr = ''\nselect\n  when n = 1 then select\n    when 1 = 1 then r = r || 'w1a'\n    otherwise nop\n  end\n  when n = 2 then select\n    when 1 = 1 then r = r || 'w2a'\n    otherwise nop\n  end\n  when n = 3 then select\n    when 1 = 1 then r = r || 'w3a'\n    otherwise nop\n  end\n  otherwise r = r || 'w4a'\nend\nr = r || '-done'\nsay r"
        )
        .into_bytes()
    };
    for (n, expected) in [
        ("1", "w1a-done\n"),
        ("2", "w2a-done\n"),
        ("3", "w3a-done\n"),
        ("4", "w4a-done\n"),
    ] {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, &source(n)),
            expected.as_bytes().to_vec(),
            "n = {n}: exactly one nested SELECT's body ran, and nothing else did"
        );
    }
}

#[test]
fn select_with_no_when_true_and_no_otherwise_raises_7_3() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"select\n  when 1 = 0 then nop\nend").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (7, 3));
    assert_eq!(raised.additional, Vec::<Vec<u8>>::new());
}

#[test]
fn select_with_no_when_true_and_an_otherwise_runs_it_without_error() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"select\n  when 1 = 0 then nop\n  otherwise say 'lo'\nend"
        ),
        b"lo\n".to_vec()
    );
}

#[test]
fn when_condition_that_is_not_0_or_1_raises_34_2() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"select\n  when 'x' then nop\nend").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (34, 2));
    assert_eq!(raised.additional, vec![b"x".to_vec()]);
}

/// **The coordinator's own finding, fixed after the first round.** A
/// `WHEN`'s own `step` arm is a pure no-op (`Select`'s own arm reads it
/// as data instead), so a raise while evaluating its *condition* never
/// went through a `Op::Clause`'s region call for the `WHEN` itself --
/// the first version of this task attributed it to the enclosing
/// `SELECT`, wrong clause *and* wrong line, measured against the
/// oracle. `record_failure_site`'s own doc comment on `Select`'s call
/// sites has the fix. Checks `failure_site` directly (line and clause
/// text), which `raised.number`/`.sub` alone -- every other test in
/// this file -- cannot: both were already unaffected by the bug, since
/// the bug is entirely in *which clause* gets echoed, not in which
/// condition failed.
#[test]
fn a_when_conditions_own_failure_is_attributed_to_the_when_not_the_select() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"select\nwhen 'x' then nop\nend").unwrap_err();
    let FailureSite::Clause { line, text, .. } = interp
        .activity
        .failure_site
        .expect("a raised condition always resolves a site when source is Some")
    else {
        panic!("not a clause site")
    };
    assert_eq!(line, 2, "the WHEN's own line, not the SELECT's (line 1)");
    assert_eq!(
        text,
        b"when 'x' ".to_vec(),
        "the WHEN's own clause text, not \"select\""
    );
}

/// The line has to move with the failing `WHEN`, not merely differ from
/// the `SELECT`'s -- a test whose expected line is the first `WHEN`'s
/// cannot tell a correct resolution from one that defaults to
/// "whichever `WHEN` this loop happens to be looking at first".
#[test]
fn the_second_of_two_whens_own_failure_moves_the_line_with_it() {
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"select\nwhen 1 = 0 then nop\nwhen 'x' then nop\nend",
    )
    .unwrap_err();
    let FailureSite::Clause { line, text, .. } =
        interp.activity.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(line, 3, "the second WHEN's own line, not the first's (2)");
    assert_eq!(text, b"when 'x' ".to_vec());
}

/// A `SELECT CASE` expression that itself raises is the `SELECT`'s own
/// clause -- confirmed against the oracle rather than assumed, since
/// `case` is evaluated directly inside `Select`'s own `step` call and
/// the coordinator asked this be checked, not taken on faith.
#[test]
fn a_select_cases_own_expression_failure_is_attributed_to_the_select() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"select case (1/0)\nwhen 1 then nop\nend").unwrap_err();
    let FailureSite::Clause { line, text, .. } =
        interp.activity.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(line, 1);
    assert_eq!(text, b"select case (1/0)".to_vec());
}

/// A `WhenCase` value expression that raises is the `WHEN`'s own
/// clause, the same rule as a plain `WHEN`'s condition -- both go
/// through `Select`'s own explicit-match-and-record path, never
/// through `Op::Clause`'s region for the `When`/`WhenCase` node itself.
#[test]
fn a_whencase_values_own_failure_is_attributed_to_the_when_not_the_select() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"select case 1\nwhen (1/0) then nop\nend").unwrap_err();
    let FailureSite::Clause { line, text, .. } =
        interp.activity.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(line, 2);
    assert_eq!(text, b"when (1/0) ".to_vec());
}

/// A raise inside an `OTHERWISE` branch was already correct before this
/// round's fix (`OTHERWISE`'s own body runs through the outer loop's
/// ordinary `Op::Clause`'s region, never through `Select`'s own
/// explicit-match path), and stays that way -- checked because the
/// coordinator asked for it explicitly, not assumed from the `WHEN` fix.
#[test]
fn a_raise_inside_an_otherwise_branch_is_attributed_to_its_own_clause() {
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"select\nwhen 1 = 0 then nop\notherwise\n  say 1/0\nend",
    )
    .unwrap_err();
    let FailureSite::Clause { line, text, .. } =
        interp.activity.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(line, 4);
    assert_eq!(text, b"say 1/0".to_vec());
}

/// A raise inside a matched `WHEN`'s **body** (not its condition) has to
/// go through `run_bounded`'s own `Op::Clause`'s region calls with
/// `source` actually threaded through, which is a different path from
/// every other test in this section: those all check a condition/value
/// expression `Select`'s own arm evaluates directly, and
/// `a_raise_inside_an_otherwise_branch_is_attributed_to_its_own_clause`
/// runs through the *outer* loop's `Op::Clause`'s region, never through a
/// `run_bounded` nested inside `If`/`Select`'s own arm. That last
/// distinction matters and an earlier wording of it was wrong: in the
/// test harness `run_source` routes everything through its own top-level
/// `run_bounded` with `source` supplied directly, so "never through
/// `run_bounded` at all" is true of the production outer loop only.
#[test]
fn a_raise_inside_a_matched_whens_body_is_attributed_to_its_own_clause() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"select\nwhen 1 = 1 then\n  say 1/0\nend").unwrap_err();
    let FailureSite::Clause { line, text, .. } =
        interp.activity.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(
        line, 3,
        "the WHEN's own body clause, not the SELECT's (line 1)"
    );
    assert_eq!(text, b"say 1/0".to_vec());
}

/// The `IF` analogue of the `WHEN`-body test above: a raise inside the
/// matched `THEN` branch's own body, which likewise only ever reaches
/// `Op::Clause`'s region through `run_bounded`. Confirmed by the same
/// mutation (`None` for `source` at `If`'s own `run_bounded` call site
/// made this fail too, alongside the `WHEN`-body test, both restored
/// after).
#[test]
fn a_raise_inside_an_ifs_then_body_is_attributed_to_its_own_clause() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"if 1 = 1 then\n  say 1/0").unwrap_err();
    let FailureSite::Clause { line, text, .. } =
        interp.activity.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(
        line, 2,
        "the THEN branch's own body clause, not the IF's (line 1)"
    );
    assert_eq!(text, b"say 1/0".to_vec());
}

// ---- SELECT CASE / WhenCase ----

/// **The central `WhenCase` rule.** `select case 2` / `when 1, 2 then`
/// matches (an OR of `==`), while a plain `select` / `when 1, 2` on the
/// same non-logical value is 34.6 (an AND, each element checked for
/// `0`/`1`) -- the two commas parse into the same-looking node and mean
/// opposites (`ast.rs:801-815`, and the brief's own framing).
#[test]
fn whencase_comma_is_an_or_of_equals_the_opposite_of_a_plain_whens_and() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"select case 2\n  when 1, 2 then say 'hit'\n  otherwise say 'miss'\nend"
        ),
        b"hit\n".to_vec(),
        "SELECT CASE: an OR of == comparisons"
    );

    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"select\n  when 1, 2 then nop\nend").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!(
        (raised.number, raised.sub),
        (34, 6),
        "plain SELECT: 2 is not a logical value, an AND-with-a-check, not an OR-of-=="
    );
    assert_eq!(raised.additional, vec![b"2".to_vec()]);
}

/// `==` is strict: byte-for-byte, no padding, no numeric awareness.
/// Measured: `'007'` does not
/// match `when 7`, because the two are not byte-identical.
#[test]
fn select_case_compares_with_strict_equality_not_numeric_equality() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"select case '007'\n  when 7 then say 'hit'\n  otherwise say 'miss'\nend"
        ),
        b"miss\n".to_vec()
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"select case 7\n  when 7 then say 'hit'\n  otherwise say 'miss'\nend"
        ),
        b"hit\n".to_vec()
    );
}

#[test]
fn select_case_evaluates_its_own_expression_and_runs_exactly_the_matching_when() {
    let source = |v: &str| -> Vec<u8> {
        format!("select case {v}\n  when 1 then say 'one'\n  when 2 then say 'two'\n  otherwise say 'other'\nend")
            .into_bytes()
    };
    for (v, expected) in [("1", "one\n"), ("2", "two\n"), ("3", "other\n")] {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, &source(v)),
            expected.as_bytes().to_vec(),
            "v = {v}"
        );
    }
}

// ---- SELECT with a LABEL, and a nested SELECT/IF ----

#[test]
fn a_labelled_select_with_otherwise_runs_normally() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"select label s\n  when 1 = 0 then nop\n  otherwise say 'lo'\nend"
        ),
        b"lo\n".to_vec()
    );
}

#[test]
fn a_select_nested_inside_an_ifs_then_branch_is_fully_resolved_before_resuming() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"a = 'a'\nif 1 = 1 then select\n  when 1 = 1 then a = a || 'b'\nend\na = a || 'c'\nsay a"
        ),
        b"abc\n".to_vec()
    );
}

/// A `WHEN` whose consequence is an `IF` with no `ELSE` and a false
/// condition: the `IF`'s false path lands where the branch runs out, which
/// has to close the branch and leave the `SELECT` rather than run the
/// `OTHERWISE` or the next `WHEN`. Measured against the oracle: `after`
/// alone, for each of the three things that can follow the branch.
#[test]
fn a_false_if_ending_a_whens_branch_leaves_the_select() {
    for (follows, expected) in [
        ("otherwise say 'other'", "after\n"),
        ("when n = 1 then say 'second'", "after\n"),
        ("", "after\n"),
    ] {
        let mut interp = Interp::new();
        let source = format!(
            "n = 1\nselect\n  when n = 0 then say 'zero'\n  when n = 1 then if n = 2 then say 'inner'\n  {follows}\nend\nsay 'after'"
        );
        assert_eq!(
            say_output(&mut interp, source.as_bytes()),
            expected.as_bytes().to_vec(),
            "followed by `{follows}`"
        );
    }
}
