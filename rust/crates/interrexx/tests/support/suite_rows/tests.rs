use super::*;

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
