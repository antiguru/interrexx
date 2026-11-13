//! Writes the brief a message-text writer works from, one per code.
//!
//! A brief holds what the writer needs and no catalogue text: the code, its
//! symbolic name, which inserts it takes and shows, the places interrexx
//! raises it with the lines that build its inserts, the spec rules that
//! mention it, and a probe program with its recorded output. The probe
//! prints the code and `ADDITIONAL` only, so no text reaches the brief
//! through it. The writer runs nothing; [`write`] runs each probe here.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use crate::errors::{Row, Table};
use crate::scan::{Haystack, Pattern};

/// The longest a probe may run.
const PROBE_TIME: Duration = Duration::from_secs(30);
/// The most raise sites and spec rules one brief lists.
const MOST_SITES: usize = 6;
const MOST_RULES: usize = 8;
/// The most of a probe's output kept.
const MOST_OUTPUT: usize = 4000;

/// Constraints on a text that the spec states for a use of the text.
/// The debug lines are `+++` lines, and the interpreter appends the error
/// number after the text of 101.4.
const CONSTRAINTS: &[((u16, u16), &str)] = &[
    (
        (101, 4),
        "This text starts each debug error line, and the interpreter appends the \
         number of the error after it (`N:` on the first line, `N.M:` on the second). \
         The line starts with `+++`. The first word of the line that ends in a colon \
         must be that number, so no word of this text ends in a colon \
         (spec rule `env.debugging.input.error`).",
    ),
    (
        (101, 5),
        "This text is the debug notice, one line. It starts with `+++` and tells how \
         to end the debugging. None of its words ends in a colon \
         (spec rule `env.debugging.notice`).",
    ),
];

/// Where a brief's facts come from.
pub struct Sources {
    /// The repository root, which the brief's paths are relative to and
    /// which the output directory must lie outside.
    pub root: PathBuf,
    /// `errors.tsv`.
    pub errors: PathBuf,
    /// The probe programs, `<major>.<sub>.rex`.
    pub probes: PathBuf,
    /// The directories of Rust source searched for raise sites.
    pub code: Vec<PathBuf>,
    /// The spec's `src` directory.
    pub spec: PathBuf,
    /// The interrexx interpreter that runs the probes.
    pub rexx: PathBuf,
}

/// The brief of every code of `major`, as (sub, text).
pub fn build(sources: &Sources, major: u16) -> Result<Vec<(u16, String)>, String> {
    let table = Table::from_tsv(&read(&sources.errors)?);
    let rows: Vec<&Row> = table.rows.iter().filter(|r| r.major == major).collect();
    if rows.is_empty() {
        return Err(format!("errors.tsv has no code of major {major}"));
    }
    let code = source_files(&sources.code, "rs")?;
    let spec = source_files(std::slice::from_ref(&sources.spec), "md")?;
    let mut briefs = Vec::new();
    for row in rows {
        let mut out = String::new();
        let code_name = format!("{}.{}", row.major, row.sub);
        let _ = writeln!(out, "# Message {code_name}: {}\n", row.symbol);
        interface(&mut out, row);
        if let Some((_, constraint)) = CONSTRAINTS.iter().find(|(c, _)| *c == (row.major, row.sub))
        {
            let _ = writeln!(out, "## Constraints\n\n{constraint}\n");
        }
        raise_sites(&mut out, sources, &code, row);
        spec_rules(&mut out, sources, &spec, row);
        probe(&mut out, sources, row)?;
        text_section(&mut out, row);
        briefs.push((row.sub, out));
    }
    Ok(briefs)
}

/// Writes [`build`]'s briefs to `out` as `<major>.<sub>.md`. `out` must lie
/// outside the repository. With `catalogue` (patterns of the catalogue's
/// texts), a brief that holds one is not written and the call fails.
pub fn write(
    sources: &Sources,
    major: u16,
    out: &Path,
    catalogue: Option<&[Pattern]>,
) -> Result<usize, String> {
    std::fs::create_dir_all(out).map_err(|e| format!("cannot create {}: {e}", out.display()))?;
    let (root, out_dir) = (canonical(&sources.root)?, canonical(out)?);
    if out_dir.starts_with(&root) {
        return Err(format!(
            "{} lies inside the repository; briefs go outside it",
            out.display()
        ));
    }
    let briefs = build(sources, major)?;
    if let Some(patterns) = catalogue {
        let leaking: Vec<String> = briefs
            .iter()
            .filter(|(_, text)| !Haystack::new(text, false).matches(patterns).is_empty())
            .map(|(sub, _)| format!("{major}.{sub}"))
            .collect();
        if !leaking.is_empty() {
            return Err(format!(
                "briefs for {} hold catalogue text; scrub their sources first",
                leaking.join(", ")
            ));
        }
    }
    for (sub, text) in &briefs {
        let path = out.join(format!("{major}.{sub}.md"));
        std::fs::write(&path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    }
    Ok(briefs.len())
}

fn canonical(path: &Path) -> Result<PathBuf, String> {
    path.canonicalize()
        .map_err(|e| format!("cannot resolve {}: {e}", path.display()))
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))
}

fn interface(out: &mut String, row: &Row) {
    let _ = writeln!(out, "## Interface\n");
    let _ = writeln!(out, "* Number: {}.", row.number);
    let _ = writeln!(out, "* Inserts the error carries: {}.", row.inserts);
    let rendered: Vec<String> = row.rendered.iter().map(|k| format!("`&{k}`")).collect();
    if rendered.is_empty() {
        let _ = writeln!(out, "* Inserts the text may use: none.");
    } else {
        let _ = writeln!(out, "* Inserts the text may use: {}.", rendered.join(", "));
    }
    let _ = writeln!(
        out,
        "* Every insert reaches `ADDITIONAL` whether the text shows it or not. \
         A text that uses an insert outside the list above is refused.\n"
    );
}

fn text_section(out: &mut String, row: &Row) {
    let _ = writeln!(out, "## Text\n");
    let _ = writeln!(
        out,
        "Write the text for message {}.{} in your own words, from the facts above.",
        row.major, row.sub
    );
    if row.sub == 0 {
        let _ = writeln!(
            out,
            "This is the text of major code {}; the labels of `conditions/errors.md` \
             in the spec may be reused for it.",
            row.major
        );
    }
}

/// Every file with extension `ext` under `dirs`, sorted, not under a
/// `tests` directory and not named `tests.rs`.
fn source_files(dirs: &[PathBuf], ext: &str) -> Result<Vec<PathBuf>, String> {
    fn walk(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) -> Result<(), String> {
        let entries =
            std::fs::read_dir(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if path.is_dir() {
                if name != "tests" {
                    walk(&path, ext, out)?;
                }
            } else if name != "tests.rs" && path.extension().is_some_and(|e| e == ext) {
                out.push(path);
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    for dir in dirs {
        walk(dir, ext, &mut files)?;
    }
    files.sort();
    Ok(files)
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

/// Whether `needle` occurs in `line` bounded by non-identifier characters.
fn has_word(line: &str, needle: &str) -> bool {
    line.match_indices(needle).any(|(at, _)| {
        let before = line[..at].chars().next_back();
        let after = line[at + needle.len()..].chars().next();
        let word = |c: char| c.is_alphanumeric() || c == '_';
        !before.is_some_and(word) && !after.is_some_and(word)
    })
}

/// Whether `line` raises `major.sub`: through its `E<major>_<sub>`
/// constant, or by a call such as `syntax(<major>, <sub>, ...)` whose name
/// says it raises or reports an error.
fn raises(line: &str, major: u16, sub: u16) -> bool {
    if line.trim_start().starts_with("use ") {
        return false;
    }
    if has_word(line, &format!("E{major}_{sub}")) {
        return true;
    }
    let args = format!("{major}, {sub}");
    line.match_indices(&args).any(|(at, _)| {
        let after = line[at + args.len()..].chars().next();
        if after.is_some_and(|c| c.is_ascii_digit()) {
            return false;
        }
        let Some(open) = line[..at].trim_end().strip_suffix('(') else {
            return false;
        };
        let name: String = open
            .chars()
            .rev()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<String>()
            .to_lowercase();
        ["syntax", "raise", "error"]
            .iter()
            .any(|k| name.contains(k))
    })
}

fn raise_sites(out: &mut String, sources: &Sources, files: &[PathBuf], row: &Row) {
    let _ = writeln!(out, "## Raise sites\n");
    let mut shown = 0;
    let mut total = 0;
    for file in files {
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if !raises(line, row.major, row.sub) {
                continue;
            }
            total += 1;
            if shown == MOST_SITES {
                continue;
            }
            shown += 1;
            let from = i.saturating_sub(3);
            let to = (i + 4).min(lines.len());
            let _ = writeln!(
                out,
                "`{}:{}`\n\n```rust",
                relative(&sources.root, file),
                i + 1
            );
            for l in &lines[from..to] {
                let _ = writeln!(out, "{l}");
            }
            let _ = writeln!(out, "```\n");
        }
    }
    if total == 0 {
        let _ = writeln!(out, "No raise site found.\n");
    } else if total > shown {
        let _ = writeln!(out, "{} more sites not shown.\n", total - shown);
    }
    let _ = writeln!(
        out,
        "The argument expressions at a site say what each insert holds.\n"
    );
}

/// Whether `line` names `major.sub` as a code: not part of a longer number.
fn names_code(line: &str, major: u16, sub: u16) -> bool {
    let code = format!("{major}.{sub}");
    line.match_indices(&code).any(|(at, _)| {
        let before = line[..at].chars().next_back();
        let after = line[at + code.len()..].chars().next();
        let more_digits = after.is_some_and(|c| c.is_ascii_digit())
            || (after == Some('.')
                && line[at + code.len() + 1..].starts_with(|c: char| c.is_ascii_digit()));
        !before.is_some_and(|c| c.is_alphanumeric() || c == '.' || c == '_') && !more_digits
    })
}

fn spec_rules(out: &mut String, sources: &Sources, files: &[PathBuf], row: &Row) {
    let _ = writeln!(out, "## Spec rules\n");
    /// A rule that names the code: where it first does, its paragraph, and
    /// the example blocks that hold the code.
    struct Cited {
        place: String,
        id: String,
        paragraph: Vec<String>,
        examples: Vec<Vec<String>>,
    }
    let mut cited: Vec<(PathBuf, Cited)> = Vec::new();
    if row.sub != 0 {
        for file in files {
            let Ok(text) = std::fs::read_to_string(file) else {
                continue;
            };
            let lines: Vec<&str> = text.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                if !names_code(line, row.major, row.sub) {
                    continue;
                }
                let Some((id, paragraph, example)) = rule_around(&lines, i) else {
                    continue;
                };
                let at = match cited.iter().position(|(f, c)| f == file && c.id == id) {
                    Some(at) => at,
                    None => {
                        cited.push((
                            file.clone(),
                            Cited {
                                place: format!("{}:{}", relative(&sources.root, file), i + 1),
                                id,
                                paragraph,
                                examples: Vec::new(),
                            },
                        ));
                        cited.len() - 1
                    }
                };
                if let Some(example) = example
                    && !cited[at].1.examples.contains(&example)
                {
                    cited[at].1.examples.push(example);
                }
            }
        }
    }
    for (_, c) in cited.iter().take(MOST_RULES) {
        let _ = writeln!(out, "`{}`, rule `{}`:\n", c.place, c.id);
        for l in &c.paragraph {
            let _ = writeln!(out, "> {l}");
        }
        for example in &c.examples {
            let _ = writeln!(out, "\n```text");
            for l in example {
                let _ = writeln!(out, "{l}");
            }
            let _ = writeln!(out, "```");
        }
        let _ = writeln!(out);
    }
    if cited.is_empty() {
        let _ = writeln!(out, "No rule names this code.\n");
    } else if cited.len() > MOST_RULES {
        let _ = writeln!(out, "{} more rules not shown.\n", cited.len() - MOST_RULES);
    }
}

/// The rule that line `at` belongs to: its id, its first paragraph, and the
/// fenced block holding the line, when it is in one.
fn rule_around(lines: &[&str], at: usize) -> Option<(String, Vec<String>, Option<Vec<String>>)> {
    let mut start = at;
    let id = loop {
        let l = lines[start];
        if let Some(id) = l.strip_prefix("r[").and_then(|r| r.strip_suffix(']')) {
            break id.to_string();
        }
        if start == 0 || l.starts_with('#') {
            return None;
        }
        start -= 1;
    };
    let paragraph: Vec<String> = lines[start + 1..]
        .iter()
        .take_while(|l| !l.trim().is_empty())
        .map(|l| (*l).to_string())
        .collect();
    let fences_before = lines[start..at]
        .iter()
        .filter(|l| l.trim_start().starts_with("```"))
        .count();
    let example =
        (fences_before % 2 == 1 && !lines[at].trim_start().starts_with("```")).then(|| {
            let open = lines[start..at]
                .iter()
                .rposition(|l| l.trim_start().starts_with("```"))
                .map_or(start, |p| start + p);
            lines[open + 1..]
                .iter()
                .take_while(|l| !l.trim_start().starts_with("```"))
                .take(60)
                .map(|l| (*l).to_string())
                .collect()
        });
    Some((id, paragraph, example))
}

fn probe(out: &mut String, sources: &Sources, row: &Row) -> Result<(), String> {
    let _ = writeln!(out, "## Probe\n");
    let file = sources
        .probes
        .join(format!("{}.{}.rex", row.major, row.sub));
    if !file.is_file() {
        let _ = writeln!(out, "No probe program for this code.\n");
        return Ok(());
    }
    let program = read(&file)?;
    let _ = writeln!(out, "```rexx\n{}\n```\n", program.trim_end());
    let (status, output) = run_probe(&sources.rexx, &file)?;
    let mut shown = output;
    if shown.len() > MOST_OUTPUT {
        let mut cut = MOST_OUTPUT;
        while !shown.is_char_boundary(cut) {
            cut -= 1;
        }
        shown.truncate(cut);
        shown.push_str("\n[output cut]");
    }
    let _ = writeln!(
        out,
        "Output ({status}):\n\n```text\n{}\n```\n",
        shown.trim_end()
    );
    Ok(())
}

/// Runs `program` on `rexx` in an empty scratch directory with no input,
/// the error output discarded, and returns how it ended and what it wrote.
#[expect(
    clippy::disallowed_methods,
    reason = "a tool that runs a child process waits on real time"
)]
fn run_probe(rexx: &Path, program: &Path) -> Result<(String, String), String> {
    static RUNS: AtomicUsize = AtomicUsize::new(0);
    let scratch = std::env::temp_dir().join(format!(
        "interrexx-brief-{}-{}",
        std::process::id(),
        RUNS.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&scratch).map_err(|e| format!("cannot create scratch: {e}"))?;
    let captured = scratch.join("stdout");
    let sink = std::fs::File::create(&captured).map_err(|e| format!("cannot create sink: {e}"))?;
    let program = canonical(program)?;
    let mut child = Command::new(rexx)
        .arg(&program)
        .current_dir(&scratch)
        .stdin(Stdio::null())
        .stdout(sink)
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("cannot run {}: {e}", rexx.display()))?;
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|e| format!("cannot wait: {e}"))? {
            break format!("{status}");
        }
        if started.elapsed() > PROBE_TIME {
            let _ = child.kill();
            let _ = child.wait();
            break "killed after the time limit".to_string();
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let output =
        String::from_utf8_lossy(&std::fs::read(&captured).unwrap_or_default()).into_owned();
    let _ = std::fs::remove_dir_all(&scratch);
    Ok((status, output))
}

#[cfg(test)]
mod tests;
