//! The strict YAML subset the specification pages open with.
//!
//! One `key: value` per line; values are plain or double-quoted scalars or an
//! inline list; `#` starts a comment. Anything else is a problem, so the
//! block stays valid YAML while no YAML crate is needed.

use std::path::{Path, PathBuf};

use crate::Problem;

/// The kind of language element a page specifies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Builtin,
    Instruction,
    Directive,
    Class,
    Method,
    Concept,
    Chapter,
}

impl Kind {
    /// The kind written `s` in front matter.
    pub fn parse(s: &str) -> Option<Kind> {
        Some(match s {
            "builtin" => Kind::Builtin,
            "instruction" => Kind::Instruction,
            "directive" => Kind::Directive,
            "class" => Kind::Class,
            "method" => Kind::Method,
            "concept" => Kind::Concept,
            "chapter" => Kind::Chapter,
            _ => return None,
        })
    }
}

/// A page's front matter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontMatter {
    pub element: String,
    pub kind: Kind,
    pub syntax: Option<String>,
    pub ansi: Option<String>,
    pub rexxref: Option<String>,
    pub deviations: Vec<u32>,
}

enum Value {
    Scalar(String),
    List(Vec<String>),
}

/// Parse the lines between the `---` fences. `first_line` is the 1-based
/// line number of `lines[0]`.
pub(crate) fn parse(
    path: &Path,
    lines: &[&str],
    first_line: usize,
    out: &mut Vec<Problem>,
) -> Option<FrontMatter> {
    let before = out.len();
    let mut p = Parser {
        path: path.to_path_buf(),
        out,
    };
    let fm = p.run(lines, first_line);
    if p.out.len() == before { fm } else { None }
}

struct Parser<'a> {
    path: PathBuf,
    out: &'a mut Vec<Problem>,
}

impl Parser<'_> {
    fn problem(&mut self, line: usize, message: String) {
        self.out.push(Problem {
            path: self.path.clone(),
            line,
            message,
        });
    }

    fn run(&mut self, lines: &[&str], first_line: usize) -> Option<FrontMatter> {
        let mut element = None;
        let mut kind = None;
        let mut syntax = None;
        let mut ansi = None;
        let mut rexxref = None;
        let mut deviations = None;
        let mut seen: Vec<&str> = Vec::new();
        let mut in_nested = false;

        for (i, raw) in lines.iter().enumerate() {
            let line = first_line + i;
            let text = raw.trim_end();
            if text.trim().is_empty() || text.trim_start().starts_with('#') {
                continue;
            }
            if text.starts_with([' ', '\t']) {
                // Indented: a nested structure, or the body of one already reported.
                if !in_nested {
                    self.problem(
                        line,
                        "indented line: nested mappings and lists are not allowed".into(),
                    );
                    in_nested = true;
                }
                continue;
            }
            in_nested = false;
            let Some((key, rest)) = text.split_once(':') else {
                self.problem(line, format!("expected `key: value`, found `{text}`"));
                continue;
            };
            if !rest.is_empty() && !rest.starts_with(' ') {
                self.problem(line, format!("expected `key: value`, found `{text}`"));
                continue;
            }
            let known = ["element", "kind", "syntax", "ansi", "rexxref", "deviations"];
            let Some(&key) = known.iter().find(|k| **k == key) else {
                self.problem(line, format!("unknown front matter key `{key}`"));
                continue;
            };
            if seen.contains(&key) {
                self.problem(line, format!("duplicate front matter key `{key}`"));
                continue;
            }
            seen.push(key);
            let rest = rest.trim();
            if rest.is_empty() || rest.starts_with('#') {
                self.problem(
                    line,
                    format!("key `{key}` has no value (nested mappings are not allowed)"),
                );
                in_nested = true;
                continue;
            }
            let value = match self.value(line, key, rest) {
                Some(v) => v,
                None => continue,
            };
            match (key, value) {
                ("deviations", Value::List(items)) => {
                    let mut nums = Vec::new();
                    for item in items {
                        match item.parse::<u32>() {
                            Ok(n) => nums.push(n),
                            Err(_) => self
                                .problem(line, format!("deviation `{item}` is not a whole number")),
                        }
                    }
                    deviations = Some(nums);
                }
                ("deviations", Value::Scalar(_)) => {
                    self.problem(
                        line,
                        "`deviations` must be a list such as `[90, 91]`".into(),
                    );
                }
                (_, Value::List(_)) => {
                    self.problem(line, format!("`{key}` must be a scalar, not a list"));
                }
                ("kind", Value::Scalar(s)) => match Kind::parse(&s) {
                    Some(k) => kind = Some(k),
                    None => self.problem(line, format!("unknown kind `{s}`")),
                },
                ("element", Value::Scalar(s)) => element = Some(s),
                ("syntax", Value::Scalar(s)) => syntax = Some(s),
                ("ansi", Value::Scalar(s)) => ansi = Some(s),
                ("rexxref", Value::Scalar(s)) => rexxref = Some(s),
                _ => {}
            }
        }

        // Required keys: a missing key is reported unless its value was
        // present but rejected above (then `seen` holds it).
        if element.is_none() && !seen.contains(&"element") {
            self.problem(
                first_line.saturating_sub(1).max(1),
                "front matter lacks required key `element`".into(),
            );
        }
        if kind.is_none() && !seen.contains(&"kind") {
            self.problem(
                first_line.saturating_sub(1).max(1),
                "front matter lacks required key `kind`".into(),
            );
        }
        Some(FrontMatter {
            element: element?,
            kind: kind?,
            syntax,
            ansi,
            rexxref,
            deviations: deviations.unwrap_or_default(),
        })
    }

    fn value(&mut self, line: usize, key: &str, rest: &str) -> Option<Value> {
        if let Some(body) = rest.strip_prefix('"') {
            let mut s = String::new();
            let mut chars = body.char_indices();
            while let Some((at, c)) = chars.next() {
                match c {
                    '\\' => match chars.next() {
                        Some((_, '"')) => s.push('"'),
                        Some((_, '\\')) => s.push('\\'),
                        _ => {
                            self.problem(line, format!("unsupported escape in `{key}`"));
                            return None;
                        }
                    },
                    '"' => {
                        let tail = body.get(at + 1..).unwrap_or("").trim();
                        if !tail.is_empty() && !tail.starts_with('#') {
                            self.problem(
                                line,
                                format!("unexpected text after the quoted value of `{key}`"),
                            );
                            return None;
                        }
                        return Some(Value::Scalar(s));
                    }
                    c => s.push(c),
                }
            }
            self.problem(line, format!("unterminated quoted value for `{key}`"));
            return None;
        }
        if let Some(body) = rest.strip_prefix('[') {
            let Some(end) = body.find(']') else {
                self.problem(line, format!("unterminated list for `{key}`"));
                return None;
            };
            let tail = body.get(end + 1..).unwrap_or("").trim();
            if !tail.is_empty() && !tail.starts_with('#') {
                self.problem(line, format!("unexpected text after the list of `{key}`"));
                return None;
            }
            let inner = body.get(..end).unwrap_or("").trim();
            if inner.is_empty() {
                return Some(Value::List(Vec::new()));
            }
            let items: Vec<String> = inner.split(',').map(|s| s.trim().to_string()).collect();
            if items.iter().any(String::is_empty) {
                self.problem(line, format!("empty item in the list of `{key}`"));
                return None;
            }
            return Some(Value::List(items));
        }
        if rest.starts_with(['{', '|', '>', '&', '*', '!']) {
            self.problem(
                line,
                format!("unsupported YAML construct in the value of `{key}`"),
            );
            return None;
        }
        let plain = match rest.find(" #") {
            Some(at) => rest.get(..at).unwrap_or(rest).trim_end(),
            None => rest,
        };
        Some(Value::Scalar(plain.to_string()))
    }
}
