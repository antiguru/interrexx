//! The CLI launcher uses the shared library for both the CLI and the C API.

#![cfg(unix)]

#[path = "../../interrexx-capi-rxapi/tests/support/names.rs"]
mod names;

#[test]
fn the_launcher_has_no_c_api_exports_of_its_own() {
    let launcher = std::path::Path::new(env!("CARGO_BIN_EXE_rexx"));
    assert!(names::exported_rexx_functions(launcher).is_empty());
}

#[cfg(target_os = "linux")]
#[path = "../cli_build.rs"]
mod cli_build;

#[cfg(target_os = "linux")]
#[test]
fn cargo_layouts_ignore_the_old_library_after_a_source_edit() {
    use std::ffi::OsStr;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    struct Fixture(PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let fixture = Fixture(
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("cli-build-{}", std::process::id())),
    );
    let root = &fixture.0;
    let engine = root.join("workspace/crates/interrexx");
    let source = engine.join("src/lib.rs");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    let unit = OsStr::new("same-cargo-unit");
    fs::write(&source, "pub const VALUE: u8 = 1;").unwrap();
    let old_build = cli_build::fingerprint(&engine, unit, 0);
    fs::write(&source, "pub const VALUE: u8 = 2;").unwrap();
    let build = cli_build::fingerprint(&engine, unit, 0);
    assert_ne!(old_build, build);

    let bin = root.join("build/checkout/target/debug");
    let exe = bin.join("rexx");
    let stale = bin.join("librexx.so.4");
    let compile = |library: &Path, build: u64| {
        fs::create_dir_all(library.parent().unwrap()).unwrap();
        let c = root.join("marker.c");
        let bytes = build.to_le_bytes().map(|byte| byte.to_string()).join(",");
        fs::write(&c, format!(
            "__attribute__((section(\".rexx_build\"))) const unsigned char interrexx_cli_build_v1[8] = {{{bytes}}};\n"
        )).unwrap();
        assert!(
            Command::new("cc")
                .args(["-shared", "-fPIC"])
                .arg(&c)
                .arg("-o")
                .arg(library)
                .status()
                .unwrap()
                .success()
        );
    };
    compile(&stale, old_build);
    assert!(interrexx_api::load::cli_library_path(&exe, build).is_err());

    for layout in [
        "deps/librexx.so",
        "build/interrexx-capi-unit/out/librexx.so",
        "build/interrexx-capi/unit/out/librexx.so",
    ] {
        let expected = bin.join(layout);
        compile(&expected, build);
        assert_eq!(
            interrexx_api::load::cli_library_path(&exe, build).unwrap(),
            expected
        );
        fs::remove_file(expected).unwrap();
    }

    let real_launcher = Path::new(env!("CARGO_BIN_EXE_rexx"));
    let real_build = env!("INTERREXX_CLI_BUILD").parse().unwrap();
    interrexx_api::load::cli_library_path(real_launcher, real_build)
        .expect("Cargo's real companion carries the identity");
}
