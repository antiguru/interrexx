use super::*;

#[test]
fn a_field_round_trips_every_byte() {
    let all: Vec<u8> = (0..=255).collect();
    assert_eq!(unescape(&escape(&all)).unwrap(), all);
}

#[test]
fn inserts_round_trip_with_the_end_mark_and_backslashes_inside_values() {
    let items = vec![
        (1, b"a\\|b".to_vec()),
        (2, Vec::new()),
        (4, b"tail\\".to_vec()),
        (5, b"\t\x80".to_vec()),
    ];
    let field = encode_inserts(&items);
    assert_eq!(decode_inserts(&field).unwrap(), items);
}

#[test]
fn no_shown_insert_is_a_dash() {
    assert_eq!(encode_inserts(&[]), "-");
    assert_eq!(decode_inserts("-").unwrap(), Vec::new());
}

#[test]
fn an_item_without_an_end_mark_is_refused() {
    assert!(decode_inserts("1=a").is_err());
    assert!(decode_inserts("1=a\\").is_err());
}

#[test]
fn the_values_a_line_shows_are_recorded_by_position() {
    // Invented template: insert 2 is shown first, insert 1 not at all.
    let (got, ambiguous) = record_inserts("beta &2 gamma", &[2], b"beta [x y] gamma").unwrap();
    assert_eq!(got, vec![(2, b"[x y]".to_vec())]);
    assert!(!ambiguous);
}

#[test]
fn a_line_that_does_not_match_is_refused() {
    assert!(record_inserts("alpha &1 beta", &[1], b"alpha only").is_err());
}

#[test]
fn an_ambiguous_split_is_reported_and_takes_the_shortest_first_value() {
    // The first value could end at either " and ".
    let (got, ambiguous) = record_inserts("&1 and &2", &[1, 2], b"a and b and c").unwrap();
    assert!(ambiguous);
    assert_eq!(got, vec![(1, b"a".to_vec()), (2, b"b and c".to_vec())]);
}

#[test]
fn a_position_outside_the_rendered_set_is_not_recorded() {
    let (got, ambiguous) = record_inserts("&1 then &2", &[1], b"a then b").unwrap();
    assert_eq!(got, vec![(1, b"a".to_vec())]);
    assert!(!ambiguous);
}
