use super::guards::guard_assertions;
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
    assert_eq!(
        assertion_lines("assertions 4 7-9 12; r[x]"),
        Ok(BTreeSet::from([4, 7, 8, 9, 12]))
    );
    for reason in [
        "r[x]",
        "assertions 4 r[x]",
        "assertions ; r[x]",
        "assertions 4  5; r[x]",
        "assertions 0; r[x]",
        "assertions +4; r[x]",
        "assertions 9-7; r[x]",
        "assertions 4 3-5; r[x]",
    ] {
        assert!(assertion_lines(reason).is_err(), "{reason}");
    }
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
