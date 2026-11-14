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
    assert!(raises("Raised::raise(E40_9, [a])", 40, 9));
    assert!(!raises("Raised::raise(E40_90, [a])", 40, 9));
    assert!(!raises("use x::E40_9;", 40, 9));
    assert!(raises("    Raised::syntax(40, 9, vec![])", 40, 9));
    assert!(!raises("    Raised::syntax(40, 90, vec![])", 40, 9));
    assert!(!raises("    point(40, 9)", 40, 9));
}
