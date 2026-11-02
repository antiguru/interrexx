//! The C entry points each library exports, and the ELF readings the export tests compare them
//! against. Shared through `#[path]` by `librexxapi`'s, `librexx`'s and `rexx-exec`'s tests.

#![allow(dead_code)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// `librexx.so.4`'s own names.
pub const LIBREXX: &[&str] = &[
    "RexxCallBack",
    "RexxCompileProgram",
    "RexxCreateInterpreter",
    "RexxCreateInterpreterImage",
    "RexxDidRexxTerminate",
    "RexxGetErrorMessage",
    "RexxGetErrorMessageByNumber",
    "RexxGetVersionInformation",
    "RexxResetTrace",
    "RexxSetHalt",
    "RexxSetTrace",
    "RexxStart",
    "RexxStemSort",
    "RexxTranslateInstoreProgram",
    "RexxTranslateProgram",
    "RexxVariablePool",
    "RexxWaitForTermination",
];

/// `librexxapi.so.4`'s names.
pub const LIBREXXAPI: &[&str] = &[
    "RexxAddMacro",
    "RexxAddQueue",
    "RexxAllocateMemory",
    "RexxClearMacroSpace",
    "RexxClearQueue",
    "RexxCreateQueue",
    "RexxCreateSessionQueue",
    "RexxDeleteQueue",
    "RexxDeleteSessionQueue",
    "RexxDeregisterExit",
    "RexxDeregisterFunction",
    "RexxDeregisterSubcom",
    "RexxDropMacro",
    "RexxFreeMemory",
    "RexxLoadMacroSpace",
    "RexxLoadSubcom",
    "RexxOpenQueue",
    "RexxPullFromQueue",
    "RexxPullQueue",
    "RexxQueryExit",
    "RexxQueryFunction",
    "RexxQueryMacro",
    "RexxQueryQueue",
    "RexxQuerySubcom",
    "RexxQueueExists",
    "RexxRegisterExitDll",
    "RexxRegisterExitExe",
    "RexxRegisterFunctionDll",
    "RexxRegisterFunctionExe",
    "RexxRegisterSubcomDll",
    "RexxRegisterSubcomExe",
    "RexxReorderMacro",
    "RexxSaveMacroSpace",
    "RexxShutDownAPI",
];

/// The oracle's internal names, which neither library exports.
pub const NOT_EXPORTED: &[&str] = &[
    "RexxResolveExit",
    "RexxResolveMacroFunction",
    "RexxResolveRoutine",
    "RexxResolveSubcom",
];

pub fn set(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

fn readelf(arguments: &[&str], path: &Path) -> String {
    let output = Command::new("readelf")
        .args(arguments)
        .arg(path)
        .output()
        .unwrap_or_else(|error| panic!("cannot run readelf on {}: {error}", path.display()));
    assert!(
        output.status.success(),
        "readelf {arguments:?} {} failed: {}",
        path.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("readelf prints text")
}

/// The functions `path` defines in its dynamic symbol table whose names start `Rexx` and an
/// uppercase letter.
pub fn exported_rexx_functions(path: &Path) -> BTreeSet<String> {
    readelf(&["--dyn-syms", "-W"], path)
        .lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            match fields.as_slice() {
                [_, _, _, "FUNC", "GLOBAL", _, section, name] if *section != "UND" => Some(*name),
                _ => None,
            }
        })
        .map(|name| name.split('@').next().unwrap_or(name))
        .filter(|name| {
            name.strip_prefix("Rexx")
                .and_then(|rest| rest.chars().next())
                .is_some_and(|first| first.is_ascii_uppercase())
        })
        .map(str::to_owned)
        .collect()
}

/// `path`'s dynamic section entries of `kind` (`NEEDED`, `SONAME`, `RUNPATH`), their bracketed
/// values.
pub fn dynamic_entries(path: &Path, kind: &str) -> Vec<String> {
    let tag = format!("({kind})");
    readelf(&["-d", "-W"], path)
        .lines()
        .filter(|line| line.contains(&tag))
        .filter_map(|line| {
            let start = line.find('[')?;
            let end = line.rfind(']')?;
            Some(line[start + 1..end].to_owned())
        })
        .collect()
}

/// The profile's `deps` directory this test binary runs from, where cargo leaves the workspace's
/// shared objects.
pub fn deps_dir() -> PathBuf {
    std::env::current_exe()
        .expect("the test binary's path")
        .parent()
        .expect("the test binary is in deps")
        .to_path_buf()
}

/// The oracle's `lib` directory, where it is built.
pub fn oracle_lib() -> Option<PathBuf> {
    let root = std::env::var_os("REXX_ORACLE_ROOT")?;
    let lib = PathBuf::from(root).join("lib");
    lib.join("librexx.so.4").exists().then_some(lib)
}
