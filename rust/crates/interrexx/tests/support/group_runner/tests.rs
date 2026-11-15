use super::*;

/// The two sides' version lines, as each prints them.
const ORACLE: &str = "Interpreter:        REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026";
const OURS: &str = "Interpreter:        REXX-ooRexx_5.3.0(MT)_64-bit 6.06 14 Nov 2026";

#[test]
fn the_two_sides_version_lines_mask_alike() {
    let masked_text = |line: &str| String::from_utf8(masked(line.as_bytes())).unwrap();
    assert_eq!(masked_text(ORACLE), masked_text(OURS));
    assert_eq!(
        masked_text(OURS),
        "Interpreter:        REXX-ooRexx_5.3.0(MT)_64-bit 6.06"
    );
    assert_ne!(
        masked_text(OURS),
        masked_text(&OURS.replace("6.06", "6.07")),
        "the version itself still counts"
    );
}

#[test]
fn a_version_line_without_that_date_shape_is_kept_whole() {
    for line in [
        "Interpreter:        REXX-ooRexx_5.3.0(MT)_64-bit 6.06 2026-11-15",
        "Interpreter:        REXX-ooRexx_5.3.0(MT)_64-bit 6.06 14 Nov 2026 extra",
        "Interpreter:        REXX-ooRexx_5.3.0(MT)_64-bit 6.06 14 November 2026",
        "Interpreter: 2026",
    ] {
        assert_eq!(without_build_date(line), line);
    }
}
