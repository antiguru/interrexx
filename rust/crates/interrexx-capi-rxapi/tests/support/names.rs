//! The C entry points each library exports, and the readings of a shared object the export tests
//! compare them against: `readelf` on ELF hosts, `nm` and `otool` on Apple's. Shared through `#[path]` by `interrexx-capi-rxapi`'s, `interrexx-capi`'s and `interrexx`'s tests.

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

/// The standard output of `tool arguments path`, which must succeed.
fn run_tool(tool: &str, arguments: &[&str], path: &Path) -> String {
    let output = Command::new(tool)
        .args(arguments)
        .arg(path)
        .output()
        .unwrap_or_else(|error| panic!("cannot run {tool} on {}: {error}", path.display()));
    assert!(
        output.status.success(),
        "{tool} {arguments:?} {} failed: {}",
        path.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap_or_else(|_| panic!("{tool} prints text"))
}

/// Whether a symbol name is one of the C API's: `Rexx` and an uppercase letter.
fn is_rexx_name(name: &str) -> bool {
    name.strip_prefix("Rexx")
        .and_then(|rest| rest.chars().next())
        .is_some_and(|first| first.is_ascii_uppercase())
}

/// The functions `path` exports whose names start `Rexx` and an uppercase letter.
#[cfg(not(target_vendor = "apple"))]
pub fn exported_rexx_functions(path: &Path) -> BTreeSet<String> {
    run_tool("readelf", &["--dyn-syms", "-W"], path)
        .lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            match fields.as_slice() {
                [_, _, _, "FUNC", "GLOBAL", _, section, name] if *section != "UND" => Some(*name),
                _ => None,
            }
        })
        .map(|name| name.split('@').next().unwrap_or(name))
        .filter(|name| is_rexx_name(name))
        .map(str::to_owned)
        .collect()
}

/// The functions `path` exports whose names start `Rexx` and an uppercase letter: the defined
/// external text symbols `nm` lists, without the C symbol prefix.
#[cfg(target_vendor = "apple")]
pub fn exported_rexx_functions(path: &Path) -> BTreeSet<String> {
    run_tool("nm", &["-gU"], path)
        .lines()
        .filter_map(
            |line| match line.split_whitespace().collect::<Vec<_>>().as_slice() {
                [_, "T", name] => name.strip_prefix('_'),
                _ => None,
            },
        )
        .filter(|name| is_rexx_name(name))
        .map(str::to_owned)
        .collect()
}

/// Every symbol `path`'s symbol tables name, one line each.
pub fn symbol_table(path: &Path) -> String {
    if cfg!(target_vendor = "apple") {
        // Without `-a`: it adds the debug map, whose `SO` entries name the source files of a
        // build with debug info, so a checkout path that contains a crate's name would read as
        // that crate's code. Local and global symbols are listed either way.
        run_tool("nm", &[], path)
    } else {
        run_tool("readelf", &["--syms", "-W"], path)
    }
}

/// `path`'s ELF dynamic section entries of `kind` (`NEEDED`, `SONAME`, `RPATH`), their bracketed
/// values.
#[cfg(not(target_vendor = "apple"))]
fn dynamic_entries(path: &Path, kind: &str) -> Vec<String> {
    let tag = format!("({kind})");
    run_tool("readelf", &["-d", "-W"], path)
        .lines()
        .filter(|line| line.contains(&tag))
        .filter_map(|line| {
            let start = line.find('[')?;
            let end = line.rfind(']')?;
            Some(line[start + 1..end].to_owned())
        })
        .collect()
}

/// The `name` values of `path`'s Mach-O load commands of type `command` (`LC_ID_DYLIB`,
/// `LC_LOAD_DYLIB`; `path` for `LC_RPATH`), as `otool -l` prints them.
#[cfg(target_vendor = "apple")]
fn load_commands(path: &Path, command: &str) -> Vec<String> {
    let listing = run_tool("otool", &["-l"], path);
    let mut found = Vec::new();
    let mut inside = false;
    for line in listing.lines().map(str::trim) {
        if let Some(cmd) = line.strip_prefix("cmd ") {
            inside = cmd == command;
        } else if inside
            && let Some(value) = line
                .strip_prefix("name ")
                .or_else(|| line.strip_prefix("path "))
        {
            // `name @rpath/librexx.4.dylib (offset 24)`
            let value = value
                .rsplit_once(" (offset")
                .map_or(value, |(value, _)| value);
            found.push(value.to_owned());
            inside = false;
        }
    }
    found
}

/// The name `path` records for itself, which a program linked against it records as its
/// dependency: its SONAME on ELF hosts, its install name on Apple's.
pub fn own_name(path: &Path) -> Vec<String> {
    #[cfg(target_vendor = "apple")]
    return load_commands(path, "LC_ID_DYLIB");
    #[cfg(not(target_vendor = "apple"))]
    return dynamic_entries(path, "SONAME");
}

/// The libraries `path` records as its dependencies, by the names it records for them.
pub fn needed(path: &Path) -> Vec<String> {
    #[cfg(target_vendor = "apple")]
    return load_commands(path, "LC_LOAD_DYLIB");
    #[cfg(not(target_vendor = "apple"))]
    return dynamic_entries(path, "NEEDED");
}

/// The directories `path` records for the loader to search: `DT_RPATH` on ELF hosts,
/// `LC_RPATH` on Apple's.
pub fn search_path(path: &Path) -> Vec<String> {
    #[cfg(target_vendor = "apple")]
    return load_commands(path, "LC_RPATH");
    #[cfg(not(target_vendor = "apple"))]
    return dynamic_entries(path, "RPATH");
}

/// The oracle's `lib` directory, where it is built; `None` when the oracle is not built there.
pub fn oracle_lib() -> Option<PathBuf> {
    let root = std::env::var_os("REXX_ORACLE_ROOT")?;
    let lib = PathBuf::from(root).join("lib");
    lib.join("librexx.so.4").exists().then_some(lib)
}
