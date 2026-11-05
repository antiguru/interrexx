//! Loading a whole `spec/` tree and the checks that need more than one page.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::{Page, Problem, parse_page};

/// The rule-ID prefix for a page at `relative` (relative to `spec/src`), which
/// names the page: a chapter that is one file has its chapter prefix
/// (`conventions.md` is `conv.`), and a page in a chapter directory adds its
/// file stem (`builtins/substr.md` is `bif.substr.`, `builtins/index.md` is
/// `bif.index.`). A reader finds a rule's page from its ID alone.
pub fn prefix_for(relative: &Path) -> Option<String> {
    let comps: Vec<&str> = relative.iter().filter_map(|c| c.to_str()).collect();
    let (chapter, file) = match comps.as_slice() {
        ["conventions.md"] => return Some("conv.".into()),
        [dir, file] => (*dir, *file),
        _ => return None,
    };
    let chapter = match chapter {
        "builtins" => "bif",
        "instructions" => "instr",
        "expressions" => "expr",
        "parsing" => "parse",
        "conditions" => "cond",
        "classes" => "class",
        _ => return None,
    };
    let stem = file.strip_suffix(".md")?;
    Some(format!("{chapter}.{stem}."))
}

fn markdown_files(dir: &Path, out: &mut Vec<PathBuf>, problems: &mut Vec<Problem>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            problems.push(Problem {
                path: dir.to_path_buf(),
                line: 0,
                message: format!("cannot read directory: {e}"),
            });
            return;
        }
    };
    let mut paths: Vec<PathBuf> = entries.filter_map(Result::ok).map(|e| e.path()).collect();
    paths.sort();
    for p in paths {
        if p.is_dir() {
            markdown_files(&p, out, problems);
        } else if p.extension().is_some_and(|e| e == "md") {
            out.push(p);
        }
    }
}

/// `/`-joined form of a relative path, for comparing against `SUMMARY.md`.
fn slashed(p: &Path) -> String {
    p.iter()
        .filter_map(|c| c.to_str())
        .collect::<Vec<_>>()
        .join("/")
}

/// Load every page under `root/src` and run the cross-page checks: front
/// matter, rule IDs (duplicates, withdrawn, prefix) and `SUMMARY.md`.
pub fn load(root: &Path) -> (Vec<Page>, Vec<Problem>) {
    let src = root.join("src");
    let mut problems = Vec::new();
    let mut files = Vec::new();
    markdown_files(&src, &mut files, &mut problems);

    let withdrawn = read_withdrawn(&root.join("withdrawn.txt"), &mut problems);

    let summary_path = src.join("SUMMARY.md");
    let mut pages = Vec::new();
    let mut relatives = BTreeSet::new();
    for file in files {
        if file == summary_path {
            continue;
        }
        let rel = file.strip_prefix(&src).unwrap_or(&file).to_path_buf();
        relatives.insert(slashed(&rel));
        let text = match fs::read_to_string(&file) {
            Ok(t) => t,
            Err(e) => {
                problems.push(Problem {
                    path: file,
                    line: 0,
                    message: format!("cannot read page: {e}"),
                });
                continue;
            }
        };
        match parse_page(&file, &text) {
            Ok(page) => pages.push((rel, page)),
            Err(p) => problems.extend(p),
        }
    }

    let mut seen: BTreeMap<String, (PathBuf, usize)> = BTreeMap::new();
    for (rel, page) in &pages {
        let prefix = prefix_for(rel);
        for rule in &page.rules {
            match &prefix {
                Some(p) if !rule.id.starts_with(p.as_str()) => problems.push(Problem {
                    path: page.path.clone(),
                    line: rule.line,
                    message: format!("rule `{}` must begin with `{p}` on this page", rule.id),
                }),
                None => problems.push(Problem {
                    path: page.path.clone(),
                    line: rule.line,
                    message: format!(
                        "rule `{}` is on a page outside any chapter directory",
                        rule.id
                    ),
                }),
                _ => {}
            }
            if withdrawn.contains(&rule.id) {
                problems.push(Problem {
                    path: page.path.clone(),
                    line: rule.line,
                    message: format!("rule ID `{}` is withdrawn and may not be reused", rule.id),
                });
            }
            match seen.get(&rule.id) {
                Some((first, first_line)) => problems.push(Problem {
                    path: page.path.clone(),
                    line: rule.line,
                    message: format!(
                        "rule ID `{}` is already defined at {}:{first_line}",
                        rule.id,
                        first.display()
                    ),
                }),
                None => {
                    seen.insert(rule.id.clone(), (page.path.clone(), rule.line));
                }
            }
        }
    }

    check_summary(&summary_path, &relatives, &mut problems);

    (pages.into_iter().map(|(_, p)| p).collect(), problems)
}

fn read_withdrawn(path: &Path, problems: &mut Vec<Problem>) -> BTreeSet<String> {
    match fs::read_to_string(path) {
        Ok(text) => text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(str::to_string)
            .collect(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => BTreeSet::new(),
        Err(e) => {
            problems.push(Problem {
                path: path.to_path_buf(),
                line: 0,
                message: format!("cannot read withdrawn list: {e}"),
            });
            BTreeSet::new()
        }
    }
}

fn check_summary(path: &Path, files: &BTreeSet<String>, problems: &mut Vec<Problem>) {
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            problems.push(Problem {
                path: path.to_path_buf(),
                line: 0,
                message: format!("cannot read SUMMARY.md: {e}"),
            });
            return;
        }
    };
    let mut listed = BTreeSet::new();
    for (i, raw) in text.lines().enumerate() {
        let line = i + 1;
        let t = raw.trim_start();
        let Some(item) = t.strip_prefix("- ").or_else(|| t.strip_prefix("* ")) else {
            continue;
        };
        let Some(open) = item.find("](") else {
            continue;
        };
        let after = item.get(open + 2..).unwrap_or("");
        let Some(close) = after.find(')') else {
            continue;
        };
        let target = after.get(..close).unwrap_or("").trim();
        if target.is_empty() {
            problems.push(Problem {
                path: path.to_path_buf(),
                line,
                message: "draft entry without a path is not allowed".into(),
            });
            continue;
        }
        if target.contains("://") {
            continue;
        }
        let target = target.split('#').next().unwrap_or(target);
        let norm: Vec<&str> = target
            .split('/')
            .filter(|c| !c.is_empty() && *c != ".")
            .collect();
        let norm = norm.join("/");
        if !files.contains(&norm) {
            problems.push(Problem {
                path: path.to_path_buf(),
                line,
                message: format!("SUMMARY.md names `{norm}`, which is not a page under spec/src"),
            });
        }
        listed.insert(norm);
    }
    for f in files {
        if !listed.contains(f) {
            problems.push(Problem {
                path: path.to_path_buf(),
                line: 0,
                message: format!("page `{f}` is not listed in SUMMARY.md"),
            });
        }
    }
}
