//! The builtin function table's interface: the names, in table order.

use crate::header;

/// The builtin names in table order, which is the index the parser resolves
/// builtins through. Never sorted.
pub fn parse(src: &str) -> Vec<String> {
    let start = src
        .find("pbuiltin LanguageParser::builtinTable[] =")
        .expect("the builtin table is still declared in its source");
    let table = &src[start..];
    let end = table.find("\n};").expect("the builtin table is terminated");

    let names: Vec<String> = table[..end]
        .lines()
        .filter_map(|line| line.trim().strip_prefix("&builtin_function_"))
        .map(|rest| {
            rest.trim_end_matches(&[' ', ',', '\t'][..])
                .trim()
                .to_string()
        })
        .collect();

    // 81 entries as of 8c880bdd. The floor catches a broken parse without
    // tripping on the real count; it is deliberately well below it.
    if names.len() < 50 {
        panic!(
            "only {} builtin names parsed -- the table format changed",
            names.len()
        );
    }
    names
}

/// The names as checked-in text, one per line, headed with the source
/// revision.
pub fn to_text(names: &[String], sha: &str) -> String {
    let mut out = header(sha);
    out.push_str("# One builtin per line, in table order. Do not sort.\n");
    for name in names {
        out.push_str(name);
        out.push('\n');
    }
    out
}

/// Reads the names back from the checked-in text.
pub fn from_text(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests;
