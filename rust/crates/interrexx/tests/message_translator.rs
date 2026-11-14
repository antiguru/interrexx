//! The translator that maps the oracle's message lines to this crate's
//! texts (`support/messages.rs`), on tables of invented texts. The tables
//! share no wording with any catalogue; `message_translator_upstream.rs` runs
//! the translator on the real one.

#[path = "support/messages.rs"]
mod messages;

use messages::{Entry, Translator};

/// The oracle's side: invented texts, one per case the translator handles.
const THEIRS: &[Entry<'static>] = &[
    (5, 0, "Flurble gone wrong."),
    (5, 3, "The &1 widget is &2 than &3."),
    // The text shows insert 2 only.
    (5, 4, "Gizmo &2 only."),
    // One insert shown twice.
    (5, 5, "Quux &1 and again &1."),
    // An insert may hold the template's own punctuation.
    (5, 6, "Wug &1: &2."),
    (101, 4, "+++ Zorp trace.  Mishap"),
    (101, 5, "+++ Zorp ready +++"),
    (101, 21, "     *-* Zorp &1 routine"),
    (101, 25, "     *-* Zorp method &1 in &2"),
    (999, 14, "Syntax: zorp file"),
    // A short name is never mapped.
    (101, 6, "Janx"),
];

/// This crate's side: other wording, inserts in another order.
const OURS: &[Entry<'static>] = &[
    (5, 0, "Wibble failed."),
    (5, 3, "&3 is less than &1 and &2."),
    (5, 4, "Only &2."),
    (5, 5, "&1 twice."),
    (5, 6, "&2 after &1."),
    (101, 4, "+++ Blip debugging.  Oops"),
    (101, 5, "+++ Blip waits +++"),
    (101, 21, "     *-* Blip &1 routine"),
    (101, 25, "     *-* Blip in &2 method &1"),
    (999, 14, "Usage: blip file"),
    (101, 6, "Jonx"),
];

fn translator() -> Translator {
    Translator::new(THEIRS, OURS)
}

fn map(text: &str) -> String {
    let translated = translator().translate(text.as_bytes());
    assert!(
        translated.ambiguous.is_empty(),
        "{:?}",
        translated.ambiguous
    );
    String::from_utf8(translated.bytes).expect("text")
}

#[test]
fn an_error_line_keeps_its_frame_and_takes_our_text() {
    assert_eq!(
        map("Error 5 running /w/p.rex line 3:  Flurble gone wrong.\n"),
        "Error 5 running /w/p.rex line 3:  Wibble failed.\n"
    );
    // A report without a position.
    assert_eq!(
        map("Error 5:  Flurble gone wrong.\n"),
        "Error 5:  Wibble failed.\n"
    );
}

#[test]
fn a_sub_line_maps_its_inserts_into_our_order() {
    assert_eq!(
        map("Error 5.3:  The red widget is slower than blue.\n"),
        "Error 5.3:  blue is less than red and slower.\n"
    );
}

#[test]
fn a_text_that_skips_a_position_maps_the_positions_it_shows() {
    assert_eq!(
        map("Error 5.4:  Gizmo two only.\n"),
        "Error 5.4:  Only two.\n"
    );
}

#[test]
fn an_insert_shown_twice_must_hold_one_value() {
    assert_eq!(
        map("Error 5.5:  Quux a and again a.\n"),
        "Error 5.5:  a twice.\n"
    );
    assert_eq!(
        map("Error 5.5:  Quux a and again b.\n"),
        "Error 5.5:  Quux a and again b.\n"
    );
}

#[test]
fn a_path_holding_the_frames_separator_is_not_cut_there() {
    assert_eq!(
        map("Error 5 running /a:  b/p.rex line 1:  Flurble gone wrong.\n"),
        "Error 5 running /a:  b/p.rex line 1:  Wibble failed.\n"
    );
}

#[test]
fn an_insert_holding_the_templates_punctuation_is_reported_as_ambiguous() {
    // The split at the first colon is taken, and the line is named because
    // the split at the second would render differently.
    let translated = translator().translate(b"Error 5.6:  Wug a: b: c.\n");
    assert_eq!(translated.bytes, b"Error 5.6:  b: c after a.\n");
    assert_eq!(translated.ambiguous, ["5.6: Wug a: b: c."]);
    // An unambiguous line with a colon in an insert is not reported.
    assert_eq!(map("Error 5.6:  Wug a: b.\n"), "Error 5.6:  b after a.\n");
}

#[test]
fn a_line_that_is_no_message_is_left_alone() {
    for line in [
        "hello\n",
        "Error 5.3:  Some other wording entirely.\n",
        "Error 77.1:  Flurble gone wrong.\n",
        "Error 5 running p.rex line 3: Flurble gone wrong.\n",
        "Error five: Flurble gone wrong.\n",
        "  Error 5:  Flurble gone wrong.\n",
        "say 'Error 5:  Flurble gone wrong.'\n",
        "\n",
        "Janx\n",
        "     2 *-* say 1\n",
    ] {
        assert_eq!(map(line), line, "{line:?}");
    }
    // No line end at the end, and bytes that are not text.
    assert_eq!(
        translator()
            .translate(b"Error 5:  Flurble gone wrong.")
            .bytes,
        b"Error 5:  Wibble failed."
    );
    assert_eq!(
        map("\u{1}\u{ff}\nError 5:  Flurble gone wrong.\n"),
        "\u{1}\u{ff}\nError 5:  Wibble failed.\n"
    );
    let raw = [
        0x80u8, 0xff, b'\n', 0x00, b'E', b'r', b'r', b'o', b'r', b' ',
    ];
    assert_eq!(translator().translate(&raw).bytes, raw);
}

#[test]
fn only_message_lines_change_in_a_whole_output() {
    assert_eq!(
        map(
            "out\n     3 *-* x = 1\nError 5 running f line 3:  Flurble gone wrong.\nError 5.3:  The a widget is b than c.\nmore\n"
        ),
        "out\n     3 *-* x = 1\nError 5 running f line 3:  Wibble failed.\nError 5.3:  c is less than a and b.\nmore\n"
    );
}

#[test]
fn a_debug_pause_maps_its_prefix_notice_and_error_lines() {
    assert_eq!(map("+++ Zorp ready +++\n"), "+++ Blip waits +++\n");
    assert_eq!(
        map("+++ Zorp trace.  Mishap 5:   Flurble gone wrong.\n"),
        "+++ Blip debugging.  Oops 5:   Wibble failed.\n"
    );
    assert_eq!(
        map("+++ Zorp trace.  Mishap 5.3:  The a widget is b than c.\n"),
        "+++ Blip debugging.  Oops 5.3:  c is less than a and b.\n"
    );
    // The prefix with no code after it is not an error line.
    assert_eq!(
        map("+++ Zorp trace.  Mishap here\n"),
        "+++ Zorp trace.  Mishap here\n"
    );
}

#[test]
fn a_traceback_line_is_mapped_whole() {
    assert_eq!(
        map("     *-* Zorp method M in S\n"),
        "     *-* Blip in S method M\n"
    );
}

#[test]
fn a_line_two_whole_line_texts_explain_is_left_alone_and_reported() {
    // `101.26` matches every line `101.21` matches.
    let theirs: &[Entry<'static>] = &[
        (101, 21, "     *-* Zorp &1 routine"),
        (101, 26, "     *-* Zorp &1"),
    ];
    let ours: &[Entry<'static>] = &[
        (101, 21, "     *-* Blip &1 routine"),
        (101, 26, "     *-* Blip &1"),
    ];
    let translated = Translator::new(theirs, ours).translate(b"     *-* Zorp x routine\n");
    assert_eq!(translated.bytes, b"     *-* Zorp x routine\n");
    assert_eq!(translated.ambiguous.len(), 1);
}

#[test]
fn a_utility_message_is_mapped_whole() {
    assert_eq!(map("Syntax: zorp file\n"), "Usage: blip file\n");
    // Only the whole line is the message.
    assert_eq!(map("Syntax: zorp file x\n"), "Syntax: zorp file x\n");
    assert_eq!(map(" Syntax: zorp file\n"), " Syntax: zorp file\n");
}

#[test]
fn a_short_name_is_not_mapped() {
    assert_eq!(map("Janx\n"), "Janx\n");
}

#[test]
fn equal_tables_make_the_translator_the_identity() {
    let same = Translator::new(THEIRS, THEIRS);
    let text = "Error 5 running /w/p.rex line 3:  Flurble gone wrong.\n\
                Error 5.3:  The red widget is slower than blue.\n\
                Error 5.4:  Gizmo two only.\n\
                Error 5.5:  Quux a and again a.\n\
                Error 5.6:  Wug a: b: c.\n\
                +++ Zorp ready +++\n\
                +++ Zorp trace.  Mishap 5.3:  The a widget is b than c.\n\
                     *-* Zorp method M in S\n\
                Syntax: zorp file\n\
                plain\n";
    assert_eq!(same.translate(text.as_bytes()).bytes, text.as_bytes());
}
