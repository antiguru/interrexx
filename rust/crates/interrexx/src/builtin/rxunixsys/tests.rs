use super::{permission_text, permissions};

/// Each position takes only its own set; `s`/`S` and `t`/`T` carry the
/// special bits with and without execute. Measured, oracle: `rwsr-S--t`
/// reads back from `SysStat` as written, and `RWXrwxrwx`, `rwxrwxrws` and
/// `rwx` are routine failures.
#[test]
fn chmod_text_round_trips_through_stat_text() {
    for text in [
        "rwsr-S--t",
        "rw-r-----",
        "rwxr-x--T",
        "---------",
        "rwSrwsrwx",
    ] {
        let mode = permissions(text.as_bytes()).expect(text);
        assert_eq!(
            &permission_text(0o100_000 | mode)[1..],
            text.as_bytes(),
            "{text}"
        );
    }
    for text in ["RWXrwxrwx", "rwxrwxrws", "rwx", "rwxrwxrwxx", "rwtrwxrwx"] {
        assert_eq!(permissions(text.as_bytes()), None, "{text}");
    }
    assert_eq!(permission_text(0o020_666), b"crw-rw-rw-");
    assert_eq!(permission_text(0o047_741), b"drwsr-S--t");
}
