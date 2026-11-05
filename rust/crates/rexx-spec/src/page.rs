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
            if closes(text, f.ticks) {
                let done = fence.take();
                if let Some(done) = done {
                    finish_block(done, &mut pending_rexx, &mut examples, &rules, &mut err);
                }
            } else {
                if near_miss_close(text, f.ticks) {
                    err(
                        line,
                        "an indented fence would close this code block in Markdown; \
                         a closing fence starts in column 0"
                            .into(),
                    );
                }
                f.body
                    .push(raw.strip_suffix('\r').unwrap_or(raw).to_string());
            }
            continue;
        }

        match fence_line(text) {
            FenceLine::Fence { ticks, info } => {
                collecting = false;
                fence = Some(Fence {
                    ticks,
                    info: info.to_string(),
                    line,
                    body: Vec::new(),
                });
                continue;
            }
            FenceLine::NearMiss => {
                collecting = false;
                pending_rexx = None;
                err(
                    line,
                    "line looks like a code fence but is not a column-0 backtick fence".into(),
                );
                continue;
            }
            FenceLine::Text => {}
        }

        if text.is_empty() {
            collecting = false;
            continue;
        }
        pending_rexx = None;

        if let Some(inner) = text.strip_prefix("r[").and_then(|t| t.strip_suffix(']')) {
            collecting = true;
            if is_rule_id(inner) {
                if has_deviation_segment(inner) && deviation_number(inner).is_none() {
                    err(
                        line,
                        format!(
                            "rule ID `{inner}` has a `deviation` segment but does not end \
                             `.deviation.N`"
                        ),
                    );
                }
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
        scan_links(text, line, &mut links, &mut err);
    }

    if let Some(f) = fence {
        err(f.line, "code block is never closed".into());
    }
    for rule in rules.iter().filter(|r| r.text.is_empty()) {
        err(
            rule.line,
            format!(
                "rule `{}` has no text: its paragraph must follow the `r[ID]` line",
                rule.id
            ),
        );
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

/// What a line outside a code block is, as far as fences go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FenceLine<'a> {
    /// A column-0 run of at least three backticks and an info string holding
    /// no backtick: the only fence a page may use.
    Fence {
        ticks: usize,
        info: &'a str,
    },
    /// A line Markdown may read as a fence that the page format does not
    /// accept: indented, `~~~`, or a backtick run whose info string holds a
    /// backtick.
    NearMiss,
    Text,
}

/// Classify `text` (a line with its line ending removed) outside a code
/// block. The parser and the renderer both read fences through this, so a
/// line one of them takes for a fence the other does too.
pub(crate) fn fence_line(text: &str) -> FenceLine<'_> {
    let ticks = text.chars().take_while(|c| *c == '`').count();
    if ticks >= 3 {
        let info = text.get(ticks..).unwrap_or("").trim();
        if !info.contains('`') {
            return FenceLine::Fence { ticks, info };
        }
    }
    let start = text.trim_start();
    if start.starts_with("```") || start.starts_with("~~~") {
        FenceLine::NearMiss
    } else {
        FenceLine::Text
    }
}

/// Whether `text` closes a code block opened by `open` backticks: a column-0
/// run of at least `open` backticks and nothing after it but blanks.
pub(crate) fn closes(text: &str, open: usize) -> bool {
    let ticks = text.chars().take_while(|c| *c == '`').count();
    ticks >= open && text.get(ticks..).unwrap_or("").trim().is_empty()
}

/// Whether `text`, inside a block opened by `open` backticks, is a closing
/// fence that is indented: Markdown closes the block there, the parser would
/// not.
fn near_miss_close(text: &str, open: usize) -> bool {
    let start = text.trim_start();
    start.len() != text.len() && closes(start, open)
}

/// Whether some segment of `id` is `deviation`.
fn has_deviation_segment(id: &str) -> bool {
    id.split('.').any(|s| s == "deviation")
}

/// The register number `N` when `id` ends `.deviation.N` and has no other
/// `deviation` segment; `N` is decimal digits with no leading zero.
pub fn deviation_number(id: &str) -> Option<u32> {
    let mut segments = id.rsplit('.');
    let n = segments.next()?;
    if segments.next()? != "deviation"
        || segments.any(|s| s == "deviation")
        || n.is_empty()
        || !n.bytes().all(|b| b.is_ascii_digit())
        || (n.len() > 1 && n.starts_with('0'))
    {
        return None;
    }
    n.parse().ok()
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

/// Record every `[text](r:ID)` on `text`, and report every link that targets
/// a rule anchor (`#r-ID`) directly: rule links use the `r:` form, which the
/// checker resolves.
fn scan_links(
    text: &str,
    line: usize,
    links: &mut Vec<RuleLink>,
    err: &mut impl FnMut(usize, String),
) {
    let mut rest = text;
    while let Some(at) = rest.find("](") {
        let after = rest.get(at + 2..).unwrap_or("");
        let Some(end) = after.find(')') else { break };
        let target = after.get(..end).unwrap_or("");
        if let Some(id) = target.strip_prefix("r:") {
            links.push(RuleLink {
                id: id.trim().to_string(),
                line,
            });
        } else if target.contains("#r-") {
            err(
                line,
                format!("link `({target})` targets a rule anchor; write it `(r:ID)`"),
            );
        }
        rest = after.get(end..).unwrap_or("");
    }
}
