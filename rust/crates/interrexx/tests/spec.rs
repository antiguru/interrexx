//! The specification checker: `spec/` is held to the checks the design lists,
//! and each check is shown to catch what it is for.

mod support;
mod watchdog;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use interrexx_spec::Problem;
use support::oracle::{Oracle, Termination};

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
///
/// The lexer carries its state across lines, so a `//` inside a string that
/// continues over several lines, or inside a raw string, is not a comment.
/// `rust` selects Rust's literals (ordinary strings with escapes, raw strings,
/// character literals against lifetimes); otherwise a `'` or `"` opens a Rexx
/// string, which ends at its closing quote or at the end of the line (a doubled
/// quote is a close and a reopen). Block comments nest in both.
fn comments_of(text: &str, rust: bool) -> Vec<(usize, String)> {
    enum State {
        Code,
        Block(usize),
        Str(char),
        Raw(usize),
    }
    let chars: Vec<char> = text.chars().collect();
    let mut line_of = Vec::with_capacity(chars.len());
    let mut line = 1;
    for c in &chars {
        line_of.push(line);
        if *c == '\n' {
            line += 1;
        }
    }
    let at = |i: usize| chars.get(i).copied();
    let ident = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '_');
    let mut out = Vec::new();
    let mut state = State::Code;
    let mut segment = String::new();
    let mut segment_line = 1;
    let mut i = 0;
    while let Some(c) = at(i) {
        let here = line_of.get(i).copied().unwrap_or(1);
        match state {
            State::Block(depth) => {
                if c == '*' && at(i + 1) == Some('/') {
                    i += 2;
                    segment.push(' ');
                    state = if depth == 1 {
                        flush(&mut out, &mut segment, segment_line);
                        State::Code
                    } else {
                        State::Block(depth - 1)
                    };
                } else if c == '/' && at(i + 1) == Some('*') {
                    i += 2;
                    state = State::Block(depth + 1);
                } else if c == '\n' {
                    flush(&mut out, &mut segment, segment_line);
                    i += 1;
                    segment_line = here + 1;
                } else {
                    segment.push(c);
                    i += 1;
                }
            }
            State::Str(quote) => {
                if rust && c == '\\' {
                    i += 2;
                } else if c == quote {
                    i += 1;
                    state = State::Code;
                } else if !rust && c == '\n' {
                    state = State::Code;
                } else {
                    i += 1;
                }
            }
            State::Raw(hashes) => {
                if c == '"' && (1..=hashes).all(|k| at(i + k) == Some('#')) {
                    i += 1 + hashes;
                    state = State::Code;
                } else {
                    i += 1;
                }
            }
            State::Code => match c {
                '"' => {
                    i += 1;
                    state = State::Str('"');
                }
                'r' if rust
                    && {
                        let prev = i.checked_sub(1).and_then(at);
                        let before_b = prev == Some('b') && !ident(i.checked_sub(2).and_then(at));
                        !ident(prev) || before_b
                    }
                    && {
                        let mut k = i + 1;
                        while at(k) == Some('#') {
                            k += 1;
                        }
                        at(k) == Some('"')
                    } =>
                {
                    let mut k = i + 1;
                    while at(k) == Some('#') {
                        k += 1;
                    }
                    state = State::Raw(k - i - 1);
                    i = k + 1;
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
                    state = State::Str('\'');
                }
                '/' if at(i + 1) == Some('/') => {
                    let mut k = i + 2;
                    let mut body = String::new();
                    while let Some(s) = at(k).filter(|s| *s != '\n') {
                        body.push(s);
                        k += 1;
                    }
                    flush(&mut out, &mut body, here);
                    i = k;
                }
                '/' if at(i + 1) == Some('*') => {
                    segment_line = here;
                    state = State::Block(1);
                    i += 2;
                }
                _ => i += 1,
            },
        }
    }
    flush(&mut out, &mut segment, segment_line);
    out
}

fn flush(out: &mut Vec<(usize, String)>, segment: &mut String, line: usize) {
    if !segment.trim().is_empty() {
        out.push((line, std::mem::take(segment)));
    }
    segment.clear();
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
            if !interrexx_spec::is_rule_id(word) {
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
/// checks `interrexx_spec::load` makes, then that every rule has an example or a
/// cited test, that every citation names a rule, that every example's output
/// and exit code are what its page says, that deviations agree with rules and
/// the Deviations table, and that rule links resolve. The table is read from
/// `repo`; `cited` is every citation the checked tree is held to.
fn check(spec_root: &Path, repo: &Path, cited: &Citations) -> Vec<Problem> {
    let (pages, mut problems) = interrexx_spec::load(spec_root);

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
    for (id, places) in cited {
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

    // 5: examples run. The examples of every check in this process share its API
    // service, whose queues and registrations are global, so the checks take
    // turns.
    static EXAMPLES: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _turn = EXAMPLES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let run_dir = Scratch::new("run");
    for page in &pages {
        for example in &page.examples {
            if let Some(message) = run_example(page, example, &run_dir.0) {
                problems.push(problem(&page.path, example.line, message));
            }
        }
    }

    // 6: deviations.
    let table = fs::read_to_string(repo.join(support::deviations::TABLE)).unwrap_or_default();
    let table = support::deviations::parse(&table).unwrap_or_default();
    let registered = |n: u32| table.get(&n).is_some_and(|d| d.active);
    for page in &pages {
        for n in &page.front.deviations {
            let id_suffix = format!(".deviation.{n}");
            if !page
                .rules
                .iter()
                .any(|r| interrexx_spec::deviation_number(&r.id) == Some(*n))
            {
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
                    format!("deviation {n} is not an active entry of the Deviations chapter"),
                ));
            }
        }
        for rule in &page.rules {
            if let Some(n) = interrexx_spec::deviation_number(&rule.id)
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

/// Points this process's API connection at a service of its own, so the
/// examples that use external data queues never reach the user's service, and
/// starts a watcher that stops that service and removes its directory once this
/// process has ended. The directory is directly under the temp directory, so the
/// service's socket path stays within the platform's limit wherever the
/// checkout is. The daemon is built first when the target directory lacks it.
///
/// # Panics
/// If the daemon cannot be built; every later call panics with the same
/// message.
fn private_service() {
    static CHOSEN: std::sync::OnceLock<Result<(), String>> = std::sync::OnceLock::new();
    let chosen = CHOSEN.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("interrexx-spec-api-{}", std::process::id()));
        let daemon = Path::new(env!("CARGO_BIN_EXE_rexx")).with_file_name("rxapi");
        if !daemon.is_file() {
            build_daemon(&daemon)?;
        }
        interrexx::use_api_service(&dir, &daemon);
        let _ = Command::new("sh")
            .arg("-c")
            .arg(
                "while kill -0 \"$0\" 2>/dev/null; do sleep 1; done; \
                 REXX_API_DIR=\"$1\" \"$2\" --stop >/dev/null 2>&1; rm -rf \"$1\"",
            )
            .arg(std::process::id().to_string())
            .arg(&dir)
            .arg(&daemon)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
        Ok(())
    });
    if let Err(message) = chosen {
        panic!("{message}");
    }
}

/// Builds `interrexx-rxapi`'s `rxapi` binary to `daemon`, the path of a binary
/// in a profile directory of a target directory, with that profile and target
/// directory. Cargo's lock on the target directory makes test processes that
/// build it at the same time wait for each other.
///
/// # Errors
/// If `daemon` is not in a profile directory of a target directory, cargo
/// cannot run, or the build fails or leaves no file at `daemon`; the message
/// of a failed build holds cargo's standard error.
fn build_daemon(daemon: &Path) -> Result<(), String> {
    let in_target = daemon
        .parent()
        .and_then(|p| Some((p.parent()?, p.file_name()?.to_str()?)));
    let Some((target_dir, profile)) = in_target else {
        return Err(format!(
            "{} is not in a profile directory of a target directory",
            daemon.display()
        ));
    };
    let profile = if profile == "debug" { "dev" } else { profile };
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Command::new(&cargo)
        .args([
            "build",
            "--locked",
            "-p",
            "interrexx-rxapi",
            "--bin",
            "rxapi",
        ])
        .args(["--profile", profile])
        .arg("--target-dir")
        .arg(target_dir)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|e| format!("running {}: {e}", Path::new(&cargo).display()))?;
    if output.status.success() && daemon.is_file() {
        return Ok(());
    }
    Err(format!(
        "building {} failed ({}):\n{}",
        daemon.display(),
        output.status,
        String::from_utf8_lossy(&output.stderr)
    ))
}

/// Runs one example in process with an empty standard input, bounded as the
/// corpus runs are. `None` when stdout is exactly the output block (its lines,
/// each followed by one `\n`) and the exit code is `rc`. An example marked
/// `defect=#N` is the reverse: a mismatch is expected, and a match is reported
/// so the marker is removed.
fn run_example(
    page: &interrexx_spec::Page,
    example: &interrexx_spec::Example,
    directory: &Path,
) -> Option<String> {
    private_service();
    let name = format!("{}:{}", page.path.display(), example.line);
    let invocation = interrexx::Invocation::none()
        .with_directory(directory.to_path_buf())
        .with_input(interrexx::ProgramInput::Bytes(Vec::new()));
    let outcome = watchdog::run_bounded(&name, example.program.clone().into_bytes(), invocation);
    if watchdog::did_not_finish(&outcome) {
        return Some(format!(
            "example under `{}` did not finish: {}",
            example.rule,
            String::from_utf8_lossy(&outcome.stderr)
        ));
    }
    let actual = String::from_utf8_lossy(&outcome.stdout);
    let matches = actual == example.output && outcome.exit_code == example.rc;
    if let Some(issue) = example.defect {
        return matches.then(|| {
            format!(
                "example under `{}`: defect #{issue} no longer reproduces: remove the marker",
                example.rule
            )
        });
    }
    if matches {
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
    fn next() -> usize {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }

    fn new(what: &str) -> Scratch {
        let n = Scratch::next();
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
    tree_with_front(scratch, "", body)
}

/// As `tree_with_page`, with `front` (whole `key: value` lines) added to the
/// builtin page's front matter.
fn tree_with_front(scratch: &Scratch, front: &str, body: &str) -> PathBuf {
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
        &format!("---\nelement: T\nkind: builtin\n{front}---\n\n{body}"),
    );
    root
}

fn page_with_example(rexx: &str, output: &str) -> String {
    format!("r[bif.t.one]\nA rule.\n\n```rexx\n{rexx}\n```\n\n```output{output}\n```\n")
}

/// The problems for a tree holding the one example, checked as
/// [`check_fixture`] checks.
fn check_example(rexx: &str, output: &str) -> Vec<Problem> {
    let scratch = Scratch::new("example");
    let root = tree_with_page(&scratch, &page_with_example(rexx, output));
    check_fixture(&root)
}

/// [`check`] of a fixture tree against the real repository's Deviations
/// table, held to no citations: the repository's citations name rules of the
/// specification, which a fixture tree does not hold.
fn check_fixture(spec_root: &Path) -> Vec<Problem> {
    check(spec_root, &repo_root(), &Citations::new())
}

#[test]
fn the_specification_holds() {
    let repo = repo_root();
    let problems = check(&repo.join("spec"), &repo, &citations(&repo));
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
    let problems = check(&copy, &repo, &citations(&repo));
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
fn comment_markers_inside_literals_are_not_comments() {
    let multi = "let s = \"first \\\n    // spec: bif.x.continued\\\n    /* spec: bif.x.block */\";\n// spec: bif.x.real\n";
    let found = comments_of(multi, true);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].0, 4);
    let raw = "let s = r#\"\n// spec: bif.x.raw \"quoted\"\n\"#;\nfn f<'a>(x: &'a str) {} // spec: bif.x.after\n";
    let found = comments_of(raw, true);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].1.contains("bif.x.after"));
    let rex = "say '/* spec: bif.x.str */' /* spec: bif.x.rex-real */\nsay \"it''s\" /* spec: bif.x.two\n   bif.x.three */\n";
    let found = comments_of(rex, false);
    assert_eq!(found.len(), 3, "{found:?}");
    assert_eq!((found[0].0, found[1].0, found[2].0), (1, 2, 3));
}

#[test]
fn the_checker_s_own_fixtures_cite_nothing() {
    let found = citations(&repo_root());
    let own = repo_root().join("rust/crates/interrexx/tests/spec.rs");
    let from_here: Vec<_> = found
        .iter()
        .filter(|(_, places)| places.iter().any(|(p, _)| *p == own))
        .collect();
    assert!(from_here.is_empty(), "{from_here:?}");
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
fn a_defect_example_that_still_fails_is_accepted() {
    let ok = check_example("say 'a'", " defect=#7\nb");
    assert!(ok.is_empty(), "{}", render(&ok));
    let rc = check_example("exit 4", " defect=#7 rc=3");
    assert!(rc.is_empty(), "{}", render(&rc));
}

#[test]
fn a_defect_example_that_now_passes_is_reported() {
    let bad = check_example("say 'a'", " defect=#7\na");
    assert_eq!(bad.len(), 1, "{}", render(&bad));
    assert!(
        bad[0].message.contains("defect #7 no longer reproduces"),
        "{}",
        bad[0].message
    );
}

#[test]
fn a_malformed_defect_marker_is_reported() {
    let bad = check_example("say 'a'", " defect=7\na");
    assert_eq!(bad.len(), 1, "{}", render(&bad));
    assert!(bad[0].message.contains("defect=#N"), "{}", bad[0].message);
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
fn deviations_match_rules_and_the_table() {
    let scratch = Scratch::new("deviations");
    let root = scratch.0.join("spec");
    write(&root.join("withdrawn.txt"), "");
    write(
        &root.join("src/SUMMARY.md"),
        "# S\n\n- [T](builtins/t.md)\n",
    );
    // Deviation 1 is active and has its rule; 2 has a rule but is not
    // listed; 5 is listed with its rule but withdrawn; 999 is listed, absent
    // from the table, and has no rule.
    write(
        &root.join("src/builtins/t.md"),
        "---\nelement: T\nkind: builtin\ndeviations: [1, 5, 999]\n---\n\n\
         r[bif.t.deviation.1]\nRule.\n\n```rexx\nnop\n```\n\n```output\n```\n\n\
         r[bif.t.deviation.2]\nRule.\n\n```rexx\nnop\n```\n\n```output\n```\n\n\
         r[bif.t.deviation.5]\nRule.\n\n```rexx\nnop\n```\n\n```output\n```\n",
    );
    let problems = check_fixture(&root);
    let text = render(&problems);
    assert_eq!(problems.len(), 4, "{text}");
    assert!(
        problems.iter().any(|p| p.message.contains("deviation.999")),
        "{text}"
    );
    assert!(
        problems
            .iter()
            .any(|p| p.message.contains("999") && p.message.contains("active entry")),
        "{text}"
    );
    assert!(
        problems
            .iter()
            .any(|p| p.message.contains("deviation 5 ") && p.message.contains("active entry")),
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

/// Whether the oracle gate is on: `REXX_SPEC_ORACLE` set, non-empty and not `0`.
fn oracle_gate_on() -> bool {
    std::env::var_os("REXX_SPEC_ORACLE").is_some_and(|v| !v.is_empty() && v != "0")
}

/// Whether `example` is under a deviation rule its page lists: the rule's ID
/// ends `.deviation.N` and `N` is in the page's `deviations`.
fn exempt_from_the_oracle(page: &interrexx_spec::Page, example: &interrexx_spec::Example) -> bool {
    interrexx_spec::deviation_number(&example.rule)
        .is_some_and(|n| page.front.deviations.contains(&n))
}

/// Runs every example under `spec_root` on the oracle, with empty standard
/// input, and reports each whose stdout is not exactly the output block or
/// whose exit code is not `rc`. An example under a rule `….deviation.N`
/// whose page lists `N` in its `deviations` is the place the oracle is known
/// to answer differently, and is exempt.
fn oracle_disagreements(spec_root: &Path, oracle: &Oracle) -> Vec<Problem> {
    let (pages, mut problems) = interrexx_spec::load(spec_root);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "spec-oracle-{}-{}",
        std::process::id(),
        Scratch::next()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("creating the oracle program directory");
    let mut n = 0;
    for page in &pages {
        for example in &page.examples {
            if exempt_from_the_oracle(page, example) {
                continue;
            }
            n += 1;
            let file = dir.join(format!("example-{n}.rex"));
            fs::write(&file, &example.program).expect("writing an example program");
            let outcome = oracle.run_with(&file, &[], None);
            let message = match outcome.termination {
                Termination::Exited(code) => {
                    let mut message = String::new();
                    if outcome.stdout != example.output.as_bytes() {
                        message.push_str(&format!(
                            "\n  expected output: {:?}\n  oracle output:   {:?}",
                            example.output,
                            String::from_utf8_lossy(&outcome.stdout)
                        ));
                    }
                    if code != example.rc {
                        message.push_str(&format!(
                            "\n  expected rc: {}\n  oracle rc:   {code}",
                            example.rc
                        ));
                    }
                    message
                }
                other => format!("\n  the oracle did not finish: {other:?}"),
            };
            if !message.is_empty() {
                problems.push(problem(
                    &page.path,
                    example.line,
                    format!(
                        "example under `{}` is answered differently by the oracle, and the rule is not a deviation{message}",
                        example.rule
                    ),
                ));
            }
        }
    }
    let _ = fs::remove_dir_all(&dir);
    problems
}

#[test]
fn examples_agree_with_the_oracle_or_are_deviations() {
    if !oracle_gate_on() {
        eprintln!("note: REXX_SPEC_ORACLE is not set; the examples were not run on the oracle");
        return;
    }
    let oracle = support::oracle::locate();
    let repo = repo_root();
    let problems = oracle_disagreements(&repo.join("spec"), &oracle);
    assert!(
        problems.is_empty(),
        "{} example(s) differ on the oracle:\n{}",
        problems.len(),
        render(&problems)
    );
}

#[test]
fn an_example_the_oracle_answers_differently_is_reported_unless_a_deviation() {
    if !oracle_gate_on() {
        eprintln!("note: REXX_SPEC_ORACLE is not set; the oracle self-test was skipped");
        return;
    }
    let oracle = support::oracle::locate();
    let scratch = Scratch::new("oracle-seeded");
    let root = tree_with_page(&scratch, &page_with_example("say 1/3", "\nx"));
    let problems = oracle_disagreements(&root, &oracle);
    assert_eq!(problems.len(), 1, "{}", render(&problems));
    let m = &problems[0].message;
    assert!(
        m.contains("bif.t.one") && m.contains("\"x\\n\"") && m.contains("0.3333333"),
        "{m}"
    );

    // The same disagreement under a deviation rule the page lists is exempt;
    // an rc mismatch under a non-deviation rule is reported.
    let root = tree_with_front(
        &scratch,
        "deviations: [1]\n",
        "r[bif.t.deviation.1]\nA rule.\n\n```rexx\nsay 1/3\n```\n\n```output\nx\n```\n\n\
         r[bif.t.two]\nA rule.\n\n```rexx\nexit 2\n```\n\n```output\n```\n",
    );
    let problems = oracle_disagreements(&root, &oracle);
    assert_eq!(problems.len(), 1, "{}", render(&problems));
    assert!(
        problems[0].message.contains("bif.t.two"),
        "{}",
        problems[0].message
    );
    assert!(
        problems[0].message.contains("oracle rc:   2"),
        "{}",
        problems[0].message
    );

    // A defect marker is not an oracle exemption: the wrong body is reported.
    let root = tree_with_page(&scratch, &page_with_example("say 1/3", " defect=#7\nx"));
    let problems = oracle_disagreements(&root, &oracle);
    assert_eq!(problems.len(), 1, "{}", render(&problems));

    // Under a deviation rule the page does not list, it is reported.
    let root = tree_with_page(
        &scratch,
        "r[bif.t.deviation.1]\nA rule.\n\n```rexx\nsay 1/3\n```\n\n```output\nx\n```\n",
    );
    let problems = oracle_disagreements(&root, &oracle);
    assert_eq!(problems.len(), 1, "{}", render(&problems));
    assert!(
        problems[0].message.contains("bif.t.deviation.1"),
        "{}",
        problems[0].message
    );
}

#[test]
fn only_a_listed_deviation_rule_is_exempt_from_the_oracle() {
    let page = |deviations: &str, rule: &str| {
        let text = format!(
            "---\nelement: T\nkind: builtin\n{deviations}---\n\n\
             r[{rule}]\nA rule.\n\n```rexx\nnop\n```\n\n```output\n```\n"
        );
        interrexx_spec::parse_page(Path::new("builtins/t.md"), &text)
    };
    let exempt = |deviations: &str, rule: &str| {
        let page = page(deviations, rule).expect("the page parses");
        exempt_from_the_oracle(&page, &page.examples[0])
    };
    assert!(exempt("deviations: [1]\n", "bif.t.deviation.1"));
    assert!(!exempt("", "bif.t.deviation.1"));
    assert!(!exempt("deviations: [2]\n", "bif.t.deviation.1"));
    assert!(!exempt("deviations: [1]\n", "bif.t.one"));
    // A defect marker does not exempt an example from the oracle.
    let marked = interrexx_spec::parse_page(
        Path::new("builtins/t.md"),
        "---\nelement: T\nkind: builtin\n---\n\n\
         r[bif.t.one]\nA rule.\n\n```rexx\nnop\n```\n\n```output defect=#7\n```\n",
    )
    .expect("the page parses");
    assert_eq!(marked.examples[0].defect, Some(7));
    assert!(!exempt_from_the_oracle(&marked, &marked.examples[0]));
    // An ID that only contains `.deviation.` is not a deviation rule at all:
    // the page does not parse, so no example of it reaches the oracle.
    assert!(page("deviations: [1]\n", "bif.t.deviation.pending").is_err());
}

#[test]
fn the_deviations_chapter_and_table_agree() {
    let repo = repo_root();
    let chapter = fs::read_to_string(repo.join(support::deviations::CHAPTER))
        .expect("reading the Deviations chapter");
    let headings = support::deviations::chapter_headings(&chapter);
    let in_chapter: BTreeMap<u32, bool> = headings.iter().copied().collect();
    assert_eq!(
        in_chapter.len(),
        headings.len(),
        "a Deviation number is a heading twice"
    );
    let in_table: BTreeMap<u32, bool> = support::deviations::load(&repo)
        .into_iter()
        .map(|(n, d)| (n, d.active))
        .collect();
    assert_eq!(
        in_chapter, in_table,
        "the chapter's `## N.` headings and the table's rows (number, active) differ"
    );
}

#[test]
fn a_chapter_heading_is_matched_in_its_own_format() {
    let text = "## 4. An entry\n## 5. Withdrawn\n### 6. Deeper\n7. Prose\n## Intro\n";
    assert_eq!(
        support::deviations::chapter_headings(text),
        [(4, true), (5, false)]
    );
    let table = "# comment\n4\tactive\t-\t-\n5\twithdrawn\tstream_model\ta,b\n";
    let rows = support::deviations::parse(table).expect("a table");
    assert!(rows[&4].active && !rows[&5].active);
    assert_eq!(rows[&5].witnesses, ["a", "b"]);
    assert!(support::deviations::parse("4\tactive\t-\t-\n4\tactive\t-\t-\n").is_err());
    assert!(support::deviations::parse("4\tlive\t-\t-\n").is_err());
}
