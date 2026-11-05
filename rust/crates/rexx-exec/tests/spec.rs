//! The specification checker: `spec/` is held to the checks the design lists,
//! and each check is shown to catch what it is for.

mod watchdog;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use rexx_spec::Problem;

/// Where a rule ID is cited from: the file and 1-based line of each citation.
type Citations = BTreeMap<String, Vec<(PathBuf, usize)>>;

/// Every rule ID cited in tracked files under `rust/` of the repository at
/// `repo`.
///
/// A citation is `spec:` followed by rule IDs, inside a `//` or `/* */`
/// comment in a `.rs` or `.rex` file, or on a `#` comment line in any other
/// file under a `testdata` or `*cases*` directory. Tracked files are the
/// ones `git ls-files` names, so an untracked file never counts.
///
/// # Panics
/// If `git ls-files` cannot run in `repo`.
fn citations(repo: &Path) -> Citations {
    let listing = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["ls-files", "-z", "--", "rust"])
        .output()
        .expect("running git ls-files");
    assert!(
        listing.status.success(),
        "git ls-files failed in {}: {}",
        repo.display(),
        String::from_utf8_lossy(&listing.stderr)
    );
    let mut found = Citations::new();
    for name in listing.stdout.split(|b| *b == 0).filter(|n| !n.is_empty()) {
        let relative = PathBuf::from(String::from_utf8_lossy(name).into_owned());
        let comments = match relative.extension().and_then(|e| e.to_str()) {
            Some("rs") => Some(true),
            Some("rex") => Some(false),
            _ => None,
        };
        let in_data = relative
            .parent()
            .into_iter()
            .flat_map(Path::components)
            .filter_map(|c| c.as_os_str().to_str())
            .any(|c| c == "testdata" || c.contains("cases"));
        if comments.is_none() && !in_data {
            continue;
        }
        // Binary or non-UTF-8 files hold no comments to cite from.
        let Ok(text) = fs::read_to_string(repo.join(&relative)) else {
            continue;
        };
        if !text.contains("spec:") {
            continue;
        }
        let segments = match comments {
            Some(rust) => comments_of(&text, rust),
            None => text
                .lines()
                .enumerate()
                .filter(|(_, l)| l.trim_start().starts_with('#'))
                .map(|(i, l)| (i + 1, l.to_string()))
                .collect(),
        };
        for (line, segment) in segments {
            for id in cited_ids(&segment) {
                found
                    .entry(id)
                    .or_default()
                    .push((repo.join(&relative), line));
            }
        }
    }
    found
}

/// The text of every comment in `text`, with the line each segment is on.
/// `rust` selects Rust's literals (a `'` may be a lifetime); otherwise a `'`
/// opens a Rexx string. Block comments nest in both.
fn comments_of(text: &str, rust: bool) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut depth = 0usize;
    for (index, line) in text.lines().enumerate() {
        let chars: Vec<char> = line.chars().collect();
        let at = |i: usize| chars.get(i).copied();
        let mut segment = String::new();
        let mut i = 0;
        while let Some(c) = at(i) {
            if depth > 0 {
                if c == '*' && at(i + 1) == Some('/') {
                    depth -= 1;
                    i += 2;
                    segment.push(' ');
                } else if c == '/' && at(i + 1) == Some('*') {
                    depth += 1;
                    i += 2;
                } else {
                    segment.push(c);
                    i += 1;
                }
                continue;
            }
            match c {
                '"' => {
                    i += 1;
                    while let Some(s) = at(i) {
                        if s == '"' {
                            break;
                        }
                        i += if rust && s == '\\' { 2 } else { 1 };
                    }
                    i += 1;
                }
                '\'' if rust => {
                    // A character literal is skipped; a lifetime is not one.
                    if at(i + 1) == Some('\\') {
                        i += 2;
                        while at(i).is_some_and(|s| s != '\'') {
                            i += 1;
                        }
                        i += 1;
                    } else if at(i + 2) == Some('\'') {
                        i += 3;
                    } else {
                        i += 1;
                    }
                }
                '\'' => {
                    i += 1;
                    while at(i).is_some_and(|s| s != '\'') {
                        i += 1;
                    }
                    i += 1;
                }
                '/' if at(i + 1) == Some('/') => {
                    segment.extend(chars.iter().skip(i + 2));
                    break;
                }
                '/' if at(i + 1) == Some('*') => {
                    depth += 1;
                    i += 2;
                }
                _ => i += 1,
            }
        }
        if !segment.trim().is_empty() {
            out.push((index + 1, segment));
        }
    }
    out
}

/// The rule IDs a comment cites: those following each `spec:`, separated by
/// spaces or commas, up to the first word that is not an ID.
fn cited_ids(comment: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut rest = comment;
    while let Some(at) = rest.find("spec:") {
        let before = rest.get(..at).and_then(|s| s.chars().next_back());
        let after = rest.get(at + "spec:".len()..).unwrap_or("");
        rest = after;
        if before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
            continue;
        }
        for word in after.split(|c: char| c.is_whitespace() || c == ',') {
            if word.is_empty() {
                continue;
            }
            let word = word.trim_matches(|c| matches!(c, '`' | '.' | ';' | ')' | '('));
            if !rexx_spec::is_rule_id(word) {
                break;
            }
            ids.push(word.to_string());
        }
    }
    ids
}

fn problem(path: &Path, line: usize, message: String) -> Problem {
    Problem {
        path: path.to_path_buf(),
        line,
        message,
    }
}

/// Every check on the specification at `spec_root`: the page and cross-page
/// checks `rexx_spec::load` makes, then that every rule has an example or a
/// cited test, that every citation names a rule, that every example's output
/// and exit code are what its page says, that deviations agree with rules and
/// the register, and that rule links resolve. Registers and citations are read
/// from `repo`.
fn check(spec_root: &Path, repo: &Path) -> Vec<Problem> {
    let (pages, mut problems) = rexx_spec::load(spec_root);
    let cited = citations(repo);

    let defined: BTreeSet<&str> = pages
        .iter()
        .flat_map(|p| p.rules.iter().map(|r| r.id.as_str()))
        .collect();

    // 3: a rule has an example under it or a test citing it.
    for page in &pages {
        for rule in &page.rules {
            let exemplified = page.examples.iter().any(|e| e.rule == rule.id);
            if !exemplified && !cited.contains_key(&rule.id) {
                problems.push(problem(
                    &page.path,
                    rule.line,
                    format!("rule `{}` has no example and no test citing it", rule.id),
                ));
            }
        }
    }

    // 4: a citation names a rule.
    for (id, places) in &cited {
        if !defined.contains(id.as_str()) {
            for (path, line) in places {
                problems.push(problem(
                    path,
                    *line,
                    format!("cites `{id}`, which is not a rule in the specification"),
                ));
            }
        }
    }

    // 5: examples run.
    let run_dir = Scratch::new("run");
    for page in &pages {
        for example in &page.examples {
            if let Some(message) = run_example(page, example, &run_dir.0) {
                problems.push(problem(&page.path, example.line, message));
            }
        }
    }

    // 6: deviations.
    let register = fs::read_to_string(repo.join("docs/superpowers/plans/phase-4-exclusions.txt"))
        .unwrap_or_default();
    let registered = |n: u32| {
        let head = format!("{n}. ");
        register
            .lines()
            .any(|l| l.trim_start_matches(' ').starts_with(&head))
    };
    for page in &pages {
        for n in &page.front.deviations {
            let id_suffix = format!(".deviation.{n}");
            if !page.rules.iter().any(|r| r.id.ends_with(&id_suffix)) {
                problems.push(problem(
                    &page.path,
                    1,
                    format!("front matter lists deviation {n}, but no rule ends `{id_suffix}`"),
                ));
            }
            if !registered(*n) {
                problems.push(problem(
                    &page.path,
                    1,
                    format!("deviation {n} has no entry in the deviation register"),
                ));
            }
        }
        for rule in &page.rules {
            let number = rule
                .id
                .rsplit_once(".deviation.")
                .and_then(|(_, n)| n.parse::<u32>().ok());
            if let Some(n) = number
                && !page.front.deviations.contains(&n)
            {
                problems.push(problem(
                    &page.path,
                    rule.line,
                    format!(
                        "rule `{}` is a deviation the front matter does not list",
                        rule.id
                    ),
                ));
            }
        }
    }

    // 8: links resolve.
    for page in &pages {
        for link in &page.links {
            if !defined.contains(link.id.as_str()) {
                problems.push(problem(
                    &page.path,
                    link.line,
                    format!("link to `{}`, which is not a rule", link.id),
                ));
            }
        }
    }

    problems
}

/// Runs one example in process with an empty standard input, bounded as the
/// corpus runs are. `None` when stdout is exactly the output block (its lines,
/// each followed by one `\n`) and the exit code is `rc`.
fn run_example(
    page: &rexx_spec::Page,
    example: &rexx_spec::Example,
    directory: &Path,
) -> Option<String> {
    let name = format!("{}:{}", page.path.display(), example.line);
    let invocation = rexx_exec::Invocation::none()
        .with_directory(directory.to_path_buf())
        .with_input(rexx_exec::ProgramInput::Bytes(Vec::new()));
    let outcome = watchdog::run_bounded(&name, example.program.clone().into_bytes(), invocation);
    if watchdog::did_not_finish(&outcome) {
        return Some(format!(
            "example under `{}` did not finish: {}",
            example.rule,
            String::from_utf8_lossy(&outcome.stderr)
        ));
    }
    let actual = String::from_utf8_lossy(&outcome.stdout);
    if actual == example.output && outcome.exit_code == example.rc {
        return None;
    }
    let mut message = format!(
        "example under `{}` disagrees with its output block",
        example.rule
    );
    if actual != example.output {
        message.push_str(&format!(
            "\n  expected output: {:?}\n  actual output:   {:?}",
            example.output, actual
        ));
    }
    if outcome.exit_code != example.rc {
        message.push_str(&format!(
            "\n  expected rc: {}\n  actual rc:   {}",
            example.rc, outcome.exit_code
        ));
    }
    if !outcome.stderr.is_empty() {
        message.push_str(&format!(
            "\n  stderr: {:?}",
            String::from_utf8_lossy(&outcome.stderr)
        ));
    }
    Some(message)
}

/// A scratch directory removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(what: &str) -> Scratch {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("interrexx-spec-{what}-{}-{n}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("creating a scratch directory");
        Scratch(dir)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root exists")
}

fn render(problems: &[Problem]) -> String {
    problems
        .iter()
        .map(|p| format!("{}:{}: {}", p.path.display(), p.line, p.message))
        .collect::<Vec<_>>()
        .join("\n")
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().expect("a file has a parent")).expect("creating directories");
    fs::write(path, text).expect("writing a file");
}

/// A specification tree holding the introduction and one builtin page, `body`
/// after front matter, with a `SUMMARY.md` listing both.
fn tree_with_page(scratch: &Scratch, body: &str) -> PathBuf {
    let root = scratch.0.join("spec");
    write(&root.join("withdrawn.txt"), "");
    write(
        &root.join("src/SUMMARY.md"),
        "# Summary\n\n- [Intro](introduction.md)\n- [T](builtins/t.md)\n",
    );
    write(
        &root.join("src/introduction.md"),
        "---\nelement: \"Introduction\"\nkind: chapter\n---\n\n# Introduction\n",
    );
    write(
        &root.join("src/builtins/t.md"),
        &format!("---\nelement: T\nkind: builtin\n---\n\n{body}"),
    );
    root
}

fn page_with_example(rexx: &str, output: &str) -> String {
    format!("r[bif.t.one]\nA rule.\n\n```rexx\n{rexx}\n```\n\n```output{output}\n```\n")
}

/// The problems for a tree holding the one example, checked against the real
/// repository (whose citations and register it reads).
fn check_example(rexx: &str, output: &str) -> Vec<Problem> {
    let scratch = Scratch::new("example");
    let root = tree_with_page(&scratch, &page_with_example(rexx, output));
    check(&root, &repo_root())
}

#[test]
fn the_specification_holds() {
    let repo = repo_root();
    let problems = check(&repo.join("spec"), &repo);
    assert!(
        problems.is_empty(),
        "the specification has {} problem(s):\n{}",
        problems.len(),
        render(&problems)
    );
}

#[test]
fn a_seeded_defect_is_caught() {
    let repo = repo_root();
    let scratch = Scratch::new("seeded");
    let copy = scratch.0.join("spec");
    copy_tree(&repo.join("spec"), &copy);
    let summary = fs::read_to_string(copy.join("src/SUMMARY.md")).expect("SUMMARY.md");
    write(
        &copy.join("src/SUMMARY.md"),
        &format!("{summary}- [Seeded](builtins/seeded.md)\n"),
    );
    write(
        &copy.join("src/builtins/seeded.md"),
        "---\nelement: Seeded\nkind: builtin\n---\n\n\
         r[bif.seeded.uncited]\nNothing exercises this.\n\n\
         r[bif.seeded.wrong]\nSee [nothing](r:bif.nope).\n\n\
         ```rexx\nsay 'a'\n```\n\n```output\nb\n```\n",
    );
    let problems = check(&copy, &repo);
    let text = render(&problems);
    assert_eq!(
        problems.len(),
        3,
        "exactly the three seeded defects:\n{text}"
    );
    assert!(
        problems
            .iter()
            .any(|p| p.message.contains("bif.seeded.uncited") && p.message.contains("example")),
        "{text}"
    );
    assert!(
        problems
            .iter()
            .any(|p| p.message.contains("\"a\\n\"") && p.message.contains("\"b\\n\"")),
        "{text}"
    );
    assert!(
        problems.iter().any(|p| p.message.contains("bif.nope")),
        "{text}"
    );
}

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.name=t", "-c", "user.email=t@t"])
        .args(["-c", "commit.gpgsign=false"])
        .args(args)
        .status()
        .expect("running git");
    assert!(status.success(), "git {args:?} failed");
}

#[test]
fn citations_are_comments_in_tracked_files() {
    let scratch = Scratch::new("citations");
    let repo = &scratch.0;
    git(repo, &["init", "-q"]);
    write(
        &repo.join("rust/crates/a/src/lib.rs"),
        "// spec: bif.x.line-comment\n\
         /* spec: bif.x.block, bif.x.second */\n\
         /// spec: bif.x.doc\n\
         fn f(spec: &str) {}\n\
         fn g() { let s = \"// spec: bif.x.in-string\"; }\n\
         fn h() {} // spec: bif.x.trailing.\n",
    );
    write(
        &repo.join("rust/crates/a/tests/cases/k.txt"),
        "# spec: bif.x.hash\nspec: bif.x.not-comment\n",
    );
    write(
        &repo.join("rust/crates/a/tests/other/k.txt"),
        "# spec: bif.x.wrong-dir\n",
    );
    write(
        &repo.join("rust/crates/a/tests/t.rex"),
        "/* spec: bif.x.rex */\nsay 1\n",
    );
    write(&repo.join("docs/notes.rs"), "// spec: bif.x.outside-rust\n");
    git(repo, &["add", "."]);
    git(repo, &["commit", "-q", "-m", "tracked"]);
    write(
        &repo.join("rust/crates/a/src/untracked.rs"),
        "// spec: bif.x.untracked\n",
    );

    let found = citations(repo);
    let ids: Vec<&str> = found.keys().map(String::as_str).collect();
    assert_eq!(
        ids,
        [
            "bif.x.block",
            "bif.x.doc",
            "bif.x.hash",
            "bif.x.line-comment",
            "bif.x.rex",
            "bif.x.second",
            "bif.x.trailing"
        ],
        "{found:?}"
    );
    let at = found.get("bif.x.line-comment").expect("cited");
    assert_eq!(at, &[(repo.join("rust/crates/a/src/lib.rs"), 1)]);
}

#[test]
fn an_example_reading_stdin_ends() {
    let problems = check_example("parse pull x; say \"[\" || x || \"]\"", "\n[]");
    assert!(problems.is_empty(), "{}", render(&problems));
}

#[test]
fn an_example_printing_nothing_has_an_empty_output_block() {
    let problems = check_example("nop", "");
    assert!(problems.is_empty(), "{}", render(&problems));
}

#[test]
fn the_exit_code_is_compared() {
    let ok = check_example("exit 3", " rc=3");
    assert!(ok.is_empty(), "{}", render(&ok));
    let bad = check_example("exit 4", " rc=3");
    assert_eq!(bad.len(), 1, "{}", render(&bad));
    assert!(bad[0].message.contains('3') && bad[0].message.contains('4'));
}

#[test]
fn a_missing_final_newline_is_a_mismatch_shown_on_both_sides() {
    // The block `x` means "x\n"; this program writes "x" with no newline.
    let problems = check_example("call charout , 'x'", "\nx");
    assert_eq!(problems.len(), 1, "{}", render(&problems));
    let m = &problems[0].message;
    assert!(m.contains("\"x\\n\"") && m.contains("\"x\""), "{m}");
    // And the reverse: the program writes a newline the block does not hold.
    let reverse = check_example("say 'x'", "");
    assert_eq!(reverse.len(), 1, "{}", render(&reverse));
    let m = &reverse[0].message;
    assert!(m.contains("\"\"") && m.contains("\"x\\n\""), "{m}");
}

#[test]
fn deviations_match_rules_and_the_register() {
    let scratch = Scratch::new("deviations");
    let root = scratch.0.join("spec");
    write(&root.join("withdrawn.txt"), "");
    write(
        &root.join("src/SUMMARY.md"),
        "# S\n\n- [T](builtins/t.md)\n",
    );
    // Deviation 1 is registered and has its rule; 2 has a rule but is not
    // listed; 999 is listed, unregistered, and has no rule.
    write(
        &root.join("src/builtins/t.md"),
        "---\nelement: T\nkind: builtin\ndeviations: [1, 999]\n---\n\n\
         r[bif.t.deviation.1]\nRule.\n\n```rexx\nnop\n```\n\n```output\n```\n\n\
         r[bif.t.deviation.2]\nRule.\n\n```rexx\nnop\n```\n\n```output\n```\n",
    );
    let problems = check(&root, &repo_root());
    let text = render(&problems);
    assert_eq!(problems.len(), 3, "{text}");
    assert!(
        problems.iter().any(|p| p.message.contains("deviation.999")),
        "{text}"
    );
    assert!(
        problems
            .iter()
            .any(|p| p.message.contains("999") && p.message.contains("register")),
        "{text}"
    );
    assert!(
        problems.iter().any(|p| p.message.contains("deviation.2")),
        "{text}"
    );
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("creating a directory");
    for entry in fs::read_dir(from).expect("reading a directory") {
        let entry = entry.expect("a directory entry");
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).expect("copying a file");
        }
    }
}
