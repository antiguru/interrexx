//! The functions `librexx` and `librexxapi` export, as the headers in
//! `include/` declare them.
//!
//! Each export's Rust type here is the type of its definition: the defining
//! crates (`interrexx-capi`, `interrexx-capi-rxapi`) coerce every definition to
//! it at compile time, so a definition whose signature drifts from its
//! declaration does not build. The header generator (`header.rs`) spells each
//! declaration from the same tokens, and the ABI dump (`tests/abi_dump.rs`)
//! measures the declared type of every export against the Rust type through a
//! C and a C++ compiler.

use std::ffi::{c_char, c_int, c_short, c_ushort, c_void};

use crate::layout::{
    CMember, CONSTRXSTRING, CSTRING, REXXDATETIME, REXXPFN, RXSTRING, RXSYSEXIT, RexxInstance_,
    RexxOption, RexxQueueTime, RexxReturnCode, RexxStemObject, RexxThreadContext_, SHVBLOCK,
    process_id_t, thread_id_t,
};

/// Where the headers declare an export.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Declared {
    /// In this file of `include/`.
    In(&'static str),
    /// In each platform's `rexxplatformapis.h`: its signature names a
    /// platform type.
    Platform,
    /// Nowhere: the library exports it, and no header declares it.
    Nowhere,
}

/// One exported function.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Export {
    /// Its name.
    pub name: &'static str,
    /// Where the headers declare it.
    pub declared: Declared,
    /// The upper-case macro that names it too, where there is one.
    pub alias: Option<&'static str>,
    /// The typedef of a pointer to it, where there is one.
    pub pointer: Option<&'static str>,
    /// Its declaration's parts: the tokens of its result's Rust type (`None`
    /// for none) and of each parameter's.
    pub declaration: CMember,
}

/// Defines [`signatures`], [`EXPORTS`] and [`export_types`] from each
/// export's name, where it is declared, its alias and pointer typedef, and its
/// Rust type.
macro_rules! exports {
    ($(
        $(#[$meta:meta])*
        $name:ident in $where:tt $(alias $alias:literal)? $(pointer $pointer:ident)?
            = extern $abi:literal fn($($arg:ty),* $(,)?) $(-> $ret:ty)?;
    )*) => {
        /// The Rust type of each export, under the export's name.
        pub mod signatures {
            use super::*;
            $( $(#[$meta])* pub type $name = unsafe extern $abi fn($($arg),*) $(-> $ret)?; )*
        }

        /// Every exported function, in the order the headers declare them.
        pub const EXPORTS: &[Export] = &[$(Export {
            name: stringify!($name),
            declared: exports!(@where $where),
            alias: exports!(@option $($alias)?),
            pointer: exports!(@option $(stringify!($pointer))?),
            declaration: CMember::Function {
                returns: exports!(@option $(stringify!($ret))?),
                parameters: &[$(stringify!($arg)),*],
            },
        }),*];

        /// Each export's name and the Rust type it has, in the order of
        /// [`EXPORTS`].
        #[must_use]
        pub fn export_types() -> Vec<(&'static str, &'static str)> {
            vec![$((stringify!($name), std::any::type_name::<signatures::$name>())),*]
        }
    };
    (@where platform) => { Declared::Platform };
    (@where undeclared) => { Declared::Nowhere };
    (@where $file:literal) => { Declared::In($file) };
    (@option) => { None };
    (@option $value:expr) => { Some($value) };
}

exports! {
    /// Runs a program in an interpreter of its own.
    RexxStart in "rexx.h" alias "REXXSTART" pointer PFNREXXSTART = extern "C" fn(
        usize,
        *mut CONSTRXSTRING,
        CSTRING,
        *mut RXSTRING,
        CSTRING,
        c_int,
        *mut RXSYSEXIT,
        *mut c_short,
        *mut RXSTRING,
    ) -> c_int;
    /// Waits for the programs `RexxStart` runs to end.
    RexxWaitForTermination in "rexx.h" alias "REXXWAITFORTERMINATION"
        pointer PFNREXXWAITFORTERMINATION = extern "C" fn();
    /// Whether the programs `RexxStart` runs have ended.
    RexxDidRexxTerminate in "rexx.h" alias "REXXDIDREXXTERMINATE"
        pointer PFNREXXDIDREXXTERMINATE = extern "C" fn() -> RexxReturnCode;
    /// Ends the API service of this process.
    RexxShutDownAPI in "rexx.h" alias "REXXSHUTDOWNAPI"
        pointer PFNREXXSHUTDOWNAPI = extern "C" fn() -> RexxReturnCode;
    /// Translates a program file into an image file.
    RexxTranslateProgram in "rexx.h" alias "REXXTRANSLATEPROGRAM"
        pointer PFNREXXTRANSLATEPROGRAM = extern "C" fn(CSTRING, CSTRING, *mut RXSYSEXIT) -> RexxReturnCode;
    /// Translates a program held in storage into an image held in storage.
    RexxTranslateInstoreProgram in "rexx.h" alias "REXXTRANSLATEINSTOREPROGRAM"
        pointer PFNREXXTRANSLATEINSTOREPROGRAM = extern "C" fn(CSTRING, *mut CONSTRXSTRING, *mut RXSTRING) -> RexxReturnCode;
    /// The interpreter's version string, in a block `RexxFreeMemory` releases.
    RexxGetVersionInformation in "rexx.h" alias "REXXGETVERSIONINFORMATON"
        pointer PFNGETVERSIONINFORMATION = extern "C" fn() -> *mut c_char;
    /// Registers a subcommand handler by library and procedure name.
    RexxRegisterSubcomDll in "rexx.h" alias "REXXREGISTERSUBCOMDLL"
        pointer PFNREXXREGISTERSUBCOMDLL = extern "C" fn(CSTRING, CSTRING, CSTRING, CSTRING, usize) -> RexxReturnCode;
    /// Registers a subcommand handler by address.
    RexxRegisterSubcomExe in "rexx.h" alias "REXXREGISTERSUBCOMEXE"
        pointer PFNREXXREGISTERSUBCOMEXE = extern "C" fn(CSTRING, REXXPFN, CSTRING) -> RexxReturnCode;
    /// Whether a subcommand handler is registered, and its user area.
    RexxQuerySubcom in "rexx.h" alias "REXXQUERYSUBCOM"
        pointer PFNREXXQUERYSUBCOM = extern "C" fn(CSTRING, CSTRING, *mut c_ushort, *mut c_char) -> RexxReturnCode;
    /// Drops a subcommand handler's registration.
    RexxDeregisterSubcom in "rexx.h" alias "REXXDEREGISTERSUBCOM"
        pointer PFNREXXDEREGISTERSUBCOM = extern "C" fn(CSTRING, CSTRING) -> RexxReturnCode;
    /// Reads and writes the variables of the running program.
    RexxVariablePool in "rexx.h" alias "REXXVARIABLEPOOL"
        pointer PFNREXXVARIABLEPOOL = extern "C" fn(*mut SHVBLOCK) -> RexxReturnCode;
    /// Calls a label of the program that called the running external function.
    RexxCallBack in "rexx.h" alias "REXXCALLBACK" pointer PFNREXXCALLBACK = extern "C-unwind" fn(
        CSTRING,
        usize,
        *mut CONSTRXSTRING,
        *mut c_short,
        *mut RXSTRING,
    ) -> RexxReturnCode;
    /// Registers an external function by library and procedure name.
    RexxRegisterFunctionDll in "rexx.h" alias "REXXREGISTERFUNCTIONDLL"
        pointer PFNREXXREGISTERFUNCTIONDLL = extern "C" fn(CSTRING, CSTRING, CSTRING) -> RexxReturnCode;
    /// Registers an external function by address.
    RexxRegisterFunctionExe in "rexx.h" alias "REXXREGISTERFUNCTIONEXE"
        pointer PFNREXXREGISTERFUNCTIONEXE = extern "C" fn(CSTRING, REXXPFN) -> RexxReturnCode;
    /// Drops an external function's registration.
    RexxDeregisterFunction in "rexx.h" alias "REXXDEREGISTERFUNCTION"
        pointer PFNREXXDEREGISTERFUNCTION = extern "C" fn(CSTRING) -> RexxReturnCode;
    /// Whether an external function is registered.
    RexxQueryFunction in "rexx.h" alias "REXXQUERYFUNCTION"
        pointer PFNREXXQUERYFUNCTION = extern "C" fn(CSTRING) -> RexxReturnCode;
    /// Registers an exit handler by library and procedure name.
    RexxRegisterExitDll in "rexx.h" alias "REXXREGISTEREXITDLL"
        pointer PFNREXXREGISTEREXITDLL = extern "C" fn(CSTRING, CSTRING, CSTRING, CSTRING, usize) -> RexxReturnCode;
    /// Registers an exit handler by address.
    RexxRegisterExitExe in "rexx.h" alias "REXXREGISTEREXITEXE"
        pointer PFNREXXREGISTEREXITEXE = extern "C" fn(CSTRING, REXXPFN, CSTRING) -> RexxReturnCode;
    /// Drops an exit handler's registration.
    RexxDeregisterExit in "rexx.h" alias "REXXDEREGISTEREXIT"
        pointer PFNREXXDEREGISTEREXIT = extern "C" fn(CSTRING, CSTRING) -> RexxReturnCode;
    /// Whether an exit handler is registered, and its user area.
    RexxQueryExit in "rexx.h" alias "REXXQUERYEXIT"
        pointer PFNREXXQUERYEXIT = extern "C" fn(CSTRING, CSTRING, *mut c_ushort, *mut c_char) -> RexxReturnCode;
    /// Asks the program a thread runs to halt.
    RexxSetHalt in "rexx.h" alias "REXXSETHALT"
        pointer PFNREXXSETHALT = extern "C" fn(process_id_t, thread_id_t) -> RexxReturnCode;
    /// Turns on the external trace of the program a thread runs.
    RexxSetTrace in "rexx.h" alias "REXXSETTRACE"
        pointer PFNREXXSETTRACE = extern "C" fn(process_id_t, thread_id_t) -> RexxReturnCode;
    /// Turns off the external trace of the program a thread runs.
    RexxResetTrace in "rexx.h" alias "REXXRESETTRACE"
        pointer PFNREXXRESETTRACE = extern "C" fn(process_id_t, thread_id_t) -> RexxReturnCode;
    /// Adds a function from a file to the macrospace.
    RexxAddMacro in "rexx.h" alias "REXXADDMACRO"
        pointer PFNREXXADDMACRO = extern "C" fn(CSTRING, CSTRING, usize) -> RexxReturnCode;
    /// Drops a function from the macrospace.
    RexxDropMacro in "rexx.h" alias "REXXDROPMACRO"
        pointer PFNREXXDROPMACRO = extern "C" fn(CSTRING) -> RexxReturnCode;
    /// Saves the macrospace, or the functions named, to a file.
    RexxSaveMacroSpace in "rexx.h" alias "REXXSAVEMACROSPACE"
        pointer PFNREXXSAVEMACROSPACE = extern "C" fn(usize, *mut CSTRING, CSTRING) -> RexxReturnCode;
    /// Loads the macrospace, or the functions named, from a file.
    RexxLoadMacroSpace in "rexx.h" alias "REXXLOADMACROSPACE"
        pointer PFNREXXLOADMACROSPACE = extern "C" fn(usize, *mut CSTRING, CSTRING) -> RexxReturnCode;
    /// Whether the macrospace holds a function, and its search order.
    RexxQueryMacro in "rexx.h" alias "REXXQUERYMACRO"
        pointer PFNREXXQUERYMACRO = extern "C" fn(CSTRING, *mut c_ushort) -> RexxReturnCode;
    /// Changes the search order of a macrospace function.
    RexxReorderMacro in "rexx.h" alias "REXXREORDERMACRO"
        pointer PFNREXXREORDERMACRO = extern "C" fn(CSTRING, usize) -> RexxReturnCode;
    /// Empties the macrospace.
    RexxClearMacroSpace in "rexx.h" alias "REXXCLEARMACROSPACE"
        pointer PFNREXXCLEARMACROSPACE = extern "C" fn() -> RexxReturnCode;
    /// Creates a named queue, or one the service names.
    RexxCreateQueue in "rexx.h" pointer PFNREXXCREATEQUEUE
        = extern "C" fn(*mut c_char, usize, CSTRING, *mut usize) -> RexxReturnCode;
    /// Opens a named queue, creating it where it does not exist.
    RexxOpenQueue in "rexx.h" pointer PFNREXXOPENQUEUE
        = extern "C" fn(CSTRING, *mut usize) -> RexxReturnCode;
    /// Whether a named queue exists.
    RexxQueueExists in "rexx.h" pointer PFNREXXQUEUEEXISTS = extern "C" fn(CSTRING) -> RexxReturnCode;
    /// Deletes a named queue.
    RexxDeleteQueue in "rexx.h" pointer PFNREXXDELETEQUEUE = extern "C" fn(CSTRING) -> RexxReturnCode;
    /// The number of lines a queue holds.
    RexxQueryQueue in "rexx.h" pointer PFNREXXQUERYQUEUE
        = extern "C" fn(CSTRING, *mut usize) -> RexxReturnCode;
    /// Adds a line to a queue.
    RexxAddQueue in "rexx.h" pointer PFNREXXADDQUEUE
        = extern "C" fn(CSTRING, *mut CONSTRXSTRING, usize) -> RexxReturnCode;
    /// Takes a line from a queue, with the time it was added.
    RexxPullFromQueue in "rexx.h" pointer PFNREXXPULLFROMQUEUE
        = extern "C" fn(CSTRING, *mut RXSTRING, *mut RexxQueueTime, usize) -> RexxReturnCode;
    /// Empties a queue.
    RexxClearQueue in "rexx.h" pointer PFNREXXCLEARQUEUE = extern "C" fn(CSTRING) -> RexxReturnCode;
    /// Takes a line from a queue, with the time it was added as this
    /// platform's date and time.
    RexxPullQueue in platform pointer PFNREXXPULLQUEUE
        = extern "C" fn(CSTRING, *mut RXSTRING, *mut REXXDATETIME, usize) -> RexxReturnCode;
    /// Allocates a block a result may be returned in.
    RexxAllocateMemory in "rexx.h" pointer PFNREXXALLOCATEMEMORY
        = extern "C" fn(usize) -> *mut c_void;
    /// Releases a block the interface allocated.
    RexxFreeMemory in "rexx.h" pointer PFNREXXFREEMEMORY
        = extern "C" fn(*mut c_void) -> RexxReturnCode;
    /// Creates an interpreter instance and attaches the calling thread to it.
    RexxCreateInterpreter in "oorexxapi.h" = extern "C" fn(
        *mut *mut RexxInstance_,
        *mut *mut RexxThreadContext_,
        *mut RexxOption,
    ) -> RexxReturnCode;
    /// Translates a program file into an image file, encoded or not.
    RexxCompileProgram in undeclared = extern "C" fn(CSTRING, CSTRING, *mut c_void, bool) -> RexxReturnCode;
    /// Writes the interpreter's image to a file.
    RexxCreateInterpreterImage in undeclared = extern "C" fn(CSTRING);
    /// The text of an error message, by its code.
    RexxGetErrorMessage in undeclared = extern "C" fn(c_int) -> CSTRING;
    /// The text of an error message, by its number.
    RexxGetErrorMessageByNumber in undeclared = extern "C" fn(c_int) -> CSTRING;
    /// Sorts the elements of a stem.
    RexxStemSort in undeclared = extern "C" fn(
        RexxStemObject,
        CSTRING,
        c_int,
        c_int,
        isize,
        isize,
        isize,
        isize,
    ) -> RexxReturnCode;
    /// Loads a subcommand handler's library.
    RexxLoadSubcom in undeclared = extern "C" fn(CSTRING, CSTRING) -> RexxReturnCode;
    /// Creates the session queue of this process.
    RexxCreateSessionQueue in undeclared = extern "C" fn() -> RexxReturnCode;
    /// Deletes the session queue of this process.
    RexxDeleteSessionQueue in undeclared = extern "C" fn() -> RexxReturnCode;
}

/// Defines each pointer typedef whose parameters are spelled other than its
/// function's, and beside them [`SPELLED_POINTERS`] and
/// [`spelled_pointer_types`].
macro_rules! spelled_pointers {
    ($($(#[$meta:meta])* $name:ident = fn($($arg:ty),* $(,)?) $(-> $ret:ty)?;)*) => {
        $( $(#[$meta])* pub type $name = unsafe extern "C" fn($($arg),*) $(-> $ret)?; )*

        /// Each pointer typedef whose parameters are spelled other than its
        /// function's, and its declaration's parts.
        pub const SPELLED_POINTERS: &[(&str, CMember)] = &[$((
            stringify!($name),
            CMember::Function {
                returns: exports!(@option $(stringify!($ret))?),
                parameters: &[$(stringify!($arg)),*],
            },
        )),*];

        /// Each of [`SPELLED_POINTERS`] and the Rust type it has.
        #[must_use]
        pub fn spelled_pointer_types() -> Vec<(&'static str, &'static str)> {
            vec![$((stringify!($name), std::any::type_name::<$name>())),*]
        }
    };
}

spelled_pointers! {
    /// `PFNREXXREGISTERSUBCOMDLL`: the user area is not constant.
    PFNREXXREGISTERSUBCOMDLL = fn(CSTRING, CSTRING, CSTRING, *mut c_char, usize) -> RexxReturnCode;
    /// `PFNREXXREGISTERSUBCOMEXE`: the user area is not constant.
    PFNREXXREGISTERSUBCOMEXE = fn(CSTRING, REXXPFN, *mut c_char) -> RexxReturnCode;
    /// `PFNREXXREGISTEREXITDLL`: the user area is not constant.
    PFNREXXREGISTEREXITDLL = fn(CSTRING, CSTRING, CSTRING, *mut c_char, usize) -> RexxReturnCode;
    /// `PFNREXXREGISTEREXITEXE`: the user area is not constant.
    PFNREXXREGISTEREXITEXE = fn(CSTRING, REXXPFN, *mut c_char) -> RexxReturnCode;
    /// `PFNREXXCREATEQUEUE`: the last parameter is a size, not a pointer to
    /// the flag the function sets.
    PFNREXXCREATEQUEUE = fn(*mut c_char, usize, CSTRING, usize) -> RexxReturnCode;
    /// `PFNREXXOPENQUEUE`: the last parameter is a size, not a pointer to the
    /// flag the function sets.
    PFNREXXOPENQUEUE = fn(CSTRING, usize) -> RexxReturnCode;
}

/// Each pointer typedef, its declaration's parts and the Rust type it has: a
/// pointer to its function, or the type [`SPELLED_POINTERS`] gives it.
#[must_use]
pub fn pointer_types() -> Vec<(&'static str, CMember, &'static str)> {
    let spelled = spelled_pointer_types();
    EXPORTS
        .iter()
        .zip(export_types())
        .filter_map(|(export, (_, rust))| {
            let pointer = export.pointer?;
            Some(
                match SPELLED_POINTERS
                    .iter()
                    .position(|(name, _)| *name == pointer)
                {
                    Some(at) => (pointer, SPELLED_POINTERS[at].1, spelled[at].1),
                    None => (pointer, export.declaration, rust),
                },
            )
        })
        .collect()
}
