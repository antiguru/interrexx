use std::os::unix::fs::PermissionsExt;

use super::*;
use crate::errors::Message;
use crate::similarity;

/// The invented catalogue text the briefs must not hold.
const THEIRS: &str = "The sprocket &1 refuses to rotate sideways";

fn catalogue() -> Vec<Message> {
    vec![Message {
        major: 91,
        sub: 1,
        number: 100,
        symbol: "Error_Invented_one".into(),
        text: THEIRS.into(),
    }]
}

/// A repository of invented files and the scratch directory beside it:
/// 91.1 has a raise site, a spec rule with an example and a probe; 91.2
/// has none of them; 101.4 carries a constraint.
struct Fixture {
    base: PathBuf,
    sources: Sources,
}

impl Fixture {
    fn new(name: &str) -> Fixture {
        let base = std::env::temp_dir().join(format!(
            "interrexx-brief-test-{}-{name}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        let root = base.join("repo");
        let put = |path: &str, text: &str| {
            let path = root.join(path);
            std::fs::create_dir_all(path.parent().expect("has a parent"))
                .expect("creates a directory");
            std::fs::write(path, text).expect("writes a file");
        };
        put(
            "rust/crates/x/src/raise.rs",
            "use inv::errors::E91_1;\nfn f() {\n    // Reports the unpolished sprocket.\n    let a = 1;\n    Raised::raise(E91_1, [name]);\n    let b = 2;\n}\n",
        );
        put(
            "rust/crates/x/src/tests.rs",
            "fn t() { Raised::raise(E91_1, []); }\n",
        );
        put(
            "spec/src/a.md",
            "# Page\n\nr[page.first]\nA sprocket that cannot rotate raises `91.1`.\nSecond line.\n\nMore.\n\n```rexx\nsay 1\n```\n\n```output\n91.1\n```\n",
        );
        put(
            "rust/crates/x/data/errors.tsv",
            "91\t1\t100\tError_Invented_one\t1\t1\n91\t2\t101\tError_Invented_two\t0\t-\n101\t4\t102\tError_Invented_debug\t0\t-\n",
        );
        put("probes/91.1.rex", "say 'code 91.1'\n");
        put(
            "probes/unreachable.tsv",
            "# a comment\n\n91.2\tno-site\tnothing raises it here\n",
        );
        let rexx = base.join("rexx");
        std::fs::write(&rexx, "#!/bin/sh\necho \"ran $(basename \"$1\")\"\n")
            .expect("writes the stand-in interpreter");
        std::fs::set_permissions(&rexx, std::fs::Permissions::from_mode(0o755))
            .expect("makes it executable");
        let sources = Sources {
            root: root.clone(),
            errors: root.join("rust/crates/x/data/errors.tsv"),
            probes: root.join("probes"),
            code: vec![root.join("rust/crates")],
            spec: root.join("spec/src"),
            rexx,
        };
        Fixture { base, sources }
    }

    fn out(&self) -> PathBuf {
        self.base.join("briefs")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.base);
    }
}

#[test]
fn a_brief_names_the_interface_the_site_the_rule_and_the_probe_output() {
    let fx = Fixture::new("content");
    let briefs = build(&fx.sources, 91).expect("builds");
    assert_eq!(briefs.len(), 2);
    let (sub, one) = &briefs[0];
    assert_eq!(*sub, 1);
    assert!(one.contains("Error_Invented_one"));
    assert!(one.contains("`&1`"));
    assert!(one.contains("raise.rs:5"), "{one}");
    assert!(one.contains("Raised::raise(E91_1, [name]);"));
    assert!(!one.contains("tests.rs"), "test sites are not raise sites");
    assert!(!one.contains("use inv::errors"), "imports are not sites");
    assert!(one.contains("rule `page.first`"), "{one}");
    assert!(one.contains("```rexx") && one.contains("say 'code 91.1'"));
    assert!(
        one.contains("ran 91.1.rex"),
        "the probe's output is recorded: {one}"
    );
    let (_, two) = &briefs[1];
    assert!(two.contains("No raise site found"));
    assert!(two.contains("No rule names this code"));
    assert!(two.contains("No probe program"));
    assert!(
        two.contains("no-site: nothing raises it here"),
        "a listed code says why it has no probe: {two}"
    );
}

#[test]
fn a_code_missing_from_the_list_says_only_that_it_has_no_probe() {
    let fx = Fixture::new("unlisted");
    std::fs::remove_file(fx.sources.probes.join(UNREACHABLE)).expect("removes the list");
    let (_, two) = build(&fx.sources, 91).expect("builds").remove(1);
    assert!(two.contains("No probe program for this code."), "{two}");
}

#[test]
fn a_list_row_without_three_fields_is_an_error() {
    let fx = Fixture::new("badlist");
    std::fs::write(fx.sources.probes.join(UNREACHABLE), "91.2\tno-site\n").expect("writes");
    let error = build(&fx.sources, 91).expect_err("refuses");
    assert!(error.contains("code, class and reason"), "{error}");
}

/// Whether the source `text` mentions `major.sub` as a raise site, however
/// the call is wrapped: the constant `E<major>_<sub>`, or the number pair
/// after an opening parenthesis (blanks and line breaks ignored).
fn mentions_site(text: &str, major: u16, sub: u16) -> bool {
    let squeezed: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    let bounded = |haystack: &str, needle: &str, before: fn(char) -> bool| {
        haystack.match_indices(needle).any(|(at, _)| {
            let after = haystack[at + needle.len()..].chars().next();
            let first = haystack[..at].chars().next_back();
            !after.is_some_and(|c| c.is_ascii_digit()) && !first.is_some_and(before)
        })
    };
    bounded(&squeezed, &format!("E{major}_{sub}"), |c| {
        c.is_alphanumeric() || c == '_'
    }) || squeezed
        .match_indices(&format!("({major},{sub}"))
        .any(|(at, needle)| {
            !squeezed[at + needle.len()..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_digit())
        })
}

#[test]
fn a_site_is_found_whatever_the_line_breaks() {
    assert!(mentions_site(
        "Raised::syntax(\n    88,\n    919,\n x)",
        88,
        919
    ));
    assert!(mentions_site("Raised::raise(E88_919, [])", 88, 919));
    assert!(!mentions_site("Raised::raise(E88_9190, [])", 88, 919));
    assert!(!mentions_site("Raised::syntax(88, 9190, [])", 88, 919));
    assert!(!mentions_site("fn f() { g(188, 919) }", 88, 919));
    assert!(!mentions_site("Raised::raise(XE88_919, [])", 88, 919));
}

/// The list's `no-site` rows keep their claim: the interpreter sources
/// mention none of those codes as a raise site.
#[test]
fn a_no_site_row_names_a_code_no_source_raises() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let probes = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/probes");
    let mut code: Vec<PathBuf> = std::fs::read_dir(&crates)
        .expect("lists the crates")
        .filter_map(Result::ok)
        .filter(|e| e.file_name() != "interrexx-upstream")
        .map(|e| e.path().join("src"))
        .filter(|p| p.is_dir())
        .collect();
    code.sort();
    let texts: Vec<String> = source_files(&code, "rs")
        .expect("lists the sources")
        .iter()
        .map(|f| std::fs::read_to_string(f).expect("reads a source"))
        .collect();
    let mut raised = Vec::new();
    for (code_name, class, _) in unreachable(&probes).expect("reads the list") {
        if class != "no-site" {
            continue;
        }
        let (major, sub) = code_name.split_once('.').expect("a code is major.sub");
        let (major, sub): (u16, u16) =
            (major.parse().expect("a major"), sub.parse().expect("a sub"));
        if texts.iter().any(|t| mentions_site(t, major, sub)) {
            raised.push(code_name);
        }
    }
    assert!(
        raised.is_empty(),
        "listed as having no raise site, yet a source raises: {raised:?}"
    );
}

#[test]
fn the_rule_shows_the_example_block_that_holds_the_code() {
    let fx = Fixture::new("example");
    let (_, one) = build(&fx.sources, 91).expect("builds").remove(0);
    // The code sits in the `output` block, not the first one.
    let shown = one.split("```text").nth(1).expect("an example is shown");
    assert!(shown.starts_with("\n91.1\n"), "{shown}");
}

#[test]
fn the_debug_texts_carry_the_plus_and_colon_constraint() {
    let fx = Fixture::new("constraint");
    let briefs = build(&fx.sources, 101).expect("builds");
    let text = &briefs[0].1;
    assert!(text.contains("## Constraints"));
    assert!(text.contains("starts with `+++`") || text.contains("starts each debug error line"));
    assert!(text.contains("ends in a colon"));
    let plain = build(&fx.sources, 91).expect("builds");
    assert!(!plain[0].1.contains("## Constraints"));
}

#[test]
fn a_brief_holds_no_catalogue_text() {
    let fx = Fixture::new("leak");
    let patterns: Vec<Pattern> = catalogue().iter().map(Pattern::new).collect();
    let written = write(&fx.sources, 91, &fx.out(), Some(&patterns)).expect("writes");
    assert_eq!(written, 2);
    for name in ["91.1.md", "91.2.md"] {
        let text = std::fs::read_to_string(fx.out().join(name)).expect("reads the brief");
        assert!(Haystack::new(&text, false).matches(&patterns).is_empty());
        assert!(similarity::compare(&text, THEIRS).run < similarity::Threshold::default().run);
    }
}

#[test]
fn a_brief_that_quotes_the_catalogue_is_refused() {
    let fx = Fixture::new("planted");
    let raise = fx.sources.root.join("rust/crates/x/src/raise.rs");
    let source = std::fs::read_to_string(&raise).expect("reads the source");
    std::fs::write(
        &raise,
        source.replace(
            "Reports the unpolished sprocket.",
            "The sprocket x refuses to rotate sideways.",
        ),
    )
    .expect("plants the text");
    let patterns: Vec<Pattern> = catalogue().iter().map(Pattern::new).collect();
    let error = write(&fx.sources, 91, &fx.out(), Some(&patterns)).expect_err("refuses");
    assert!(error.contains("91.1"), "{error}");
    assert!(!fx.out().join("91.1.md").exists());
}

#[test]
fn an_output_directory_inside_the_repository_is_refused() {
    let fx = Fixture::new("inside");
    let error = write(&fx.sources, 91, &fx.sources.root.join("briefs"), None).expect_err("refuses");
    assert!(error.contains("inside the repository"), "{error}");
}

#[test]
fn a_major_without_codes_is_an_error() {
    let fx = Fixture::new("nomajor");
    assert!(build(&fx.sources, 5).is_err());
}

#[test]
fn raising_is_recognised_by_constant_and_by_call() {
    let found = |text: &str, major, sub| sites(text, major, sub);
    assert_eq!(found("Raised::raise(E40_9, [a])", 40, 9), [(0, 0)]);
    assert!(found("Raised::raise(E40_90, [a])", 40, 9).is_empty());
    assert!(found("use x::E40_9;", 40, 9).is_empty());
    assert_eq!(found("    Raised::syntax(40, 9, vec![])", 40, 9), [(0, 0)]);
    assert!(found("    Raised::syntax(40, 90, vec![])", 40, 9).is_empty());
    assert!(found("    Raised::syntax(140, 9, vec![])", 40, 9).is_empty());
    assert!(found("    point(40, 9)", 40, 9).is_empty());
}

#[test]
fn a_call_wrapped_over_lines_is_a_site_and_its_context_covers_the_call() {
    let wrapped =
        "fn f() {\n    Raised::syntax(\n        88,\n        919,\n        [x],\n    )\n}\n";
    assert_eq!(sites(wrapped, 88, 919), [(1, 3)]);
    assert!(sites(wrapped, 88, 91).is_empty());
    let fx = Fixture::new("wrapped");
    let path = fx.sources.root.join("rust/crates/x/src/wrapped.rs");
    std::fs::write(&path, wrapped.replace("88", "91").replace("919", "2")).expect("writes");
    let (_, two) = build(&fx.sources, 91).expect("builds").remove(1);
    assert!(!two.contains("No raise site found"), "{two}");
    assert!(two.contains("wrapped.rs:2"), "{two}");
    assert!(two.contains("[x],"), "the whole call is shown: {two}");
}

#[test]
fn a_path_in_the_probe_output_becomes_a_placeholder() {
    let fx = Fixture::new("paths");
    let probe = fx.sources.probes.join("91.1.rex");
    let root = fx.sources.root.canonicalize().expect("resolves the root");
    // The stand-in interpreter prints its argument, the root and its working
    // directory.
    std::fs::write(
        &fx.sources.rexx,
        "#!/bin/sh\necho \"path $1\"\necho \"root %ROOT%/x\"\necho \"cwd $(pwd)\"\n"
            .replace("%ROOT%", &root.display().to_string()),
    )
    .expect("writes the stand-in interpreter");
    let (_, one) = build(&fx.sources, 91).expect("builds").remove(0);
    assert!(one.contains("path <probe>"), "{one}");
    assert!(one.contains("root <repo>/x"), "{one}");
    assert!(one.contains("cwd <scratch>"), "{one}");
    assert!(!one.contains(&root.display().to_string()), "{one}");
    assert!(!one.contains(&probe.display().to_string()), "{one}");
}
