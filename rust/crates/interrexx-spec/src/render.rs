//! Rendering of one specification page for mdBook.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Component, Path, PathBuf};

use crate::page::{FenceLine, closes, fence_line};
use crate::{FrontMatter, is_rule_id, parse_page};

/// Render `text`, the page at `here`, as Markdown for mdBook.
///
/// `pages` maps each rule ID to its page; `here` and the map's paths are
/// relative to the book's `src`. A page that does not parse is returned
/// unchanged: the checker, not the renderer, enforces correctness.
pub fn render(text: &str, pages: &BTreeMap<String, PathBuf>, here: &Path) -> String {
    let Ok(page) = parse_page(here, text) else {
        return text.to_string();
    };
    let body = strip_front_matter(text);

    let mut out = String::new();
    let mut header = Some(header(&page.front));
    let mut fence: Option<usize> = None;
    let mut seen_content = false;
    for raw in body.split_inclusive('\n') {
        let line = raw.trim_end_matches(['\n', '\r']);
        let trimmed = line.trim_end();
        if let Some(open) = fence {
            if closes(trimmed, open) {
                fence = None;
            }
            out.push_str(raw);
            continue;
        }
        if let FenceLine::Fence { ticks, .. } = fence_line(trimmed) {
            fence = Some(ticks);
            seen_content = true;
            out.push_str(raw);
            continue;
        }
        if let Some(id) = trimmed
            .strip_prefix("r[")
            .and_then(|t| t.strip_suffix(']'))
            .filter(|id| is_rule_id(id))
        {
            // The blank line ends the HTML block, so the rule's paragraph
            // under it is read as Markdown.
            let _ = write!(
                out,
                "<div class=\"rule\" id=\"r-{id}\"><a href=\"#r-{id}\">[{id}]</a></div>\n\n"
            );
            seen_content = true;
            continue;
        }
        let start = out.len();
        out.push_str(&rewrite_links(raw, pages, here));
        if !seen_content && !trimmed.is_empty() {
            seen_content = true;
            if trimmed.starts_with("# ") {
                // The header goes under the page title.
                if let Some(h) = header.take() {
                    out.push('\n');
                    out.push_str(&h);
                }
            } else if let Some(h) = header.take() {
                // No title first: put the header before the content.
                out.insert_str(start, &h);
            }
        }
    }
    if let Some(h) = header {
        out.insert_str(0, &h);
    }
    out
}

fn strip_front_matter(text: &str) -> &str {
    let mut lines = text.split_inclusive('\n');
    if lines.next().is_none_or(|l| l.trim_end() != "---") {
        return text;
    }
    let mut consumed = text.len() - lines.clone().map(str::len).sum::<usize>();
    for l in lines {
        consumed += l.len();
        if l.trim_end() == "---" {
            return text.get(consumed..).unwrap_or("");
        }
    }
    text
}

fn header(front: &FrontMatter) -> String {
    let mut h = String::new();
    if let Some(syntax) = &front.syntax {
        let _ = write!(h, "\n```\n{syntax}\n```\n");
    }
    let mut refs = Vec::new();
    if let Some(a) = &front.ansi {
        refs.push(format!("ANSI: {}", escape(a)));
    }
    if let Some(r) = &front.rexxref {
        refs.push(format!("rexxref: {}", escape(r)));
    }
    if !refs.is_empty() {
        let _ = write!(h, "\n<p class=\"references\">{}</p>\n", refs.join("; "));
    }
    if !front.deviations.is_empty() {
        let list: Vec<String> = front.deviations.iter().map(|d| format!("D{d}")).collect();
        let _ = write!(
            h,
            "\n<p class=\"deviations\">Deviations: {}</p>\n",
            list.join(", ")
        );
    }
    if !h.is_empty() {
        h.push('\n');
    }
    h
}

/// `text` with `&`, `<`, `>` and `"` written as HTML entities.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
    out
}

/// Rewrite every `(r:ID)` link target on `line` whose ID is known.
fn rewrite_links(line: &str, pages: &BTreeMap<String, PathBuf>, here: &Path) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(at) = rest.find("](r:") {
        let (before, after) = rest.split_at(at + 2);
        out.push_str(before);
        let id_and_more = after.get(2..).unwrap_or("");
        let Some(end) = id_and_more.find(')') else {
            rest = after;
            break;
        };
        let id = id_and_more.get(..end).unwrap_or("").trim();
        match pages.get(id) {
            Some(target) => {
                let _ = write!(out, "{}#r-{id}", relative(here, target));
            }
            None => {
                let _ = write!(out, "r:{}", id_and_more.get(..end).unwrap_or(""));
            }
        }
        rest = id_and_more.get(end..).unwrap_or("");
    }
    out.push_str(rest);
    out
}

/// The path of `target` as seen from the directory holding `here`; empty when
/// both are the same page.
fn relative(here: &Path, target: &Path) -> String {
    if here == target {
        return String::new();
    }
    let from: Vec<Component> = here
        .parent()
        .map(|p| p.components().collect())
        .unwrap_or_default();
    let to: Vec<Component> = target.components().collect();
    let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    let mut parts: Vec<String> = vec!["..".to_string(); from.len().saturating_sub(common)];
    parts.extend(
        to.iter()
            .skip(common)
            .map(|c| c.as_os_str().to_string_lossy().into_owned()),
    );
    parts.join("/")
}
