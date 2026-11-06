//! `corpus/introspection-arity.tsv`: what each documented
//! introspection method does when it is sent an argument list it could
//! accept.

mod support;

use support::arity;

const HEADER: &str = "\
# What each documented introspection method does when sent an argument list it
# could accept. Derived by
# crates/interrexx/tests/introspection_arity.rs; edit that, never this.
#
# class<TAB>method<TAB>arm<TAB>verdict<TAB>evidence
#
# THIS TABLE REPLACES corpus/method-bodies.txt's verdict column for these
# classes. That column sends no arguments, so for a method that needs them it
# records agreement about an arity error.
#
#   agree          the oracle and the crate give identical descriptors,
#                  the answered value among them
#   send-differs   the receiver was built on both sides and the send differs
#   setup-differs  one side could not build the receiver, so the row says
#                  nothing about its own method
#   exempt         a committed reason instead of an argument list
#   no-value       the send completed the same way on both sides and returned
#                  no result, so nothing beyond the three descriptors was
#                  compared. It is deliberately not `agree`: a reader sizing
#                  work from it learns that the send is reachable and nothing
#                  about what the method answers.
#   unstable       classified identity-derived value. Its documented
#                  stability and uniqueness properties are checked within
#                  one process without printing or comparing raw identities.
#
# THE ANSWERED VALUE IS COMPARED HERE, unlike corpus/collection-arity.tsv:
# the probe assigns the send's result and prints `vv~string`, so a row both
# sides complete with different answers reads send-differs rather than agree.
# `Class~enhanced` is the row it exists for: the oracle answers `enhanced K`
# at rc 0, so a crate that completes the send and answers something else was
# invisible to an instrument reading only the exit status and the streams.
# `~string` does not read the answer's own contents.
#
# The argument lists are corpus/introspection-arguments.tsv and the receivers
# are corpus/introspection-receivers.tsv. A list is only real if the ORACLE
# completes the send; the test makes an oracle run that does not a failure for
# that row.
#
# Pointer's and Buffer's INSTANCE arm is deliberately absent: class-set.txt
# gives them no construction expression, because the reference says instances
# come only from native code (utilityclasses.xml:429, :6910), so every
# instance row of theirs would be setup-differs on the oracle's own side.
# Their class arm is here, marked REFUSED: in the argument file -- the oracle
# answers 93.967 and that refusal is what the row measures.
";

fn layout() -> arity::Layout {
    arity::Layout {
        receivers: "introspection-receivers.tsv",
        arguments: "introspection-arguments.tsv",
        scopes: "introspection-scopes.tsv",
        table: "introspection-arity.tsv",
        refresh_env: "REXX_INTROSPECTION_ARITY_REFRESH",
        header: HEADER,
        arm_column: true,
        compare_values: true,
        fixture: true,
        probe_prefix: "rexx-introspection-arity",
    }
}

/// How many tests each of the passes over the rows is spread over: the unit nextest runs in
/// parallel is a test function. A row belongs to the shard its index in the argument list,
/// modulo this count, names.
const SHARDS: usize = 8;

/// Every shard index below `SHARDS` has a test running it, for every pass.
#[test]
fn every_row_shard_has_a_test() {
    let source = include_str!("introspection_arity.rs");
    for shard in 0..SHARDS {
        for call in ["table_shard", "sent_shard", "refused_shard"] {
            let call = format!("\n    {call}({shard});\n");
            assert_eq!(source.matches(&call).count(), 1, "{call:?}");
        }
    }
}

/// What no one shard can check: the committed table names each row of the argument list once
/// and no other, so the shards, each finding its own rows as committed, have found the table.
#[test]
fn the_table_names_exactly_the_argument_rows() {
    let problems = arity::table_rows_are_the_argument_rows(&layout());
    assert!(problems.is_empty(), "{problems:#?}");
}

/// Under the refresh variable, the table written from every row; otherwise nothing.
#[test]
fn refresh_the_table_when_asked() {
    if layout().refreshing() {
        arity::table_disagreements(&layout());
    }
}

/// The committed table is what the three sides say today, for one shard of its rows.
fn table_shard(shard: usize) {
    let moved = arity::table_disagreements_in(&layout(), shard, SHARDS);
    assert!(
        moved.is_empty(),
        "corpus/introspection-arity.tsv disagrees with the interpreters. A row that moved \
         because a task implemented something is a refresh; a row that moved otherwise is the \
         finding.\n{moved:#?}"
    );
}

/// **The harness rule.** A row whose send the oracle does not complete is a
/// failure of this instrument for that row, never a data point.
fn sent_shard(shard: usize) {
    let unsent = arity::unsent_rows_in(&layout(), shard, SHARDS);
    assert!(
        unsent.is_empty(),
        "these argument lists are not real -- the oracle does not complete the send, so the \
         row's verdict is about the list and not about the method. Fix the list in \
         corpus/introspection-arguments.tsv, or exempt it with a reason.\n{unsent:#?}"
    );
}

/// A row marked `REFUSED:` is one the oracle really does refuse.
fn refused_shard(shard: usize) {
    let completed = arity::refusals_the_oracle_completes_in(&layout(), shard, SHARDS);
    assert!(completed.is_empty(), "{completed:#?}");
}

#[test]
fn the_committed_table_matches_the_three_sides_0() {
    table_shard(0);
}

#[test]
fn the_committed_table_matches_the_three_sides_1() {
    table_shard(1);
}

#[test]
fn the_committed_table_matches_the_three_sides_2() {
    table_shard(2);
}

#[test]
fn the_committed_table_matches_the_three_sides_3() {
    table_shard(3);
}

#[test]
fn the_committed_table_matches_the_three_sides_4() {
    table_shard(4);
}

#[test]
fn the_committed_table_matches_the_three_sides_5() {
    table_shard(5);
}

#[test]
fn the_committed_table_matches_the_three_sides_6() {
    table_shard(6);
}

#[test]
fn the_committed_table_matches_the_three_sides_7() {
    table_shard(7);
}

#[test]
fn the_oracle_completes_every_send_0() {
    sent_shard(0);
}

#[test]
fn the_oracle_completes_every_send_1() {
    sent_shard(1);
}

#[test]
fn the_oracle_completes_every_send_2() {
    sent_shard(2);
}

#[test]
fn the_oracle_completes_every_send_3() {
    sent_shard(3);
}

#[test]
fn the_oracle_completes_every_send_4() {
    sent_shard(4);
}

#[test]
fn the_oracle_completes_every_send_5() {
    sent_shard(5);
}

#[test]
fn the_oracle_completes_every_send_6() {
    sent_shard(6);
}

#[test]
fn the_oracle_completes_every_send_7() {
    sent_shard(7);
}

#[test]
fn every_refused_row_is_really_refused_0() {
    refused_shard(0);
}

#[test]
fn every_refused_row_is_really_refused_1() {
    refused_shard(1);
}

#[test]
fn every_refused_row_is_really_refused_2() {
    refused_shard(2);
}

#[test]
fn every_refused_row_is_really_refused_3() {
    refused_shard(3);
}

#[test]
fn every_refused_row_is_really_refused_4() {
    refused_shard(4);
}

#[test]
fn every_refused_row_is_really_refused_5() {
    refused_shard(5);
}

#[test]
fn every_refused_row_is_really_refused_6() {
    refused_shard(6);
}

#[test]
fn every_refused_row_is_really_refused_7() {
    refused_shard(7);
}

/// An identity marker has an explicit classification and satisfies its contract.
#[test]
fn every_identity_marker_satisfies_its_documented_property() {
    let invalid = arity::invalid_identity_markers(&layout());
    assert!(invalid.is_empty(), "{invalid:#?}");
}

/// Every documented row has a list, and a native row whose upstream arity is
/// not zero is sent something.
#[test]
fn every_row_is_sent_something_its_arity_needs() {
    let wrong = arity::rows_missing_arguments(&layout());
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// An exemption carries its reason, so declining a row is a sentence and not
/// a blank.
#[test]
fn every_exemption_says_why() {
    let blank = arity::exemptions_without_a_reason(&layout());
    assert!(blank.is_empty(), "{blank:#?}");
}
