//! The routines the oracle's two internal packages export, which are neither
//! builtins nor `::ROUTINE`s: `REXX` (`runtime/NativeFunctions.h`) and
//! `REXXUTIL` (`runtime/RexxUtilCommon.cpp`'s table plus the platform half in
//! `platform/unix/SysRexxUtilFunctions.h`). The oracle consults them after a
//! program's own routines and before the external file search, so a name here
//! that this crate has no body for must refuse rather than fall through to
//! 43.1 -- a "routine not found" a program cannot tell from its own typo.
//! `tests/internal_routines.rs` re-derives this table from those three C++
//! files.

/// Which internal package exports the routine.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum Package {
    /// `runtime/NativeFunctions.h`, loaded as the `REXX` package.
    Rexx,
    /// `runtime/RexxUtilCommon.cpp`, loaded as the `REXXUTIL` package.
    RexxUtil,
}

impl Package {
    /// The internal package a `LIBRARY` name or `::REQUIRES ... LIBRARY`
    /// names, matched byte for byte as `PackageManager::loadLibrary` matches
    /// its table (`package/PackageManager.cpp:81-88`, `memory/GlobalNames.h`):
    /// measured, oracle, `LIBRARY REXXUTIL` is 98.903 where `LIBRARY
    /// rexxutil` loads.
    pub(crate) fn named(name: &[u8]) -> Option<Package> {
        match name {
            b"REXX" => Some(Package::Rexx),
            b"rexxutil" => Some(Package::RexxUtil),
            _ => None,
        }
    }

    /// The package's name, as the C++ registers it.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Package::Rexx => "REXX",
            Package::RexxUtil => "REXXUTIL",
        }
    }
}

/// What one internal routine runs: the same shape a builtin's body has, since
/// both are entered with their arguments already evaluated and neither pushes
/// an activation.
pub(crate) type InternalBody = fn(
    &mut crate::Interp,
    &'static [u8],
    &[Option<rexx_core::ObjRef>],
) -> Result<rexx_core::ObjRef, crate::Failure>;

/// One routine an internal package exports.
pub(crate) struct InternalRoutine {
    /// The name as the C++ registers it; the lookup is caseless, so the
    /// spelling is for a reader.
    pub(crate) name: &'static str,
    pub(crate) package: Package,
    /// The code, once some phase has written it. A row with `None` refuses.
    pub(crate) body: Option<InternalBody>,
}

/// A `REXX`-package row this crate runs.
const fn rexx_run(name: &'static str, body: InternalBody) -> InternalRoutine {
    InternalRoutine {
        name,
        package: Package::Rexx,
        body: Some(body),
    }
}

/// A `REXXUTIL` row this crate runs.
const fn util_run(name: &'static str, body: InternalBody) -> InternalRoutine {
    InternalRoutine {
        name,
        package: Package::RexxUtil,
        body: Some(body),
    }
}

/// Every name the two packages register on Linux.
pub(crate) static INTERNAL_ROUTINES: &[InternalRoutine] = &[
    rexx_run("Directory", crate::builtin::platform::directory),
    rexx_run("Filespec", crate::builtin::platform::filespec),
    rexx_run("Beep", crate::builtin::platform::beep),
    util_run("SysAddRexxMacro", crate::builtin::rexxutil::macros::add),
    util_run(
        "SysClearRexxMacroSpace",
        crate::builtin::rexxutil::macros::clear,
    ),
    util_run("SysCloseEventSem", crate::builtin::rexxutil::close_sem),
    util_run("SysCloseMutexSem", crate::builtin::rexxutil::close_sem),
    util_run("SysCls", crate::builtin::rexxutil::process::cls),
    util_run(
        "SysCreateEventSem",
        crate::builtin::rexxutil::create_event_sem,
    ),
    util_run(
        "SysCreateMutexSem",
        crate::builtin::rexxutil::create_mutex_sem,
    ),
    util_run(
        "SysCreatePipe",
        crate::builtin::rexxutil::process::create_pipe,
    ),
    util_run(
        "SysDropFuncs",
        crate::builtin::rexxutil::process::load_funcs,
    ),
    util_run(
        "SysDropRexxMacro",
        crate::builtin::rexxutil::macros::drop_macro,
    ),
    util_run(
        "SysDumpVariables",
        crate::builtin::rexxutil::stems::dump_variables,
    ),
    util_run("SysFileCopy", crate::builtin::rexxutil::files::file_copy),
    util_run("SysFileDelete", crate::builtin::rexxutil::file_delete),
    util_run("SysFileExists", crate::builtin::rexxutil::file_exists),
    util_run("SysFileMove", crate::builtin::rexxutil::files::file_move),
    util_run(
        "SysFileSearch",
        crate::builtin::rexxutil::files::file_search,
    ),
    util_run("SysFileTree", crate::builtin::rexxutil::tree::file_tree),
    util_run("SysFork", crate::builtin::rexxutil::process::fork),
    util_run(
        "SysFormatMessage",
        crate::builtin::rexxutil::messages::format,
    ),
    util_run(
        "SysGetErrorText",
        crate::builtin::rexxutil::messages::error_text,
    ),
    util_run(
        "SysGetFileDateTime",
        crate::builtin::rexxutil::files::get_file_date_time,
    ),
    util_run("SysGetKey", crate::builtin::rexxutil::process::get_key),
    util_run(
        "SysGetMessage",
        crate::builtin::rexxutil::messages::get_message,
    ),
    util_run(
        "SysGetMessageX",
        crate::builtin::rexxutil::messages::get_message_x,
    ),
    util_run("SysIsFile", crate::builtin::rexxutil::is_file),
    util_run(
        "SysIsFileDirectory",
        crate::builtin::rexxutil::files::is_file_directory,
    ),
    util_run(
        "SysIsFileLink",
        crate::builtin::rexxutil::files::is_file_link,
    ),
    util_run("SysLinVer", crate::builtin::rexxutil::version),
    util_run(
        "SysLoadFuncs",
        crate::builtin::rexxutil::process::load_funcs,
    ),
    util_run(
        "SysLoadRexxMacroSpace",
        crate::builtin::rexxutil::macros::load,
    ),
    util_run("SysMkDir", crate::builtin::rexxutil::mk_dir),
    util_run("SysOpenEventSem", crate::builtin::rexxutil::open_sem),
    util_run("SysOpenMutexSem", crate::builtin::rexxutil::open_sem),
    util_run("SysPostEventSem", crate::builtin::rexxutil::post_event_sem),
    util_run(
        "SysQueryProcess",
        crate::builtin::rexxutil::process::query_process,
    ),
    util_run("SysQueryRexxMacro", crate::builtin::rexxutil::macros::query),
    util_run(
        "SysReleaseMutexSem",
        crate::builtin::rexxutil::release_mutex_sem,
    ),
    util_run(
        "SysReorderRexxMacro",
        crate::builtin::rexxutil::macros::reorder,
    ),
    util_run("SysRequestMutexSem", crate::builtin::rexxutil::wait_sem),
    util_run(
        "SysResetEventSem",
        crate::builtin::rexxutil::reset_event_sem,
    ),
    util_run("SysRmDir", crate::builtin::rexxutil::rm_dir),
    util_run(
        "SysSaveRexxMacroSpace",
        crate::builtin::rexxutil::macros::save,
    ),
    util_run(
        "SysSearchPath",
        crate::builtin::rexxutil::files::search_path,
    ),
    util_run(
        "SysSetFileDateTime",
        crate::builtin::rexxutil::files::set_file_date_time,
    ),
    util_run(
        "SysSetPriority",
        crate::builtin::rexxutil::process::set_priority,
    ),
    util_run("SysSleep", crate::builtin::rexxutil::sleep),
    util_run("SysStemCopy", crate::builtin::rexxutil::stems::copy),
    util_run("SysStemDelete", crate::builtin::rexxutil::stems::delete),
    util_run("SysStemInsert", crate::builtin::rexxutil::stems::insert),
    util_run("SysStemSort", crate::builtin::rexxutil::stems::sort),
    util_run(
        "SysTempFileName",
        crate::builtin::rexxutil::files::temp_file_name,
    ),
    util_run(
        "SysUtilVersion",
        crate::builtin::rexxutil::process::util_version,
    ),
    util_run("SysVersion", crate::builtin::rexxutil::version),
    util_run("SysWait", crate::builtin::rexxutil::process::wait),
    util_run("SysWaitEventSem", crate::builtin::rexxutil::wait_sem),
];

/// Every row `package` exports, in table order.
fn package_rows(package: Package) -> impl Iterator<Item = &'static InternalRoutine> {
    INTERNAL_ROUTINES
        .iter()
        .filter(move |row| row.package == package)
}

/// `row`'s [`INTERNAL_ROUTINES`] index.
pub(crate) fn index_of(row: &'static InternalRoutine) -> usize {
    INTERNAL_ROUTINES
        .iter()
        .position(|candidate| std::ptr::eq(candidate, row))
        .expect("every row is an INTERNAL_ROUTINES element")
}

/// The [`INTERNAL_ROUTINES`] index of every row `package` exports.
pub(crate) fn package_indexes(package: Package) -> impl Iterator<Item = usize> {
    INTERNAL_ROUTINES
        .iter()
        .enumerate()
        .filter(move |(_, row)| row.package == package)
        .map(|(index, _)| index)
}

/// `package`'s row `name` names, found as `LibraryPackage::resolveRoutine`
/// finds a routine table entry
/// (`interpreter/package/LibraryPackage.cpp:410-435`): by exact spelling, then
/// without regard to case.
pub(crate) fn package_routine(package: Package, name: &[u8]) -> Option<&'static InternalRoutine> {
    package_rows(package)
        .find(|row| row.name.as_bytes() == name)
        .or_else(|| {
            package_rows(package).find(|row| row.name.as_bytes().eq_ignore_ascii_case(name))
        })
}

/// The row `name` names, matched caselessly as the oracle's own lookup does.
pub(crate) fn lookup(name: &[u8]) -> Option<&'static InternalRoutine> {
    INTERNAL_ROUTINES
        .iter()
        .find(|row| row.name.as_bytes().eq_ignore_ascii_case(name))
}
