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
    }
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
