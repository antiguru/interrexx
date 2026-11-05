use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use rexx_spec::render;

const SUBSTR: &str = "---\nelement: SUBSTR\nkind: builtin\nsyntax: \"SUBSTR(string, n)\"\nansi: \"9.4.1\"\nrexxref: \"SUBSTR\"\ndeviations: [90]\n---\n# SUBSTR\n\nr[bif.substr.result]\nReturns the part.\n\n```rexx\nr[not.a.rule]\nsee [x](r:conv.x)\n```\n```output\nbc\n```\n\nr[bif.substr.pad]\nSee [the result](r:bif.substr.result) and [conv](r:conv.x) and [gone](r:bif.nope.x).\n";

const CONV: &str = "---\nelement: Conventions\nkind: concept\n---\n# Conventions\n\nr[conv.x]\nA rule; see [pad](r:bif.substr.pad).\n";

fn pages() -> BTreeMap<String, PathBuf> {
    let mut m = BTreeMap::new();
    m.insert(
        "bif.substr.result".into(),
        PathBuf::from("builtins/substr.md"),
    );
    m.insert("bif.substr.pad".into(), PathBuf::from("builtins/substr.md"));
    m.insert("conv.x".into(), PathBuf::from("conventions.md"));
    m
}

#[test]
fn renders_anchor_links_and_drops_front_matter() {
    let out = render(SUBSTR, &pages(), Path::new("builtins/substr.md"));
    assert!(out.contains(
        "<div class=\"rule\" id=\"r-bif.substr.result\"><a href=\"#r-bif.substr.result\">[bif.substr.result]</a></div>"
    ));
    assert!(out.contains("(#r-bif.substr.result)"), "{out}");
    assert!(out.contains("(../conventions.md#r-conv.x)"), "{out}");
    assert!(out.contains("(r:bif.nope.x)"), "unknown ids stay: {out}");
    assert!(!out.contains("\n---\n") && !out.starts_with("---"), "{out}");
    assert!(
        out.contains("r[not.a.rule]\nsee [x](r:conv.x)\n"),
        "fenced text stays: {out}"
    );
    assert!(
        out.contains("SUBSTR(string, n)") && out.contains("D90"),
        "{out}"
    );
    let title = out.find("# SUBSTR").unwrap();
    assert!(out.find("D90").unwrap() > title, "header under the title");
}

#[test]
fn link_from_the_root_into_a_chapter_directory() {
    let out = render(CONV, &pages(), Path::new("conventions.md"));
    assert!(
        out.contains("(builtins/substr.md#r-bif.substr.pad)"),
        "{out}"
    );
}

#[test]
fn a_page_that_does_not_parse_is_unchanged() {
    let bad = "no front matter\nr[bif.x.y]\n";
    assert_eq!(render(bad, &pages(), Path::new("a.md")), bad);
}

fn chapter(
    name: &str,
    path: Option<&str>,
    content: &str,
    subs: Vec<serde_json::Value>,
) -> serde_json::Value {
    serde_json::json!({"Chapter": {"name": name, "content": content, "number": null,
        "sub_items": subs, "path": path, "source_path": path, "parent_names": []}})
}

fn run_bin(book_key: &str) -> serde_json::Value {
    let book = serde_json::json!({ book_key: [
        chapter("Conventions", Some("conventions.md"), CONV,
            vec![chapter("SUBSTR", Some("builtins/substr.md"), SUBSTR, vec![])]),
        "Separator",
        chapter("Draft", None, "# draft\nr[bif.q.r]\n", vec![]),
    ]});
    let input = serde_json::json!([{"root": "."}, book]).to_string();
    let mut child = Command::new(env!("CARGO_BIN_EXE_mdbook-rexx-spec"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    serde_json::from_slice(&out.stdout).unwrap()
}

#[test]
fn binary_rewrites_every_chapter_including_sub_items() {
    for key in ["sections", "items"] {
        let book = run_bin(key);
        let top = &book[key][0]["Chapter"];
        let conv = top["content"].as_str().unwrap();
        assert!(
            conv.contains("(builtins/substr.md#r-bif.substr.pad)"),
            "{conv}"
        );
        let sub = book[key][0]["Chapter"]["sub_items"][0]["Chapter"]["content"]
            .as_str()
            .unwrap();
        assert!(sub.contains("(../conventions.md#r-conv.x)"), "{sub}");
        assert_eq!(book[key][1], "Separator");
        assert_eq!(book[key][2]["Chapter"]["content"], "# draft\nr[bif.q.r]\n");
    }
}

#[test]
fn supports_exits_zero() {
    let st = Command::new(env!("CARGO_BIN_EXE_mdbook-rexx-spec"))
        .args(["supports", "html"])
        .status()
        .unwrap();
    assert!(st.success());
}

#[test]
fn titleless_page_with_a_link_on_its_first_line_gets_an_intact_header() {
    let page = "---\nelement: X\nkind: concept\nsyntax: \"X()\"\n---\nSee [c](r:conv.x) first.\n\nr[conv.y]\nBody.\n";
    let out = render(page, &pages(), Path::new("builtins/x.md"));
    assert!(out.contains("X()"), "{out}");
    assert!(
        out.contains("\nSee [c](../conventions.md#r-conv.x) first.\n"),
        "{out}"
    );
    assert!(
        out.find("X()").unwrap() < out.find("See [c]").unwrap(),
        "{out}"
    );
}

// A `<div>` opens an HTML block that only a blank line ends; without one the
// rule's paragraph would be shown as raw text.
#[test]
fn a_rule_paragraph_is_separated_from_its_anchor_by_a_blank_line() {
    let page = "---\nelement: X\nkind: concept\n---\n# X\n\nr[conv.y]\nTake `x`; see [pad](r:bif.substr.pad).\n";
    let out = render(page, &pages(), Path::new("conventions.md"));
    assert!(
        out.contains("</a></div>\n\nTake `x`; see [pad](builtins/substr.md#r-bif.substr.pad).\n"),
        "{out}"
    );
}

#[test]
fn an_html_header_is_separated_from_what_follows_by_a_blank_line() {
    // Header under a title.
    let out = render(SUBSTR, &pages(), Path::new("builtins/substr.md"));
    assert!(out.contains("D90</p>\n\n"), "{out}");
    // Header before a page with no title.
    let page = "---\nelement: X\nkind: concept\nansi: \"1.2\"\n---\nr[conv.y]\nBody.\n";
    let out = render(page, &pages(), Path::new("conventions.md"));
    assert!(
        out.contains("ANSI: 1.2</p>\n\n<div class=\"rule\""),
        "{out}"
    );
}

#[test]
fn references_are_html_escaped() {
    let page = "---\nelement: X\nkind: concept\nansi: \"a & b\"\nrexxref: \"A > B < \\\"C\\\"\"\n---\n# X\n\nr[conv.y]\nBody.\n";
    let out = render(page, &pages(), Path::new("conventions.md"));
    assert!(
        out.contains("ANSI: a &amp; b; rexxref: A &gt; B &lt; &quot;C&quot;</p>"),
        "{out}"
    );
}

#[test]
fn non_id_brackets_are_left_alone() {
    // The parser reports such a line, so the page is returned as written.
    let page = "---\nelement: X\nkind: concept\n---\n# X\n\nr[1] + r[2]\n\nr[conv.y]\nBody.\n";
    assert_eq!(render(page, &pages(), Path::new("builtins/x.md")), page);
}
