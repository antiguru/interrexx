//! The routines the two internal packages export, which are neither builtins nor
//! `::ROUTINE`s: `REXX` and `REXXUTIL` (a shared table plus a platform half). They
//! are consulted after a program's own routines and before the external file search,
//! so a name here that this crate has no body for must refuse rather than fall
//! through to 43.1 -- a "routine not found" a program cannot tell from its own typo.
//! `tests/internal_routines.rs` re-derives this table from the reference build.

/// Which internal package exports the routine.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum Package {
    /// Loaded as the `REXX` package.
    Rexx,
    /// Loaded as the `REXXUTIL` package.
    RexxUtil,
}

impl Package {
    /// The internal package a `LIBRARY` name or `::REQUIRES ... LIBRARY` names,
    /// matched byte for byte: `LIBRARY REXXUTIL` is 98.903 where `LIBRARY
    /// rexxutil` loads.
    pub(crate) fn named(name: &[u8]) -> Option<Package> {
        match name {
            b"REXX" => Some(Package::Rexx),
            b"rexxutil" => Some(Package::RexxUtil),
            _ => None,
        }
    }

    /// The package's name, as registered.
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
    &[Option<interrexx_core::ObjRef>],
) -> Result<interrexx_core::ObjRef, crate::Failure>;

/// One routine an internal package exports.
pub(crate) struct InternalRoutine {
    /// The name as registered; the lookup is caseless, so the spelling is for a
    /// reader.
    pub(crate) name: &'static str,
    pub(crate) package: Package,
    /// The code. A row with `None` refuses.
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

/// Names the two packages register on this platform.
pub(crate) static INTERNAL_ROUTINES: &[InternalRoutine] = &[
    rexx_run("Directory", crate::builtin::platform::directory),
    rexx_run("Filespec", crate::builtin::platform::filespec),
    rexx_run("Beep", crate::builtin::platform::beep),
    #[cfg(windows)]
    util_run(
        "RxMessageBox",
        crate::builtin::rexxutil::windows_interaction::message_box,
    ),
    #[cfg(windows)]
    util_run(
        "RxWinExec",
        crate::builtin::rexxutil::windows_process::win_exec,
    ),
    util_run("SysAddRexxMacro", crate::builtin::rexxutil::macros::add),
    #[cfg(windows)]
    util_run(
        "SysBootDrive",
        crate::builtin::rexxutil::windows_files::boot_drive,
    ),
    util_run(
        "SysClearRexxMacroSpace",
        crate::builtin::rexxutil::macros::clear,
    ),
    util_run("SysCloseEventSem", crate::builtin::rexxutil::close_sem),
    util_run("SysCloseMutexSem", crate::builtin::rexxutil::close_sem),
    #[cfg(windows)]
    util_run("SysCls", crate::builtin::rexxutil::cls),
    #[cfg(unix)]
    util_run("SysCls", crate::builtin::rexxutil::process::cls),
    util_run(
        "SysCreateEventSem",
        crate::builtin::rexxutil::create_event_sem,
    ),
    util_run(
        "SysCreateMutexSem",
        crate::builtin::rexxutil::create_mutex_sem,
    ),
    #[cfg(unix)]
    util_run(
        "SysCreatePipe",
        crate::builtin::rexxutil::process::create_pipe,
    ),
    #[cfg(windows)]
    util_run(
        "SysCurPos",
        crate::builtin::rexxutil::windows_console::cursor_position,
    ),
    #[cfg(windows)]
    util_run(
        "SysCurState",
        crate::builtin::rexxutil::windows_console::cursor_state,
    ),
    #[cfg(windows)]
    util_run(
        "SysDriveInfo",
        crate::builtin::rexxutil::windows_files::drive_info,
    ),
    #[cfg(windows)]
    util_run(
        "SysDriveMap",
        crate::builtin::rexxutil::windows_files::drive_map,
    ),
    util_run("SysDropFuncs", crate::builtin::rexxutil::load_funcs),
    util_run(
        "SysDropRexxMacro",
        crate::builtin::rexxutil::macros::drop_macro,
    ),
    util_run(
        "SysDumpVariables",
        crate::builtin::rexxutil::stems::dump_variables,
    ),
    #[cfg(windows)]
    util_run(
        "SysFileCopy",
        crate::builtin::rexxutil::windows_files::file_copy,
    ),
    #[cfg(unix)]
    util_run("SysFileCopy", crate::builtin::rexxutil::files::file_copy),
    #[cfg(windows)]
    util_run(
        "SysFileDelete",
        crate::builtin::rexxutil::windows_files::file_delete,
    ),
    #[cfg(unix)]
    util_run("SysFileDelete", crate::builtin::rexxutil::file_delete),
    util_run("SysFileExists", crate::builtin::rexxutil::file_exists),
    #[cfg(windows)]
    util_run(
        "SysFileMove",
        crate::builtin::rexxutil::windows_files::file_move,
    ),
    #[cfg(unix)]
    util_run("SysFileMove", crate::builtin::rexxutil::files::file_move),
    util_run(
        "SysFileSearch",
        crate::builtin::rexxutil::search::file_search,
    ),
    #[cfg(windows)]
    util_run(
        "SysFileSystemType",
        crate::builtin::rexxutil::windows_files::file_system_type,
    ),
    util_run("SysFileTree", crate::builtin::rexxutil::tree::file_tree),
    #[cfg(unix)]
    util_run("SysFork", crate::builtin::rexxutil::process::fork),
    util_run(
        "SysFormatMessage",
        crate::builtin::rexxutil::messages::format,
    ),
    #[cfg(windows)]
    util_run(
        "SysFromUnicode",
        crate::builtin::rexxutil::windows_encoding::from_unicode,
    ),
    util_run(
        "SysGetErrorText",
        crate::builtin::rexxutil::messages::error_text,
    ),
    #[cfg(windows)]
    util_run(
        "SysGetFileDateTime",
        crate::builtin::rexxutil::windows_files::get_file_date_time,
    ),
    #[cfg(unix)]
    util_run(
        "SysGetFileDateTime",
        crate::builtin::rexxutil::files::get_file_date_time,
    ),
    #[cfg(windows)]
    util_run(
        "SysGetKey",
        crate::builtin::rexxutil::windows_interaction::get_key,
    ),
    #[cfg(unix)]
    util_run("SysGetKey", crate::builtin::rexxutil::process::get_key),
    #[cfg(windows)]
    util_run(
        "SysGetLongPathName",
        crate::builtin::rexxutil::windows_files::file_name_spelling::<false>,
    ),
    util_run(
        "SysGetMessage",
        crate::builtin::rexxutil::messages::get_message,
    ),
    util_run(
        "SysGetMessageX",
        crate::builtin::rexxutil::messages::get_message_x,
    ),
    #[cfg(windows)]
    util_run(
        "SysGetShortPathName",
        crate::builtin::rexxutil::windows_files::file_name_spelling::<true>,
    ),
    #[cfg(windows)]
    util_run("SysIni", crate::builtin::rexxutil::windows_ini::ini),
    util_run("SysIsFile", crate::builtin::rexxutil::is_file),
    #[cfg(windows)]
    util_run(
        "SysIsFileCompressed",
        crate::builtin::rexxutil::windows_files::has_attribute::<
            { windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_COMPRESSED },
        >,
    ),
    #[cfg(windows)]
    util_run(
        "SysIsFileDirectory",
        crate::builtin::rexxutil::windows_files::is_directory,
    ),
    #[cfg(unix)]
    util_run(
        "SysIsFileDirectory",
        crate::builtin::rexxutil::files::is_file_directory,
    ),
    #[cfg(windows)]
    util_run(
        "SysIsFileEncrypted",
        crate::builtin::rexxutil::windows_files::has_attribute::<
            { windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_ENCRYPTED },
        >,
    ),
    #[cfg(windows)]
    util_run(
        "SysIsFileLink",
        crate::builtin::rexxutil::windows_files::is_link,
    ),
    #[cfg(unix)]
    util_run(
        "SysIsFileLink",
        crate::builtin::rexxutil::files::is_file_link,
    ),
    #[cfg(windows)]
    util_run(
        "SysIsFileNotContentIndexed",
        crate::builtin::rexxutil::windows_files::has_attribute::<
            { windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NOT_CONTENT_INDEXED },
        >,
    ),
    #[cfg(windows)]
    util_run(
        "SysIsFileOffline",
        crate::builtin::rexxutil::windows_files::has_attribute::<
            { windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_OFFLINE },
        >,
    ),
    #[cfg(windows)]
    util_run(
        "SysIsFileSparse",
        crate::builtin::rexxutil::windows_files::has_attribute::<
            { windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_SPARSE_FILE },
        >,
    ),
    #[cfg(windows)]
    util_run(
        "SysIsFileTemporary",
        crate::builtin::rexxutil::windows_files::has_attribute::<
            { windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_TEMPORARY },
        >,
    ),
    util_run("SysLinVer", crate::builtin::rexxutil::version),
    util_run("SysLoadFuncs", crate::builtin::rexxutil::load_funcs),
    util_run(
        "SysLoadRexxMacroSpace",
        crate::builtin::rexxutil::macros::load,
    ),
    #[cfg(windows)]
    util_run("SysMkDir", crate::builtin::rexxutil::windows_files::mk_dir),
    #[cfg(unix)]
    util_run("SysMkDir", crate::builtin::rexxutil::mk_dir),
    util_run("SysOpenEventSem", crate::builtin::rexxutil::open_sem),
    util_run("SysOpenMutexSem", crate::builtin::rexxutil::open_sem),
    util_run("SysPostEventSem", crate::builtin::rexxutil::post_event_sem),
    #[cfg(windows)]
    util_run(
        "SysQueryProcess",
        crate::builtin::rexxutil::windows_process::query_process,
    ),
    #[cfg(unix)]
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
    #[cfg(windows)]
    util_run("SysRmDir", crate::builtin::rexxutil::windows_files::rm_dir),
    #[cfg(unix)]
    util_run("SysRmDir", crate::builtin::rexxutil::rm_dir),
    util_run(
        "SysSaveRexxMacroSpace",
        crate::builtin::rexxutil::macros::save,
    ),
    util_run(
        "SysSearchPath",
        crate::builtin::rexxutil::search::search_path,
    ),
    #[cfg(windows)]
    util_run(
        "SysSetFileDateTime",
        crate::builtin::rexxutil::windows_files::set_file_date_time,
    ),
    #[cfg(unix)]
    util_run(
        "SysSetFileDateTime",
        crate::builtin::rexxutil::files::set_file_date_time,
    ),
    #[cfg(windows)]
    util_run(
        "SysSetPriority",
        crate::builtin::rexxutil::windows_process::set_priority,
    ),
    #[cfg(unix)]
    util_run(
        "SysSetPriority",
        crate::builtin::rexxutil::process::set_priority,
    ),
    #[cfg(windows)]
    util_run(
        "SysShutdownSystem",
        crate::builtin::rexxutil::windows_process::shutdown_system,
    ),
    util_run("SysSleep", crate::builtin::rexxutil::sleep),
    util_run("SysStemCopy", crate::builtin::rexxutil::stems::copy),
    util_run("SysStemDelete", crate::builtin::rexxutil::stems::delete),
    util_run("SysStemInsert", crate::builtin::rexxutil::stems::insert),
    util_run("SysStemSort", crate::builtin::rexxutil::stems::sort),
    #[cfg(windows)]
    util_run(
        "SysSwitchSession",
        crate::builtin::rexxutil::windows_interaction::switch_session,
    ),
    #[cfg(windows)]
    util_run(
        "SysSystemDirectory",
        crate::builtin::rexxutil::windows_files::system_directory,
    ),
    util_run(
        "SysTempFileName",
        crate::builtin::rexxutil::search::temp_file_name,
    ),
    #[cfg(windows)]
    util_run(
        "SysTextScreenRead",
        crate::builtin::rexxutil::windows_console::screen_read,
    ),
    #[cfg(windows)]
    util_run(
        "SysTextScreenSize",
        crate::builtin::rexxutil::windows_console::screen_size,
    ),
    #[cfg(windows)]
    util_run(
        "SysToUnicode",
        crate::builtin::rexxutil::windows_encoding::to_unicode,
    ),
    util_run("SysUtilVersion", crate::builtin::rexxutil::util_version),
    util_run("SysVersion", crate::builtin::rexxutil::version),
    #[cfg(windows)]
    util_run(
        "SysVolumeLabel",
        crate::builtin::rexxutil::windows_files::volume_label,
    ),
    #[cfg(unix)]
    util_run("SysWait", crate::builtin::rexxutil::process::wait),
    #[cfg(windows)]
    util_run(
        "SysWaitNamedPipe",
        crate::builtin::rexxutil::windows_interaction::wait_named_pipe,
    ),
    util_run("SysWaitEventSem", crate::builtin::rexxutil::wait_sem),
    #[cfg(windows)]
    util_run(
        "SysWinDecryptFile",
        crate::builtin::rexxutil::windows_files::set_encrypted::<false>,
    ),
    #[cfg(windows)]
    util_run(
        "SysWinEncryptFile",
        crate::builtin::rexxutil::windows_files::set_encrypted::<true>,
    ),
    #[cfg(windows)]
    util_run(
        "SysWinGetDefaultPrinter",
        crate::builtin::rexxutil::windows_printers::get_default,
    ),
    #[cfg(windows)]
    util_run(
        "SysWinGetPrinters",
        crate::builtin::rexxutil::windows_printers::list,
    ),
    #[cfg(windows)]
    util_run(
        "SysWinSetDefaultPrinter",
        crate::builtin::rexxutil::windows_printers::set_default,
    ),
    #[cfg(windows)]
    util_run("SysWinVer", crate::builtin::rexxutil::version),
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

/// `package`'s row `name` names, found as a routine table entry is: by exact
/// spelling, then without regard to case.
pub(crate) fn package_routine(package: Package, name: &[u8]) -> Option<&'static InternalRoutine> {
    package_rows(package)
        .find(|row| row.name.as_bytes() == name)
        .or_else(|| {
            package_rows(package).find(|row| row.name.as_bytes().eq_ignore_ascii_case(name))
        })
}

/// The row `name` names, matched caselessly.
pub(crate) fn lookup(name: &[u8]) -> Option<&'static InternalRoutine> {
    INTERNAL_ROUTINES
        .iter()
        .find(|row| row.name.as_bytes().eq_ignore_ascii_case(name))
}
