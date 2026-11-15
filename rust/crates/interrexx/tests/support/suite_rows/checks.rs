use super::*;

/// One row's owner check.
pub fn check_row(row: &Row, context: &Context) -> Result<(), String> {
    if !context.groups.contains(&row.group) {
        return Err("no such group".to_string());
    }
    let text = read_lossy(&context.group_file(&row.group));
    let (dir, group) = row.group.rsplit_once('/').unwrap_or(("", &row.group));
    let defined = source_test_names_in(&context.root, dir, group).contains(&row.test);
    if !defined {
        return Err("the group file defines no such method".to_string());
    }
    match row.owner {
        Owner::Deviation(number) => {
            if context.deviations.contains(&number) {
                Ok(())
            } else {
                Err(format!(
                    "{DEVIATIONS} has no Deviation {number} that is not withdrawn"
                ))
            }
        }
        Owner::Environment => check_environment(row, context),
        Owner::Flaky => {
            let stem = row.group.rsplit('/').next().expect("a group name");
            let entry = row
                .reason
                .strip_prefix("common.txt: ")
                .ok_or("the reason is not `common.txt: ENTRY`")?
                .to_ascii_uppercase();
            if !context.flaky.contains(&entry) {
                Err(format!("{COMMON_FAILURES} has no entry {entry}"))
            } else if entry != format!("{stem}.TESTGROUP/{}", row.test).to_ascii_uppercase() {
                Err(format!("{entry} is not this row's test"))
            } else {
                Ok(())
            }
        }
        Owner::NotRun => check_not_run(row, &text, context),
        Owner::Unspecified => {
            check_assertions(row, &text, context)?;
            check_unspecified(row, context)
        }
    }
}

/// Each line an Unspecified row names is in the group file and starts an
/// assertion, in the row's test or in a method or routine that is no other
/// test of the group.
fn check_assertions(row: &Row, text: &str, context: &Context) -> Result<(), String> {
    let (dir, group) = row.group.rsplit_once('/').unwrap_or(("", &row.group));
    let tests = source_test_names_in(&context.root, dir, group);
    let source: Vec<&str> = text.lines().collect();
    for line in assertion_lines(&row.reason)? {
        let Some(clause) = source.get(line - 1) else {
            return Err(format!("the group file has no line {line}"));
        };
        if !guardable(
            clause.as_bytes(),
            source.get(line).map(|next| next.as_bytes()),
        ) {
            return Err(format!(
                "line {line} starts no assertion and is no IF of one"
            ));
        }
        match enclosing(text, line) {
            None => return Err(format!("line {line} is in no method or routine")),
            Some((keyword, name))
                if keyword == "method" && name != row.test && tests.contains(&name) =>
            {
                return Err(format!("line {line} is in the test {name}"));
            }
            Some(_) => {}
        }
    }
    Ok(())
}

/// An Unspecified row's reason cites a rule as `r[ID]`, and the
/// specification under `spec/src` states that rule.
fn check_unspecified(row: &Row, context: &Context) -> Result<(), String> {
    let id = row
        .reason
        .split_once("r[")
        .and_then(|(_, rest)| rest.split_once(']'))
        .map(|(id, _)| id)
        .filter(|id| !id.is_empty())
        .ok_or("the reason cites no rule as `r[ID]`")?;
    let marker = format!("r[{id}]");
    let mut pending = vec![context.worktree.join("spec/src")];
    while let Some(dir) = pending.pop() {
        let entries =
            fs::read_dir(&dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|ext| ext == "md")
                && read_lossy(&path)
                    .lines()
                    .any(|line| line.trim_end() == marker)
            {
                return Ok(());
            }
        }
    }
    Err(format!("the specification states no rule {marker}"))
}

fn check_environment(row: &Row, context: &Context) -> Result<(), String> {
    let record = row
        .reason
        .split("; ")
        .find_map(|part| part.strip_prefix("record: "))
        .ok_or("no `record: PATH`")?;
    if !record.starts_with(ORACLE_RUNS) {
        return Err(format!("the record is not under {ORACLE_RUNS}"));
    }
    let tracked = Command::new("git")
        .arg("-C")
        .arg(&context.worktree)
        .args(["ls-files", "--error-unmatch", record])
        .output()
        .expect("cannot run git");
    if !tracked.status.success() {
        return Err(format!("{record} is not committed"));
    }
    if !read_lossy(&context.worktree.join(record)).contains(&row.test) {
        return Err(format!("{record} does not name the test"));
    }
    Ok(())
}

pub(super) fn check_not_run(row: &Row, text: &str, context: &Context) -> Result<(), String> {
    if row.reason == "platform windows" {
        return if row.group.starts_with(WINDOWS_DIR) {
            Ok(())
        } else {
            Err(format!("the group is not below {WINDOWS_DIR}"))
        };
    }
    if let Some(line) = row.reason.strip_prefix("restrictOS line ") {
        let number: usize = line.parse().map_err(|_| "not a line number")?;
        let source = text
            .lines()
            .nth(number.wrapping_sub(1))
            .ok_or("no such line")?;
        let lower = source.to_ascii_lowercase();
        let at = lower
            .find("restrictos(")
            .ok_or("the line holds no restrictOS")?;
        let argument = source[at + "restrictos(".len()..]
            .trim_start()
            .trim_start_matches(['\'', '"']);
        let argument = argument
            .split(['\'', '"'])
            .next()
            .unwrap_or("")
            .to_ascii_uppercase();
        let words: Vec<&str> = argument.split_whitespace().collect();
        return if words.is_empty() || words == ["UNIX"] || words.contains(&"LINUX") {
            Err(format!("restrictOS({argument:?}) admits LINUX"))
        } else {
            Ok(())
        };
    }
    if let Some(name) = row.reason.strip_prefix("test type ") {
        if !context.skipped_types.contains(name) {
            return Err(format!("{name} is a type the framework runs by default"));
        }
        let classes = source_classes(text);
        let tests = test_classes(&classes);
        let assignment = format!("~{}", name.to_ascii_lowercase());
        let typed = classes.iter().any(|class| {
            tests.contains(&class.name)
                && class.methods.contains(&row.test)
                && class.body.contains("ootesttype")
                && class.body.contains(&assignment)
        });
        return if typed {
            Ok(())
        } else {
            Err(format!("no test class defining it sets its type to {name}"))
        };
    }
    Err("the reason is not `platform windows`, `restrictOS line N` or `test type NAME`".to_string())
}
