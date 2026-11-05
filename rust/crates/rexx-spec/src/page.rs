//! One specification page: front matter, rules, examples and rule links.

use std::path::{Path, PathBuf};

use crate::front_matter::{self, FrontMatter};
use crate::{Problem, is_rule_id};

/// A rule: the `r[ID]` line and the paragraph under it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub id: String,
    pub line: usize,
    pub text: String,
}

/// A `rexx` block followed by an `output` block, belonging to `rule`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Example {
    pub rule: String,
    /// Line of the opening fence of the `rexx` block.
    pub line: usize,
    pub program: String,
    /// The output block's lines, each followed by `\n`.
    pub output: String,
    pub rc: i32,
}

/// A `[text](r:ID)` link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleLink {
    pub id: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub path: PathBuf,
    pub front: FrontMatter,
    pub rules: Vec<Rule>,
    pub examples: Vec<Example>,
    pub links: Vec<RuleLink>,
}

struct Fence {
    ticks: usize,
    info: String,
    line: usize,
    body: Vec<String>,
}

/// Parse one page. Every defect found is returned, not only the first.
pub fn parse_page(path: &Path, text: &str) -> Result<Page, Vec<Problem>> {
    let mut problems = Vec::new();
    let lines: Vec<&str> = text.lines().collect();

    let mut front = None;
    let mut body_start = 0;
    if lines.first().is_some_and(|l| l.trim_end() == "---") {
        match lines.iter().skip(1).position(|l| l.trim_end() == "---") {
            Some(n) => {
                let inner = lines.get(1..=n).unwrap_or(&[]);
                front = front_matter::parse(path, inner, 2, &mut problems);
                body_start = n + 2;
            }
            None => problems.push(Problem {
                path: path.to_path_buf(),
                line: 1,
                message: "front matter is not closed by a `---` line".into(),
            }),
        }
    } else {
        problems.push(Problem {
            path: path.to_path_buf(),
            line: 1,
            message: "page does not begin with front matter (`---`)".into(),
        });
    }

    let mut rules: Vec<Rule> = Vec::new();
    let mut examples = Vec::new();
    let mut links = Vec::new();
    let mut fence: Option<Fence> = None;
    let mut pending_rexx: Option<(usize, String)> = None;
    let mut collecting = false; // inside a rule's paragraph
    let mut err = |line: usize, message: String| {
        problems.push(Problem {
            path: path.to_path_buf(),
            line,
            message,
        });
    };

    for (i, raw) in lines.iter().enumerate().skip(body_start) {
        let line = i + 1;
        let text = raw.trim_end();

        if let Some(f) = fence.as_mut() {
            let ticks = text.chars().take_while(|c| *c == '`').count();
            if text.trim_start().chars().all(|c| c == '`')
                && ticks >= f.ticks
                && !text.starts_with(' ')
            {
                let done = fence.take();
                if let Some(done) = done {
                    finish_block(done, &mut pending_rexx, &mut examples, &rules, &mut err);
                }
            } else {
                f.body
                    .push(raw.strip_suffix('\r').unwrap_or(raw).to_string());
            }
            continue;
        }

        if let Some(opening) = fence_open(text) {
            collecting = false;
            fence = Some(Fence {
                ticks: opening.0,
                info: opening.1,
                line,
                body: Vec::new(),
            });
            continue;
        }

        if text.is_empty() {
            collecting = false;
            continue;
        }
        pending_rexx = None;

        if let Some(inner) = text.strip_prefix("r[").and_then(|t| t.strip_suffix(']')) {
            collecting = true;
            if is_rule_id(inner) {
                rules.push(Rule {
                    id: inner.to_string(),
                    line,
                    text: String::new(),
                });
            } else {
                err(line, format!("malformed rule ID `{inner}`"));
                // Keep a placeholder out of `rules`; the paragraph is dropped.
                collecting = false;
            }
            continue;
        }

        if text.trim_start().starts_with("r[") {
            collecting = false;
            err(
                line,
                "line looks like a rule marker but is not a bare `r[ID]` line".into(),
            );
            continue;
        }

        if collecting && let Some(rule) = rules.last_mut() {
            if !rule.text.is_empty() {
                rule.text.push('\n');
            }
            rule.text.push_str(text);
        }
        scan_links(text, line, &mut links);
    }

    if let Some(f) = fence {
        err(f.line, "code block is never closed".into());
    }

    match front {
        Some(front) if problems.is_empty() => Ok(Page {
            path: path.to_path_buf(),
            front,
            rules,
            examples,
            links,
        }),
        _ => Err(problems),
    }
}

/// `(backtick count, info string)` when `text` opens a fenced block.
fn fence_open(text: &str) -> Option<(usize, String)> {
    let ticks = text.chars().take_while(|c| *c == '`').count();
    if ticks < 3 {
        return None;
    }
    let info = text.get(ticks..).unwrap_or("").trim();
    if info.contains('`') {
        return None;
    }
    Some((ticks, info.to_string()))
}

fn finish_block(
    f: Fence,
    pending_rexx: &mut Option<(usize, String)>,
    examples: &mut Vec<Example>,
    rules: &[Rule],
    err: &mut impl FnMut(usize, String),
) {
    let mut content = String::new();
    for l in &f.body {
        content.push_str(l);
        content.push('\n');
    }
    let mut words = f.info.split_whitespace();
    match words.next() {
        Some("rexx") => *pending_rexx = Some((f.line, content)),
        Some("output") => {
            let mut rc = 0;
            for w in words {
                match w.strip_prefix("rc=").and_then(|n| n.parse::<i32>().ok()) {
                    Some(n) => rc = n,
                    None => err(f.line, format!("unrecognised output block option `{w}`")),
                }
            }
            let Some((line, program)) = pending_rexx.take() else {
                err(
                    f.line,
                    "`output` block does not follow a `rexx` block".into(),
                );
                return;
            };
            match rules.last() {
                Some(rule) => examples.push(Example {
                    rule: rule.id.clone(),
                    line,
                    program,
                    output: content,
                    rc,
                }),
                None => err(line, "example appears before any rule".into()),
            }
        }
        _ => *pending_rexx = None,
    }
}

/// Record every `[text](r:ID)` on `text`.
fn scan_links(text: &str, line: usize, links: &mut Vec<RuleLink>) {
    let mut rest = text;
    while let Some(at) = rest.find("](r:") {
        let after = rest.get(at + 4..).unwrap_or("");
        let Some(end) = after.find(')') else { break };
        links.push(RuleLink {
            id: after.get(..end).unwrap_or("").trim().to_string(),
            line,
        });
        rest = after.get(end..).unwrap_or("");
    }
}
