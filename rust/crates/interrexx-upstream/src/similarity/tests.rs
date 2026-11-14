use super::*;

fn catalogue() -> Vec<Message> {
    vec![
        Message {
            major: 91,
            sub: 1,
            number: 100,
            symbol: "Error_Invented_one".into(),
            text: "The widget &1 cannot be polished before the gadget is assembled".into(),
        },
        Message {
            major: 91,
            sub: 2,
            number: 101,
            symbol: "Error_Invented_two".into(),
            text: "Frobnication failed at &1".into(),
        },
    ]
}

fn text(major: u16, sub: u16, text: &str) -> Text {
    Text {
        major,
        sub,
        text: text.to_string(),
    }
}

#[test]
fn identical_words_score_one_and_disjoint_words_score_zero() {
    let same = compare("Frobnication failed at &1", "Frobnication failed at &1");
    assert!((same.dice - 1.0).abs() < 1e-9);
    assert_eq!(same.run, 3);
    let apart = compare("Nothing shared", "Frobnication failed");
    assert_eq!(apart, Score { dice: 0.0, run: 0 });
}

#[test]
fn an_insert_is_not_a_shared_word_and_ends_a_run() {
    let score = compare("one &1 two", "one &1 two");
    assert_eq!(score.run, 1);
    assert!((score.dice - 1.0).abs() < 1e-9);
    assert_eq!(compare("&1", "&1"), Score { dice: 0.0, run: 0 });
}

#[test]
fn case_and_punctuation_do_not_hide_a_copy() {
    let score = compare(
        "the WIDGET, &1, cannot be polished before the gadget is assembled!",
        "The widget &1 cannot be polished before the gadget is assembled",
    );
    assert!(score.run >= 6, "{score:?}");
}

#[test]
fn a_reworded_text_stays_below_the_threshold_and_a_copy_does_not() {
    let ours = [
        text(91, 1, "Polishing &1 needs an assembled gadget"),
        text(91, 2, "Frobnication failed at &1"),
    ];
    let found = hits(&ours, &catalogue(), Threshold::default());
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!((found[0].major, found[0].sub), (91, 2));
    assert_eq!(found[0].symbol, "Error_Invented_two");
}

#[test]
fn a_code_the_catalogue_lacks_is_skipped() {
    assert!(
        hits(
            &[text(91, 9, "Frobnication failed")],
            &catalogue(),
            Threshold::default()
        )
        .is_empty()
    );
}

#[test]
fn the_allow_list_parses_codes_and_reasons() {
    let allow = parse_allow("# note\n91.2\tno other wording\n");
    assert_eq!(allow, vec![((91, 2), "no other wording".to_string())]);
}
