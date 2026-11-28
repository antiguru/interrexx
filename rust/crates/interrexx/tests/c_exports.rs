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
#[test]
fn a_newer_cargo_library_from_another_build_is_ignored() {
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    let launcher = Path::new(env!("CARGO_BIN_EXE_rexx"));
    let build = env!("INTERREXX_CLI_BUILD")
        .parse()
        .expect("the launcher build ID");
    let companion = interrexx_api::load::cli_library_path(launcher, build)
        .expect("the launcher has a matching companion");
    let bin =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("cli-build-{}", std::process::id()));
    let units = bin.join("build/interrexx-capi");
    let expected = units.join("matched/out/librexx.so");
    let stale = units.join("different/out/librexx.so");
    for library in [&expected, &stale] {
        fs::create_dir_all(library.parent().expect("the library directory"))
            .expect("create the Cargo fixture");
    }
    fs::hard_link(&companion, &expected)
        .or_else(|_| fs::copy(&companion, &expected).map(|_| ()))
        .expect("stage the matching library");
    let c = bin.join("stale.c");
    fs::write(
        &c,
        format!(
            "unsigned long long interrexx_cli_build_v1(void) {{ return {}ULL; }}\n",
            build ^ 1_u64
        ),
    )
    .expect("write the stale library source");
    assert!(
        Command::new("cc")
            .args(["-shared", "-fPIC"])
            .arg(&c)
            .arg("-o")
            .arg(&stale)
            .status()
            .expect("run cc")
            .success()
    );
    fs::File::open(&stale)
        .expect("open stale library")
        .set_times(
            fs::FileTimes::new().set_modified(
                std::time::UNIX_EPOCH + std::time::Duration::from_secs(4_000_000_000),
            ),
        )
        .expect("make the wrong build newer");
    assert_eq!(
        interrexx_api::load::cli_library_path(&bin.join("rexx"), build).expect("select a library"),
        expected
    );
    fs::remove_dir_all(bin).expect("remove the fixture");
}
