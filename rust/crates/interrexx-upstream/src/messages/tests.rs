use super::*;

/// An invented table: 91.1 shows inserts 1 and 3 (and takes three), 91.2
/// shows two, 91.3 shows none but takes one.
fn table() -> Table {
    Table::from_tsv(
        "91\t1\t100\tError_Invented_one\t3\t1,3\n\
         91\t2\t101\tError_Invented_two\t2\t1,2\n\
         91\t3\t102\tError_Invented_three\t1\t-\n",
    )
}

fn text(major: u16, sub: u16, text: &str) -> Text {
    Text {
        major,
        sub,
        text: text.to_string(),
    }
}

#[test]
fn a_file_parses_with_comments_and_blank_lines() {
    let rows = parse_file("91.tsv", "# note\n\n91\t1\tItem &1 of &3\n").expect("parses");
    assert_eq!(rows, vec![text(91, 1, "Item &1 of &3")]);
}

#[test]
fn a_file_reports_every_problem() {
    let problems = parse_file("91.tsv", "91\t1\n92\t1\tother major\nx\t1\tbad\n91\t4\t \n")
        .expect_err("four bad rows");
    assert_eq!(problems.len(), 4, "{problems:?}");
    assert!(problems[0].contains("91.tsv:1"));
    assert!(problems[1].contains("does not belong"));
    assert!(parse_file("notes.txt", "").is_err());
    assert!(parse_file("91.tsv", "").is_ok());
}

#[test]
fn a_text_that_uses_a_third_insert_of_a_two_insert_code_is_rejected() {
    let problems = validate(&table(), &[text(91, 2, "Got &1, &2 and &3")]);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].contains("&3"));
}

#[test]
fn a_text_that_uses_a_position_outside_the_rendered_set_is_rejected() {
    // 91.1 takes three inserts but shows only the first and third.
    let problems = validate(&table(), &[text(91, 1, "Item &2")]);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(validate(&table(), &[text(91, 1, "Item &1 of &3")]).is_empty());
    // A message that shows no insert takes none in its text.
    assert_eq!(validate(&table(), &[text(91, 3, "Odd &1")]).len(), 1);
}

#[test]
fn an_unknown_and_a_repeated_code_are_rejected() {
    let problems = validate(
        &table(),
        &[
            text(91, 9, "No such code"),
            text(91, 2, "a"),
            text(91, 2, "b"),
        ],
    );
    assert_eq!(problems.len(), 2, "{problems:?}");
    assert!(problems[0].contains("not in errors.tsv"));
    assert!(problems[1].contains("more than one"));
}
