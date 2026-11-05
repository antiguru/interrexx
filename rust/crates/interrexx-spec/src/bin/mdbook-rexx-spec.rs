//! mdBook preprocessor: renders specification pages (rule anchors, rule links,
//! front matter header). Speaks the preprocessor protocol over stdin/stdout.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use interrexx_spec::{parse_page, render};
use serde_json::Value;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() == Some("supports") {
        return ExitCode::SUCCESS;
    }
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("mdbook-rexx-spec: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|e| format!("reading stdin: {e}"))?;
    let parsed: Value = serde_json::from_str(&input).map_err(|e| format!("bad JSON: {e}"))?;
    let Value::Array(mut pair) = parsed else {
        return Err("expected a `[context, book]` array".into());
    };
    if pair.len() != 2 {
        return Err("expected a `[context, book]` array".into());
    }
    let mut book = pair.pop().unwrap_or(Value::Null);

    let mut pages = BTreeMap::new();
    for key in ["sections", "items"] {
        if let Some(items) = book.get(key) {
            collect(items, &mut pages);
        }
    }
    for key in ["sections", "items"] {
        if let Some(items) = book.get_mut(key) {
            rewrite(items, &pages);
        }
    }

    let out = serde_json::to_string(&book).map_err(|e| e.to_string())?;
    std::io::stdout()
        .write_all(out.as_bytes())
        .map_err(|e| format!("writing stdout: {e}"))
}

/// The `(path, content)` of a chapter item that is not a draft.
fn chapter(item: &Value) -> Option<(PathBuf, &Value)> {
    let ch = item.get("Chapter")?;
    let path = ch.get("path")?.as_str()?;
    Some((PathBuf::from(path), ch))
}

fn collect(items: &Value, pages: &mut BTreeMap<String, PathBuf>) {
    let Some(items) = items.as_array() else {
        return;
    };
    for item in items {
        let Some((path, ch)) = chapter(item) else {
            continue;
        };
        if let Some(content) = ch.get("content").and_then(Value::as_str) {
            match parse_page(&path, content) {
                Ok(page) => {
                    for rule in page.rules {
                        pages.entry(rule.id).or_insert_with(|| path.clone());
                    }
                }
                Err(problems) => eprintln!(
                    "mdbook-rexx-spec: warning: {}: {} problem(s), page left unrendered (first: line {}: {})",
                    path.display(),
                    problems.len(),
                    problems.first().map_or(0, |p| p.line),
                    problems.first().map_or("", |p| p.message.as_str()),
                ),
            }
        }
        if let Some(sub) = ch.get("sub_items") {
            collect(sub, pages);
        }
    }
}

fn rewrite(items: &mut Value, pages: &BTreeMap<String, PathBuf>) {
    let Some(items) = items.as_array_mut() else {
        return;
    };
    for item in items {
        let Some(ch) = item.get_mut("Chapter") else {
            continue;
        };
        let path = ch.get("path").and_then(Value::as_str).map(PathBuf::from);
        if let Some(path) = path
            && let Some(content) = ch.get("content").and_then(Value::as_str)
        {
            let rendered = render(content, pages, Path::new(&path));
            ch["content"] = Value::String(rendered);
        }
        if let Some(sub) = ch.get_mut("sub_items") {
            rewrite(sub, pages);
        }
    }
}
