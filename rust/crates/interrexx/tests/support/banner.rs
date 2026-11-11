//! The `-v` banner as Deviation 96 states it. The oracle's banner lines are mapped to
//! interrexx's, and the build date and interrexx's version are masked, so a recorded block holds
//! interrexx's text and both sides must agree on it.

/// `line` as a recorded block shows it, or `None` for a line the oracle's banner has and
/// interrexx's does not. `oracle` says which side printed it.
pub fn recorded(line: &str, oracle: bool) -> Option<String> {
    if let Some(_date) = line.strip_prefix("Build date: ") {
        return Some("Build date: <date>".to_string());
    }
    if oracle {
        if let Some(rest) = line.strip_prefix("Open Object Rexx Version ")
            && let Some(version) = rest.split(' ').next()
        {
            return Some(format!(
                "interrexx <version> (Open Object Rexx {version} compatible)"
            ));
        }
        if line.starts_with("Copyright (c) 1995, 2004 IBM Corporation.") {
            return Some("Licensed under MIT OR Apache-2.0.".to_string());
        }
        let dropped = [
            "Copyright (c) 2005-",
            "This program and the accompanying materials",
            "of the Common ",
            "https://www.oorexx.org/license.html",
        ];
        if dropped.iter().any(|prefix| line.starts_with(prefix)) {
            return None;
        }
    } else if let Some(rest) = line.strip_prefix("interrexx ")
        && let Some((_, tail)) = rest.split_once(" (Open Object Rexx ")
    {
        return Some(format!("interrexx <version> (Open Object Rexx {tail}"));
    }
    Some(line.to_string())
}
