//! The `-v` banner as Deviation 96 states it. The oracle's banner is mapped to interrexx's by
//! position, and the build date and interrexx's version are masked, so a recorded block holds
//! interrexx's text and both sides must agree on it.

/// The lines of the oracle's banner after the one naming the addressing mode: the notice block
/// that interrexx replaces by one line. The count is the oracle's `rexx -v` output, not a rule
/// of the spec.
const ORACLE_NOTICE_LINES: usize = 5;

/// `lines` as a recorded block shows them. The oracle's banner is recognised by position: the
/// line before `Build date: ` is the version line, whose last word is the version, and the
/// [`ORACLE_NOTICE_LINES`] lines after `Addressing mode: ` become one line. `oracle` says which
/// side printed the lines.
pub fn recorded<'a>(lines: impl IntoIterator<Item = &'a str>, oracle: bool) -> Vec<String> {
    let lines: Vec<&str> = lines.into_iter().collect();
    let mut out = Vec::with_capacity(lines.len());
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        index += 1;
        if line.starts_with("Build date: ") {
            out.push("Build date: <date>".to_string());
        } else if oracle
            && lines
                .get(index)
                .is_some_and(|next| next.starts_with("Build date: "))
            && let Some(version) = line.split(' ').next_back()
        {
            out.push(format!(
                "interrexx <version> (Open Object Rexx {version} compatible)"
            ));
        } else if oracle && line.starts_with("Addressing mode: ") {
            out.push(line.to_string());
            out.push("Licensed under MIT OR Apache-2.0.".to_string());
            index = (index + ORACLE_NOTICE_LINES).min(lines.len());
        } else if !oracle
            && let Some(rest) = line.strip_prefix("interrexx ")
            && let Some((_, tail)) = rest.split_once(" (Open Object Rexx ")
        {
            out.push(format!("interrexx <version> (Open Object Rexx {tail}"));
        } else {
            out.push(line.to_string());
        }
    }
    out
}
