#[test]
fn control_reads_the_submodule() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../third_party/ooRexx/api/rexx.h");
    std::fs::read(&root).expect("control: reads the submodule");
}
