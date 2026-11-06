use std::fs;
use std::path::{Path, PathBuf};

use interrexx_spec::{Kind, Problem, deviation_number, load, parse_page, prefix_for};

const FRONT: &str = "---\nelement: SUBSTR\nkind: builtin\nsyntax: \"SUBSTR(string, n)\"\ndeviations: [90, 91]\n---\n";

fn page_text() -> String {
    format!(
        "{FRONT}\n# SUBSTR\n\nr[bif.substr.result]\nReturns the part.\n\n```rexx\nsay substr('abc', 2)\n```\n```output\nbc\n```\n\nr[bif.substr.pad]\nSee [the result](r:bif.substr.result).\n"
    )
}

fn parse(text: &str) -> Result<interrexx_spec::Page, Vec<Problem>> {
    parse_page(Path::new("builtins/substr.md"), text)
}

fn problems(text: &str) -> Vec<(usize, String)> {
    match parse(text) {
        Ok(_) => Vec::new(),
        Err(p) => p.into_iter().map(|p| (p.line, p.message)).collect(),
    }
}

#[test]
fn a_page_yields_its_rules_examples_and_links() {
    let page = parse(&page_text()).unwrap();
    assert_eq!(page.front.element, "SUBSTR");
    assert_eq!(page.front.kind, Kind::Builtin);
    assert_eq!(page.front.syntax.as_deref(), Some("SUBSTR(string, n)"));
    assert_eq!(page.front.deviations, vec![90, 91]);
    let ids: Vec<_> = page.rules.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids, ["bif.substr.result", "bif.substr.pad"]);
    assert_eq!(page.rules[0].text, "Returns the part.");
    assert_eq!(page.examples.len(), 1);
    let e = &page.examples[0];
    assert_eq!(e.rule, "bif.substr.result");
    assert_eq!(e.program, "say substr('abc', 2)\n");
    assert_eq!(e.output, "bc\n");
    assert_eq!(e.rc, 0);
    assert_eq!(page.links.len(), 1);
    assert_eq!(page.links[0].id, "bif.substr.result");
    assert_eq!(page.links[0].line, 21);
}

#[test]
fn output_rc_is_read() {
    let text = page_text().replace("```output\n", "```output rc=1\n");
    assert_eq!(parse(&text).unwrap().examples[0].rc, 1);
}

#[test]
fn output_defect_is_read_and_combines_with_rc() {
    let text = page_text().replace("```output\n", "```output defect=#12 rc=1\n");
    let e = &parse(&text).unwrap().examples[0];
    assert_eq!((e.defect, e.rc), (Some(12), 1));
    let text = page_text().replace("```output\n", "```output rc=1 defect=#12\n");
    let e = &parse(&text).unwrap().examples[0];
    assert_eq!((e.defect, e.rc), (Some(12), 1));
    assert_eq!(parse(&page_text()).unwrap().examples[0].defect, None);
}

#[test]
fn a_malformed_defect_is_a_problem() {
    for bad in [
        "defect=12",
        "defect=#",
        "defect=#x",
        "defect=#1a",
        "defect=",
        "defect=#-1",
    ] {
        let text = page_text().replace("```output\n", &format!("```output {bad}\n"));
        assert!(parse(&text).is_err(), "{bad}");
    }
}

#[test]
fn a_duplicate_output_option_is_a_problem() {
    for dup in [
        "defect=#1 defect=#2",
        "defect=#1 defect=#1",
        "rc=1 rc=2",
        "rc=0 rc=0",
    ] {
        let text = page_text().replace("```output\n", &format!("```output {dup}\n"));
        assert!(parse(&text).is_err(), "{dup}");
    }
}

fn escaped_page(output: &str, options: &str) -> String {
    page_text().replace("```output\nbc\n", &format!("```output {options}\n{output}"))
}

#[test]
fn an_escaped_output_block_is_decoded() {
    let text = escaped_page("\\a[] 0\n\\x07\\x41\\\\a\n", "escaped");
    assert_eq!(
        parse(&text).unwrap().examples[0].output,
        "\u{7}[] 0\n\u{7}A\\a\n"
    );
    let text = escaped_page("\\a\n", "defect=#3 escaped rc=2");
    let e = &parse(&text).unwrap().examples[0];
    assert_eq!((e.output.as_str(), e.defect, e.rc), ("\u{7}\n", Some(3), 2));
}

#[test]
fn a_backslash_in_an_unmarked_output_block_is_text() {
    let text = escaped_page("\\a\\x07\n", "");
    assert_eq!(parse(&text).unwrap().examples[0].output, "\\a\\x07\n");
}

#[test]
fn a_bad_escape_is_a_problem_at_its_line() {
    for bad in ["\\n", "\\x7", "\\x80", "\\xg0", "\\x+7", "\\", "ok\\"] {
        let text = escaped_page(&format!("fine\n{bad}\n"), "escaped");
        let found = problems(&text);
        assert_eq!(found.len(), 1, "{bad}: {found:?}");
        assert_eq!(found[0].0, 18, "{bad}: {found:?}");
    }
}

#[test]
fn a_duplicate_escaped_option_is_a_problem() {
    let text = escaped_page("x\n", "escaped escaped");
    assert_eq!(problems(&text).len(), 1);
}

#[test]
fn an_empty_output_block_is_the_empty_string() {
    let text = page_text().replace("bc\n```\n", "```\n");
    assert_eq!(parse(&text).unwrap().examples[0].output, "");
}

#[test]
fn a_rexx_block_without_output_is_not_an_example() {
    let text = format!("{FRONT}\nr[bif.a.b]\nText.\n\n```rexx\nsay 1\n```\n");
    assert!(parse(&text).unwrap().examples.is_empty());
}

#[test]
fn rules_in_code_fences_are_not_rules() {
    let text = format!("{FRONT}\nr[bif.a.b]\nText.\n\n````markdown\nr[bif.c.d]\n````\n");
    assert_eq!(parse(&text).unwrap().rules.len(), 1);
}

// Review Focus 1: a clean page and a damaged one parse the same.
#[test]
fn crlf_and_trailing_space_parse_the_same() {
    let clean = parse(&page_text()).unwrap();
    let damaged = page_text()
        .replace("r[bif.substr.result]", "r[bif.substr.result] ")
        .replace('\n', "\r\n");
    let damaged = parse(&damaged).unwrap();
    assert_eq!(clean.rules, damaged.rules);
    assert_eq!(clean.examples, damaged.examples);
    assert_eq!(clean.links, damaged.links);
    assert_eq!(clean.front, damaged.front);
}

#[test]
fn bad_front_matter_is_reported() {
    let page = |fm: &str| format!("---\n{fm}---\n");
    // Missing `element`: reported on the opening fence.
    let p = problems(&page("kind: builtin\n"));
    assert_eq!(p.len(), 1, "{p:?}");
    assert_eq!(p[0].0, 1);
    assert!(p[0].1.contains("element"));
    // Unknown key.
    let p = problems(&page("element: X\nkind: builtin\nfoo: 1\n"));
    assert_eq!(p.len(), 1, "{p:?}");
    assert_eq!(p[0].0, 4);
    assert!(p[0].1.contains("foo"));
    // Unknown kind.
    let p = problems(&page("element: X\nkind: banana\n"));
    assert_eq!(p.len(), 1, "{p:?}");
    assert_eq!(p[0].0, 3);
    assert!(p[0].1.contains("banana"));
    // Nested mapping.
    let p = problems(&page("element: X\nkind: builtin\nsyntax:\n  a: b\n"));
    assert_eq!(p.len(), 1, "{p:?}");
    assert_eq!(p[0].0, 4);
}

#[test]
fn missing_or_unclosed_front_matter_is_reported() {
    assert_eq!(problems("r[bif.a.b]\nText.\n")[0].0, 1);
    assert_eq!(problems("---\nelement: X\nkind: builtin\n")[0].0, 1);
}

#[test]
fn quoted_values_comments_and_lists() {
    let text = "---\n# note\nelement: X # trailing\nkind: concept\nrexxref: \"A > B # not a comment\"\ndeviations: []\n---\n";
    let page = parse(text).unwrap();
    assert_eq!(page.front.element, "X");
    assert_eq!(page.front.rexxref.as_deref(), Some("A > B # not a comment"));
    assert!(page.front.deviations.is_empty());
    let p = problems("---\nelement: X\nkind: concept\ndeviations: [a]\n---\n");
    assert_eq!(p.len(), 1, "{p:?}");
}

#[test]
fn example_before_any_rule_is_reported() {
    let text = format!("{FRONT}\n```rexx\nsay 1\n```\n```output\n1\n```\n");
    let p = problems(&text);
    assert_eq!(p.len(), 1, "{p:?}");
    assert_eq!(p[0].0, 8);
}

#[test]
fn malformed_id_is_reported() {
    for bad in ["r[Bif.X]", "r[bif]", "r[bif..x]", "r[]"] {
        let p = problems(&format!("{FRONT}\n{bad}\nText.\n"));
        assert_eq!(p.len(), 1, "{bad}: {p:?}");
        assert_eq!(p[0].0, 8);
    }
}

#[test]
fn prefixes_name_the_page() {
    let p = |s: &str| prefix_for(Path::new(s));
    assert_eq!(p("builtins/substr.md").as_deref(), Some("bif.substr."));
    assert_eq!(p("builtins/index.md").as_deref(), Some("bif.index."));
    assert_eq!(p("instructions/do.md").as_deref(), Some("instr.do."));
    assert_eq!(p("expressions/x.md").as_deref(), Some("expr.x."));
    assert_eq!(p("parsing/x.md").as_deref(), Some("parse.x."));
    assert_eq!(p("conditions/x.md").as_deref(), Some("cond.x."));
    assert_eq!(p("classes/x.md").as_deref(), Some("class.x."));
    assert_eq!(p("lexical/x.md").as_deref(), Some("lex.x."));
    assert_eq!(p("numbers/x.md").as_deref(), Some("num.x."));
    assert_eq!(p("directives/x.md").as_deref(), Some("dir.x."));
    assert_eq!(p("rexxutil/x.md").as_deref(), Some("util.x."));
    assert_eq!(p("environment/x.md").as_deref(), Some("env.x."));
    assert_eq!(p("conventions.md").as_deref(), Some("conv."));
    assert_eq!(p("intro.md"), None);
    assert_eq!(p("other/x.md"), None);
    assert_eq!(p("builtins/deeper/x.md"), None);
}

struct Tree(PathBuf);

impl Tree {
    fn new(name: &str) -> Tree {
        let dir =
            std::env::temp_dir().join(format!("interrexx-spec-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("src")).unwrap();
        Tree(dir)
    }
    fn write(&self, rel: &str, text: &str) {
        let p = self.0.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn simple(ids: &[&str]) -> String {
    let mut s = String::from("---\nelement: X\nkind: concept\n---\n");
    for id in ids {
        s.push_str(&format!("\nr[{id}]\nText.\n"));
    }
    s
}

#[test]
fn load_accepts_a_clean_tree_without_withdrawn_list() {
    let t = Tree::new("clean");
    t.write(
        "src/SUMMARY.md",
        "# Summary\n\n- [Conv](conventions.md)\n- [A](builtins/a.md)\n",
    );
    t.write("src/conventions.md", &simple(&["conv.x.y"]));
    t.write("src/builtins/a.md", &simple(&["bif.a.b"]));
    let (pages, problems) = load(&t.0);
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(pages.len(), 2);
}

#[test]
fn load_reports_duplicates_withdrawn_prefix_and_summary() {
    let t = Tree::new("defects");
    t.write("withdrawn.txt", "# gone\n\nbif.a.old\n");
    t.write("src/SUMMARY.md", "# Summary\n\n- [Conv](conventions.md)\n- [A](builtins/a.md)\n- [B](builtins/b.md)\n- [Gone](builtins/missing.md)\n");
    t.write("src/conventions.md", &simple(&["bif.wrong.prefix"]));
    t.write("src/builtins/a.md", &simple(&["bif.a.b", "bif.a.old"]));
    t.write("src/builtins/b.md", &simple(&["bif.a.b"]));
    t.write("src/builtins/unlisted.md", &simple(&["bif.unlisted.v"]));
    let (_, problems) = load(&t.0);
    let msgs: Vec<String> = problems
        .iter()
        .map(|p| format!("{}:{}: {}", p.path.display(), p.line, p.message))
        .collect();
    // b.md's duplicate is also on the wrong page for its ID.
    assert_eq!(problems.len(), 6, "{msgs:#?}");
    let dup = msgs.iter().find(|m| m.contains("already defined")).unwrap();
    assert!(
        dup.contains("builtins/b.md") && dup.contains("builtins/a.md"),
        "{dup}"
    );
    assert!(msgs.iter().any(|m| m.contains("withdrawn")));
    assert!(
        msgs.iter()
            .any(|m| m.contains("conventions.md") && m.contains("conv."))
    );
    assert!(
        msgs.iter()
            .any(|m| m.contains("builtins/b.md") && m.contains("`bif.b.`"))
    );
    assert!(msgs.iter().any(|m| m.contains("missing.md")));
    assert!(msgs.iter().any(|m| m.contains("unlisted.md")));
}

// Review Focus 5.
#[test]
fn a_duplicate_within_one_page_is_reported() {
    let t = Tree::new("dup-same");
    t.write("src/SUMMARY.md", "- [A](builtins/a.md)\n");
    t.write("src/builtins/a.md", &simple(&["bif.a.b", "bif.a.b"]));
    let (_, problems) = load(&t.0);
    assert_eq!(problems.len(), 1, "{problems:?}");
}

#[test]
fn draft_summary_entries_are_reported() {
    let t = Tree::new("draft");
    t.write("src/SUMMARY.md", "- [Draft]()\n");
    let (_, problems) = load(&t.0);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert_eq!(problems[0].line, 1);
}

#[test]
fn trailing_spaces_in_blocks_are_kept_and_crlf_does_not_change_them() {
    let text = page_text()
        .replace("say substr('abc', 2)\n", "say 'bc  '  \n")
        .replace("\nbc\n", "\nbc  \n");
    let lf = parse(&text).unwrap();
    assert_eq!(lf.examples[0].output, "bc  \n");
    assert_eq!(lf.examples[0].program, "say 'bc  '  \n");
    let crlf = parse(&text.replace('\n', "\r\n")).unwrap();
    assert_eq!(crlf.examples, lf.examples);
}

#[test]
fn a_near_miss_rule_marker_is_reported() {
    for bad in [" r[bif.a.b]", "r[bif.a.b] trailing", "\tr[bif.a.b]"] {
        let p = problems(&format!("{FRONT}\n{bad}\nText.\n"));
        assert_eq!(p.len(), 1, "{bad:?}: {p:?}");
        assert_eq!(p[0].0, 8);
    }
    // Inside a fence it is just text.
    assert!(problems(&format!("{FRONT}\n```text\n r[bif.a.b]\n```\n")).is_empty());
}

#[test]
fn deviation_numbers_are_read_only_from_a_final_deviation_segment() {
    assert_eq!(deviation_number("bif.substr.deviation.4"), Some(4));
    assert_eq!(deviation_number("bif.count.answer.deviation.90"), Some(90));
    assert_eq!(deviation_number("bif.x.deviation.0"), Some(0));
    for none in [
        "bif.x.deviation.pending",
        "bif.x.deviation.04",
        "bif.x.deviation",
        "bif.deviation.x",
        "bif.deviation.x.deviation.4",
        "bif.x.deviations.4",
        "bif.x.4",
    ] {
        assert_eq!(deviation_number(none), None, "{none}");
    }
}

#[test]
fn a_deviation_segment_not_ending_in_a_number_is_reported() {
    for bad in [
        "r[bif.x.deviation.pending]",
        "r[bif.x.deviation.later.4]",
        "r[bif.deviation.x]",
    ] {
        let p = problems(&format!("{FRONT}\n{bad}\nText.\n"));
        assert_eq!(p.len(), 1, "{bad}: {p:?}");
        assert_eq!(p[0].0, 8);
        assert!(p[0].1.contains("deviation"), "{p:?}");
    }
    assert!(problems(&format!("{FRONT}\nr[bif.x.deviation.4]\nText.\n")).is_empty());
}

#[test]
fn a_fence_markdown_accepts_but_the_format_does_not_is_reported() {
    // Each would open a code block in Markdown that the parser does not see,
    // hiding the rule under it or an example's program.
    for bad in [" ```rexx", "   ```", "~~~", "  ~~~rexx", "```a`b"] {
        let text = format!("{FRONT}\nr[bif.a.b]\nText.\n\n{bad}\nr[bif.c.d]\nMore.\n");
        let p = problems(&text);
        assert_eq!(p.len(), 1, "{bad:?}: {p:?}");
        assert_eq!(p[0].0, 11, "{bad:?}");
        assert!(p[0].1.contains("fence"), "{p:?}");
    }
    // An indented closing fence ends the block in Markdown, not in the parser.
    let text = format!("{FRONT}\nr[bif.a.b]\nText.\n\n```rexx\nsay 1\n  ```\n```\n");
    let p = problems(&text);
    assert_eq!(p.len(), 1, "{p:?}");
    assert_eq!(p[0].0, 13);
    // Inside a block, an indented run with an info string is just text.
    let text = format!("{FRONT}\nr[bif.a.b]\nText.\n\n````markdown\n  ```rexx\n````\n");
    assert!(problems(&text).is_empty());
}

#[test]
fn a_rule_with_no_text_is_reported() {
    for after in ["\n\n> **Note.** Text.\n", "\n```text\nx\n```\n", "\n"] {
        let p = problems(&format!("{FRONT}\nr[bif.a.b]{after}"));
        assert_eq!(p.len(), 1, "{after:?}: {p:?}");
        assert_eq!(p[0].0, 8);
        assert!(p[0].1.contains("no text"), "{p:?}");
    }
}

#[test]
fn a_direct_link_to_a_rule_anchor_is_reported() {
    for link in [
        "[x](substr.md#r-bif.substr.result)",
        "[x](#r-bif.substr.result)",
        "[x](../conventions.md#r-conv.x)",
    ] {
        let p = problems(&format!("{FRONT}\nr[bif.a.b]\nSee {link}.\n"));
        assert_eq!(p.len(), 1, "{link}: {p:?}");
        assert_eq!(p[0].0, 9);
        assert!(p[0].1.contains("r:ID"), "{p:?}");
    }
    let page = parse(&format!(
        "{FRONT}\nr[bif.a.b]\nSee [x](r:bif.a.b) and [y](other.md#heading).\n"
    ))
    .unwrap();
    assert_eq!(page.links.len(), 1);
}

#[test]
fn prose_after_the_first_rule_must_be_a_note() {
    // FRONT is lines 1-6.
    let text = format!(
        "{FRONT}\n# SUBSTR\n\nAn overview, informative.\n\nr[bif.substr.result]\nReturns the part.\n\n## Section\n\n> **Note.** Where it\n> comes from.\n\nStray prose,\ntwo lines.\n\nMore stray prose.\n"
    );
    let found = problems(&text);
    let lines: Vec<usize> = found.iter().map(|(l, _)| *l).collect();
    assert_eq!(lines, [20, 23], "{found:?}");
    assert!(found[0].1.contains("informative note"), "{found:?}");
}
