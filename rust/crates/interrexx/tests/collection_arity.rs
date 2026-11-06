//! `corpus/collection-arity.tsv`: what each documented
//! collection method does when it is sent an argument list it could accept.

mod support;

use support::arity;

const HEADER: &str = "\
# What each documented collection method does when sent an argument list it
# could accept. Derived by
# crates/interrexx/tests/collection_arity.rs; edit that, never this.
#
# class<TAB>method<TAB>verdict<TAB>evidence
#
# THIS TABLE REPLACES corpus/method-bodies.txt's verdict column for these
# classes. That column sends no arguments, so for a method that needs them it
# records agreement about an arity error.
#
#   agree          the oracle and the crate give identical descriptors
#   send-differs   the receiver was built on both sides and the send differs
#   setup-differs  one side could not build the receiver, so the row says
#                  nothing about its own method -- the class has no store
#   exempt         a committed reason instead of an argument list
#
# The argument lists are corpus/collection-arguments.tsv and the receivers are
# corpus/collection-receivers.tsv. A list is only real if the ORACLE completes
# the send; the test makes an oracle run that does not a failure for that row.
";

fn layout() -> arity::Layout {
    arity::Layout {
        receivers: "collection-receivers.tsv",
        arguments: "collection-arguments.tsv",
        scopes: "collection-scopes.tsv",
        table: "collection-arity.tsv",
        refresh_env: "REXX_COLLECTION_ARITY_REFRESH",
        header: HEADER,
        arm_column: false,
        compare_values: false,
        fixture: false,
        probe_prefix: "rexx-collection-arity",
    }
}

/// How many tests each of the passes over the rows is spread over: the unit nextest runs in
/// parallel is a test function. A row belongs to the shard its index in the argument list,
/// modulo this count, names.
const SHARDS: usize = 8;

/// Every shard index below `SHARDS` has a test running it, for every pass.
#[test]
fn every_row_shard_has_a_test() {
    let source = include_str!("collection_arity.rs");
    for shard in 0..SHARDS {
        for call in ["table_shard", "sent_shard"] {
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
        "corpus/collection-arity.tsv disagrees with the interpreters. A row that moved because \
         a task implemented something is a refresh; a row that moved otherwise is the finding.\n\
         {moved:#?}"
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
         corpus/collection-arguments.tsv, or exempt it with a reason.\n{unsent:#?}"
    );
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

/// Every documented instance row has a list, and a native row whose upstream
/// arity is not zero is sent something.
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
