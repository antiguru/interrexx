use super::*;

fn message(major: u16, sub: u16, text: &str) -> Message {
    Message {
        major,
        sub,
        number: 1,
        symbol: format!("Error_Invented_{major}_{sub}"),
        text: text.to_string(),
    }
}

/// Invented catalogue texts: a long literal, a text of inserts and short
/// literals, a text with neither, a single short literal.
fn patterns() -> Vec<Pattern> {
    [
        message(91, 1, "The sprocket &1 refuses to rotate sideways"),
        message(91, 2, "&1 clashes &2 near &3"),
        message(91, 3, "&1"),
        message(91, 4, "Odd"),
        message(91, 5, "Frobnication failed &1"),
    ]
    .iter()
    .map(Pattern::new)
    .collect()
}

fn found(content: &str, comments_only: bool) -> Vec<(u16, u16, usize, Kind)> {
    scan(
        &patterns(),
        [("f.rs".to_string(), content.to_string())],
        comments_only,
    )
    .into_iter()
    .map(|f| (f.major, f.sub, f.line, f.kind))
    .collect()
}

#[test]
fn a_long_literal_is_found_wherever_it_sits_and_however_it_is_wrapped() {
    let hits = found(
        "fn f() {\n    // The SPROCKET\n    // x refuses to\n    // rotate   sideways\n}\n",
        false,
    );
    assert_eq!(hits, vec![(91, 1, 3, Kind::Piece)]);
    assert_eq!(
        found(
            "assert_eq!(t, \"the sprocket x refuses to rotate sideways\");",
            false
        )
        .len(),
        1
    );
}

#[test]
fn a_template_of_short_literals_is_found_by_its_whole_shape() {
    let hits = found("let t = \"abc clashes de near f\";\n", false);
    assert_eq!(hits, vec![(91, 2, 1, Kind::Template)]);
    // The literals are there, but too far apart for one text.
    let far = format!("a clashes {} near b", "x".repeat(200));
    assert!(found(&far, false).is_empty());
    // And in the wrong order.
    assert!(found("near a clashes b", false).is_empty());
}

#[test]
fn a_text_with_no_findable_literal_is_listed_and_not_found() {
    // A two-word stock phrase with one insert is no evidence of a copy
    // either, and neither is a single short literal.
    assert_eq!(unfindable(&patterns()), vec![(91, 3), (91, 4), (91, 5)]);
    assert!(found("Frobnication failed 7", false).is_empty());
    assert!(found("&1 Odd", false).is_empty());
}

#[test]
fn comments_only_ignores_code_and_non_rust_files() {
    let source = "let t = \"the sprocket x refuses to rotate sideways\";\nlet u = 1; // clean\n";
    assert!(found(source, true).is_empty());
    assert_eq!(
        found(
            "let u = 1; // the sprocket x refuses to rotate sideways\n",
            true
        )
        .len(),
        1
    );
    let md = [(
        "a.md".to_string(),
        "// the sprocket x refuses to rotate sideways".to_string(),
    )];
    assert!(scan(&patterns(), md, true).is_empty());
}

#[test]
fn a_planted_text_is_found_in_a_temp_repository() {
    let dir = std::env::temp_dir().join(format!("interrexx-scan-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("sub")).expect("creates the repository directory");
    std::fs::write(
        dir.join("sub/planted.txt"),
        "ok\nThe sprocket 7 refuses to rotate sideways\n",
    )
    .expect("writes the planted file");
    std::fs::write(dir.join("clean.txt"), "nothing here\n").expect("writes the clean file");
    std::fs::write(dir.join("binary.bin"), [0xff, 0xfe, 0x00]).expect("writes the binary file");
    let git = |args: &[&str]| {
        let status = Command::new("git")
            .arg("-C")
            .arg(&dir)
            .args(args)
            .status()
            .expect("git runs");
        assert!(status.success(), "git {args:?}");
    };
    git(&["init", "-q"]);
    git(&["add", "sub/planted.txt", "clean.txt", "binary.bin"]);
    let (files, lossy) = tracked_files(&dir, ".").expect("lists the files");
    assert_eq!(lossy, 1);
    let hits = scan(&patterns(), files, false);
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert_eq!(
        (hits[0].path.as_str(), hits[0].line),
        ("sub/planted.txt", 2)
    );
    let _ = std::fs::remove_dir_all(&dir);
}
