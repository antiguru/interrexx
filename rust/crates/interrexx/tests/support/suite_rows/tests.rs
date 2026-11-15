use super::guards::{Condition, condition_span, guard_assertions, guard_conditions};
use super::*;
use crate::group_runner::RUNNING_TEST;

const WINDOWS_GROUP: &str = "extensions/platform/windows/fixture/rows";
const OTHER_GROUP: &str = "extensions/fixture/rows";

fn fixture() -> (PathBuf, Context) {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("suite-row-classes-{}-{id}", std::process::id()));
    let text = "::class helper\n::method TEST_HELPER\n::class actual subclass ooTestCase\n::method TEST_REAL\n::method ordinary\n::class inherited subclass actual\n::method TEST_INHERITED\n";
    for group in [WINDOWS_GROUP, OTHER_GROUP] {
        let file = root.join(format!("{group}.testGroup"));
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, text).unwrap();
    }
    let context = Context {
        root: root.clone(),
        groups: BTreeSet::from([WINDOWS_GROUP.to_string(), OTHER_GROUP.to_string()]),
        deviations: BTreeSet::new(),
        flaky: BTreeSet::new(),
        skipped_types: BTreeSet::new(),
        worktree: root.clone(),
    };
    (root, context)
}

fn windows_row(group: &str, test: &str) -> Row {
    Row {
        file: NOT_RUN.to_string(),
        line: 1,
        group: group.to_string(),
        test: test.to_string(),
        owner: Owner::NotRun,
        reason: "platform windows".to_string(),
    }
}

#[test]
fn only_tests_of_test_classes_can_own_suite_rows() {
    let (root, context) = fixture();
    for name in ["TEST_REAL", "TEST_INHERITED"] {
        assert!(
            check_row(&windows_row(WINDOWS_GROUP, name), &context).is_ok(),
            "{name}"
        );
    }
    for name in ["TEST_HELPER", "ORDINARY", "TEST_MISSING"] {
        assert!(
            check_row(&windows_row(WINDOWS_GROUP, name), &context).is_err(),
            "{name}"
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_platform_windows_row_is_derived_and_checked_only_below_the_windows_directory() {
    let (root, context) = fixture();
    assert!(check_row(&windows_row(WINDOWS_GROUP, "TEST_REAL"), &context).is_ok());
    assert!(check_row(&windows_row(OTHER_GROUP, "TEST_REAL"), &context).is_err());
    assert_eq!(
        derived_rows(&context),
        [
            format!("{WINDOWS_GROUP}\tTEST_INHERITED\tNotRun\tplatform windows"),
            format!("{WINDOWS_GROUP}\tTEST_REAL\tNotRun\tplatform windows"),
        ]
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn an_unspecified_rows_lines_are_numbers_and_ranges_each_named_once() {
    let named = named_lines("assertions 4 7-9 12; r[x]").unwrap();
    assert_eq!(named.assertions, BTreeSet::from([4, 7, 8, 9, 12]));
    assert!(named.conditions.is_empty());
    let named = named_lines("assertions 4; conditions 6 9-10 as a~b > 1 & c; r[x]").unwrap();
    assert_eq!(named.assertions, BTreeSet::from([4]));
    assert_eq!(named.conditions, BTreeSet::from([6, 9, 10]));
    assert_eq!(named.replacement, "a~b > 1 & c");
    let named = named_lines("conditions 6 as x; r[x]").unwrap();
    assert!(named.assertions.is_empty());
    assert_eq!(named.conditions, BTreeSet::from([6]));
    for reason in [
        "r[x]",
        "assertions 4 r[x]",
        "assertions ; r[x]",
        "assertions 4  5; r[x]",
        "assertions 0; r[x]",
        "assertions +4; r[x]",
        "assertions 9-7; r[x]",
        "assertions 4 3-5; r[x]",
        "conditions 4; r[x]",
        "conditions 4 as ; r[x]",
        "conditions 4 as x r[x]",
        "conditions 4 3-5 as x; r[x]",
        "assertions 4; conditions 4 as x; r[x]",
    ] {
        assert!(named_lines(reason).is_err(), "{reason}");
    }
}

#[test]
fn a_condition_is_the_text_of_an_if_between_its_if_and_its_then() {
    let span = |line: &str| condition_span(line.as_bytes()).map(|span| line[span].to_string());
    assert_eq!(span("  IF a | b  THEN  "), Some("a | b".to_string()));
    assert_eq!(span("if x then\r\n"), Some("x".to_string()));
    for line in [
        "  self~assertTrue(1)",
        "if then",
        "if x",
        "ifx then",
        "if x then do",
        "",
    ] {
        assert_eq!(span(line), None, "{line:?}");
    }
}

#[test]
fn a_condition_guard_keeps_its_lines_and_replaces_the_test_only_for_its_tests() {
    let file = std::env::temp_dir().join(format!(
        "suite-row-condition-{}.testGroup",
        std::process::id()
    ));
    let text = "::method test_a\n  if p | q then\n    say 1\n  else say 2\n  say 'done'\n";
    std::fs::write(&file, text).unwrap();
    let guards = BTreeMap::from([(
        2,
        Condition {
            tests: BTreeSet::from(["TEST_A".to_string(), "TEST_B".to_string()]),
            replacement: "r".to_string(),
        },
    )]);
    guard_conditions(&file, &guards);
    let running = format!("wordpos(.{RUNNING_TEST}, 'TEST_A TEST_B')");
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        format!(
            "::method test_a\n  if ({running} > 0 & (r)) | ({running} = 0 & (p | q)) then\n    say 1\n  else say 2\n  say 'done'\n"
        )
    );
    for line in [1, 3, 5, 6] {
        std::fs::write(&file, text).unwrap();
        let named = BTreeMap::from([(
            line,
            Condition {
                tests: BTreeSet::from(["TEST_A".to_string()]),
                replacement: "r".to_string(),
            },
        )]);
        let file = file.clone();
        assert!(
            std::panic::catch_unwind(move || guard_conditions(&file, &named)).is_err(),
            "line {line} is guarded"
        );
    }
    std::fs::remove_file(file).unwrap();
}

#[test]
fn a_guard_keeps_its_lines_and_skips_the_assertion_only_for_its_tests() {
    let file =
        std::env::temp_dir().join(format!("suite-row-guard-{}.testGroup", std::process::id()));
    let text = "::method test_a\n  self~assertSame(1, 1)\n  if x then\n    self~assertFail('x')\n  else nop\n  say 'done' -- self~assertTrue(0)\n";
    std::fs::write(&file, text).unwrap();
    let guards = BTreeMap::from([
        (
            2,
            BTreeSet::from(["TEST_A".to_string(), "TEST_B".to_string()]),
        ),
        (3, BTreeSet::from(["TEST_A".to_string()])),
    ]);
    guard_assertions(&file, &guards);
    let guard = |names: &str| format!("if wordpos(.{RUNNING_TEST}, '{names}') > 0 then nop; else ");
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        format!(
            "::method test_a\n  {}self~assertSame(1, 1)\n  {}if x then\n    self~assertFail('x')\n  else nop\n  say 'done' -- self~assertTrue(0)\n",
            guard("TEST_A TEST_B"),
            guard("TEST_A")
        )
    );
    for line in [1, 5, 6, 7] {
        std::fs::write(&file, text).unwrap();
        let named = BTreeMap::from([(line, BTreeSet::from(["TEST_A".to_string()]))]);
        let file = file.clone();
        assert!(
            std::panic::catch_unwind(move || guard_assertions(&file, &named)).is_err(),
            "line {line} is guarded"
        );
    }
    std::fs::remove_file(file).unwrap();
}
