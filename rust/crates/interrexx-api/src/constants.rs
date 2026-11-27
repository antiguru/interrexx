//! The constants the headers in `include/` define, by their C names.
//!
//! Each value is the one the interpreter uses: where Rust code already names
//! the value, the table takes it from there, so the header and the code that
//! reads what an extension passes cannot disagree. The header generator
//! (`header.rs`) writes these tables, and the ABI dump (`tests/abi_dump.rs`)
//! reads every value back through a C and a C++ compiler.

use std::borrow::Cow;
use std::ffi::c_int;

use interrexx_cast::{i64_from_i32, i64_from_isize, i64_from_u32, u32_from_u16, u64_from_usize};

use crate::layout::{
    CALL_INTERFACE_VERSION, CSTRING, EXIT_INTERFACE_VERSION, INSTANCE_INTERFACE_VERSION,
    METHOD_INTERFACE_VERSION, POINTER, REDIRECT_INTERFACE_VERSION, RexxArrayObject,
    RexxClassObject, RexxMutableBufferObject, RexxObjectPtr, RexxStemObject, RexxStringObject,
    RexxVariableReferenceObject, THREAD_INTERFACE_VERSION, intptr_t, logical_t, stringsize_t,
    uintptr_t, wholenumber_t,
};
use crate::load::{
    CURRENT_INTERPRETER_VERSION, CURRENT_LANGUAGE_LEVEL, DIRECT_COMMAND_ENVIRONMENT,
    METHOD_TYPED_STYLE, PACKAGE_API_NO, REDIRECTING_COMMAND_ENVIRONMENT, ROUTINE_CLASSIC_STYLE,
    ROUTINE_TYPED_STYLE, exit_code,
};
use crate::values::{
    ARGUMENT_EXISTS, ARGUMENT_TERMINATOR, OPTIONAL_ARGUMENT, SPECIAL_ARGUMENT, rows,
};

/// How the header writes a constant's value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    /// A decimal literal.
    Decimal,
    /// A hexadecimal literal, of at least `digits` digits.
    Hex { digits: usize },
    /// `REXX_OPTIONAL_ARGUMENT` ORed with `REXX_VALUE_<name>`.
    Optional(&'static str),
}

/// One `#define` of a number.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CConstant {
    /// Its C name.
    pub name: Cow<'static, str>,
    /// Its value.
    pub value: i64,
    /// How the header writes the value.
    pub form: Form,
}

impl CConstant {
    /// The text the header defines the name as.
    #[must_use]
    pub fn text(&self) -> String {
        match self.form {
            Form::Decimal if self.value < 0 => format!("({})", self.value),
            Form::Decimal => self.value.to_string(),
            Form::Hex { digits } => format!("0x{:0digits$x}", self.value),
            Form::Optional(name) => format!("(REXX_OPTIONAL_ARGUMENT | REXX_VALUE_{name})"),
        }
    }
}

const fn decimal(name: &'static str, value: i64) -> CConstant {
    CConstant {
        name: Cow::Borrowed(name),
        value,
        form: Form::Decimal,
    }
}

const fn hex(name: &'static str, value: i64) -> CConstant {
    CConstant {
        name: Cow::Borrowed(name),
        value,
        form: Form::Hex { digits: 2 },
    }
}

const fn version(name: &'static str, value: i64) -> CConstant {
    CConstant {
        name: Cow::Borrowed(name),
        value,
        form: Form::Hex { digits: 8 },
    }
}

/// The classic API's codes, in `rexxapidefs.h`.
pub const CLASSIC_CONSTANTS: &[CConstant] = &[
    // How `RexxStart` calls a program.
    decimal("RXCOMMAND", 0),
    decimal("RXSUBROUTINE", 1),
    decimal("RXFUNCTION", 2),
    decimal("RXMETHOD", 3),
    decimal("RXSCRIPT", 4),
    // The exit function codes, the end of an exit list and the number of codes.
    decimal("RXENDLST", i64_from_i32(exit_code::END)),
    decimal("RXFNC", i64_from_i32(exit_code::FNC)),
    decimal("RXCMD", i64_from_i32(exit_code::CMD)),
    decimal("RXMSQ", i64_from_i32(exit_code::MSQ)),
    decimal("RXSIO", i64_from_i32(exit_code::SIO)),
    decimal("RXHLT", i64_from_i32(exit_code::HLT)),
    decimal("RXTRC", i64_from_i32(exit_code::TRC)),
    decimal("RXINI", i64_from_i32(exit_code::INI)),
    decimal("RXTER", i64_from_i32(exit_code::TER)),
    decimal("RXEXF", i64_from_i32(exit_code::EXF)),
    decimal("RXNOVAL", i64_from_i32(exit_code::NOVAL)),
    decimal("RXVALUE", i64_from_i32(exit_code::VALUE)),
    decimal("RXOFNC", i64_from_i32(exit_code::OFNC)),
    decimal(
        "RXNOOFEXITS",
        u64_from_usize(exit_code::COUNT).cast_signed(),
    ),
    // The exit subfunction codes.
    decimal("RXFNCCAL", 1),
    decimal("RXCMDHST", 1),
    decimal("RXMSQPLL", 1),
    decimal("RXMSQPSH", 2),
    decimal("RXMSQSIZ", 3),
    decimal("RXMSQNAM", 20),
    decimal("RXSIOSAY", 1),
    decimal("RXSIOTRC", 2),
    decimal("RXSIOTRD", 3),
    decimal("RXSIODTR", 4),
    decimal("RXSIOTLL", 5),
    decimal("RXHLTCLR", 1),
    decimal("RXHLTTST", 2),
    decimal("RXTRCTST", 1),
    decimal("RXINIEXT", 1),
    decimal("RXTEREXT", 1),
    decimal("RXEXFCAL", 1),
    decimal("RXNOVALCALL", 1),
    decimal("RXVALUECALL", 1),
    decimal("RXOFNCCAL", 1),
    // What an exit handler answers.
    decimal("RXEXIT_HANDLED", 0),
    decimal("RXEXIT_NOT_HANDLED", 1),
    decimal("RXEXIT_RAISE_ERROR", -1),
    // The shared-variable request codes and the flags of its answer.
    hex("RXSHV_OK", 0x00),
    hex("RXSHV_SET", 0x00),
    hex("RXSHV_FETCH", 0x01),
    hex("RXSHV_NEWV", 0x01),
    hex("RXSHV_DROPV", 0x02),
    hex("RXSHV_LVAR", 0x02),
    hex("RXSHV_SYSET", 0x03),
    hex("RXSHV_SYFET", 0x04),
    hex("RXSHV_TRUNC", 0x04),
    hex("RXSHV_SYDRO", 0x05),
    hex("RXSHV_NEXTV", 0x06),
    hex("RXSHV_PRIV", 0x07),
    hex("RXSHV_BADN", 0x08),
    hex("RXSHV_MEMFL", 0x10),
    hex("RXSHV_BADF", 0x80),
    decimal("RXSHV_NOAVL", 144),
    // The subcommand registration flags and return codes.
    hex("RXSUBCOM_DROPPABLE", 0x00),
    decimal("RXSUBCOM_OK", 0),
    hex("RXSUBCOM_ERROR", 0x01),
    hex("RXSUBCOM_ISREG", 0x01),
    hex("RXSUBCOM_NONDROP", 0x01),
    hex("RXSUBCOM_FAILURE", 0x02),
    decimal("RXSUBCOM_DUP", 10),
    decimal("RXSUBCOM_MAXREG", 20),
    decimal("RXSUBCOM_NOTREG", 30),
    decimal("RXSUBCOM_NOCANDROP", 40),
    decimal("RXSUBCOM_LOADERR", 50),
    decimal("RXSUBCOM_NOPROC", 127),
    decimal("RXSUBCOM_BADENTRY", 1001),
    decimal("RXSUBCOM_NOEMEM", 1002),
    decimal("RXSUBCOM_BADTYPE", 1003),
    decimal("RXSUBCOM_NOTINIT", 1004),
    // The exit registration flags and return codes.
    hex("RXEXIT_DROPPABLE", 0x00),
    decimal("RXEXIT_OK", 0),
    hex("RXEXIT_ERROR", 0x01),
    hex("RXEXIT_ISREG", 0x01),
    hex("RXEXIT_NONDROP", 0x01),
    hex("RXEXIT_FAILURE", 0x02),
    decimal("RXEXIT_DUP", 10),
    decimal("RXEXIT_MAXREG", 20),
    decimal("RXEXIT_NOTREG", 30),
    decimal("RXEXIT_NOCANDROP", 40),
    decimal("RXEXIT_LOADERR", 50),
    decimal("RXEXIT_NOPROC", 127),
    decimal("RXEXIT_BADENTRY", 1001),
    decimal("RXEXIT_NOEMEM", 1002),
    decimal("RXEXIT_BADTYPE", 1003),
    decimal("RXEXIT_NOTINIT", 1004),
    // The function registration kinds and return codes.
    decimal("RXFUNC_OK", 0),
    decimal("RXFUNC_DYNALINK", 1),
    decimal("RXFUNC_CALLENTRY", 2),
    decimal("RXFUNC_DEFINED", 10),
    decimal("RXFUNC_NOMEM", 20),
    decimal("RXFUNC_NOTREG", 30),
    decimal("RXFUNC_MODNOTFND", 40),
    decimal("RXFUNC_ENTNOTFND", 50),
    decimal("RXFUNC_BADTYPE", 70),
    decimal("RXFUNC_NOEMEM", 1002),
    decimal("RXFUNC_NOTINIT", 1004),
    // The macrospace search orders and return codes.
    decimal("RXMACRO_OK", 0),
    decimal("RXMACRO_NO_STORAGE", 1),
    decimal("RXMACRO_SEARCH_BEFORE", 1),
    decimal("RXMACRO_NOT_FOUND", 2),
    decimal("RXMACRO_SEARCH_AFTER", 2),
    decimal("RXMACRO_EXTENSION_REQUIRED", 3),
    decimal("RXMACRO_ALREADY_EXISTS", 4),
    decimal("RXMACRO_FILE_ERROR", 5),
    decimal("RXMACRO_SIGNATURE_ERROR", 6),
    decimal("RXMACRO_SOURCE_NOT_FOUND", 7),
    decimal("RXMACRO_INVALID_POSITION", 8),
    decimal("RXMACRO_NOT_INIT", 1004),
    // The queue orders, wait flags and return codes.
    decimal("RXQUEUE_FIFO", 0),
    decimal("RXQUEUE_NOWAIT", 0),
    decimal("RXQUEUE_OK", 0),
    decimal("RXQUEUE_LIFO", 1),
    decimal("RXQUEUE_STORAGE", 1),
    decimal("RXQUEUE_WAIT", 1),
    decimal("RXQUEUE_SIZE", 2),
    decimal("RXQUEUE_DUP", 3),
    decimal("RXQUEUE_BADQNAME", 5),
    decimal("RXQUEUE_PRIORITY", 6),
    decimal("RXQUEUE_BADWAITFLAG", 7),
    decimal("RXQUEUE_EMPTY", 8),
    decimal("RXQUEUE_NOTREG", 9),
    decimal("RXQUEUE_ACCESS", 10),
    decimal("RXQUEUE_MAXREG", 11),
    decimal("RXQUEUE_MEMFAIL", 1002),
    decimal("RXQUEUE_NOEMEM", 1002),
    decimal("RXQUEUE_NOTINIT", 1004),
    // The return codes that report on the API service rather than on the request.
    decimal("RXAPI_OK", 0),
    decimal("RXAPI_MEMFAIL", 1002),
    decimal("RXAPI_NORXAPI", 1004),
    // What `RexxSetHalt`, `RexxSetTrace` and `RexxResetTrace` answer.
    decimal("RXARI_OK", 0),
    decimal("RXARI_NOT_FOUND", 1),
    decimal("RXARI_PROCESSING_ERROR", 2),
    // The size of the result buffer `RexxStart` and a classic handler are handed.
    decimal("RXAUTOBUFLEN", 256),
];

/// The classic API's limits and the codes `RexxCallBack` answers, in `rexx.h`.
pub const CLASSIC_LIMITS: &[CConstant] = &[
    // The longest queue name.
    decimal("MAX_QUEUE_NAME_LENGTH", 250),
    // What `RexxCallBack` answers.
    decimal("RX_CB_OK", 0),
    decimal("RX_CB_BADP", 1),
    decimal("RX_CB_NOTSTARTED", 2),
    decimal("RX_CB_TOOMANYP", 3),
    decimal("RX_CB_BADN", 8),
];

/// The interface versions and interpreter and language levels, in
/// `oorexxapi.h`. A `_<x>_<y>_<z>` name is the version a release of that
/// number carried; the unsuffixed name is the one this interpreter's tables
/// carry.
pub const API_VERSIONS: &[CConstant] = &[
    decimal("INSTANCE_INTERFACE_VERSION_4_0_0", 100),
    decimal("INSTANCE_INTERFACE_VERSION_5_0_0", 101),
    decimal("INSTANCE_INTERFACE_VERSION_5_3_0", 102),
    decimal(
        "INSTANCE_INTERFACE_VERSION",
        i64_from_isize(INSTANCE_INTERFACE_VERSION),
    ),
    decimal("THREAD_INTERFACE_VERSION_4_0_0", 100),
    decimal("THREAD_INTERFACE_VERSION_4_1_1", 101),
    decimal("THREAD_INTERFACE_VERSION_4_2_0", 102),
    decimal("THREAD_INTERFACE_VERSION_5_0_0", 103),
    decimal("THREAD_INTERFACE_VERSION_5_3_0", 104),
    decimal(
        "THREAD_INTERFACE_VERSION",
        i64_from_isize(THREAD_INTERFACE_VERSION),
    ),
    decimal("METHOD_INTERFACE_VERSION_4_0_0", 100),
    decimal("METHOD_INTERFACE_VERSION_4_2_0", 101),
    decimal("METHOD_INTERFACE_VERSION_5_0_0", 102),
    decimal(
        "METHOD_INTERFACE_VERSION",
        i64_from_isize(METHOD_INTERFACE_VERSION),
    ),
    decimal("CALL_INTERFACE_VERSION_4_0_0", 100),
    decimal("CALL_INTERFACE_VERSION_5_0_0", 101),
    decimal(
        "CALL_INTERFACE_VERSION",
        i64_from_isize(CALL_INTERFACE_VERSION),
    ),
    decimal("EXIT_INTERFACE_VERSION_4_0_0", 100),
    decimal("EXIT_INTERFACE_VERSION_5_0_0", 101),
    decimal(
        "EXIT_INTERFACE_VERSION",
        i64_from_isize(EXIT_INTERFACE_VERSION),
    ),
    decimal("REDIRECT_INTERFACE_VERSION_5_0_0", 100),
    decimal(
        "REDIRECT_INTERFACE_VERSION",
        i64_from_isize(REDIRECT_INTERFACE_VERSION),
    ),
    version("REXX_INTERPRETER_4_0_0", 0x0004_0000),
    version("REXX_INTERPRETER_4_0_1", 0x0004_0001),
    version("REXX_INTERPRETER_4_1_0", 0x0004_0100),
    version("REXX_INTERPRETER_4_1_1", 0x0004_0101),
    version("REXX_INTERPRETER_4_1_2", 0x0004_0102),
    version("REXX_INTERPRETER_4_1_3", 0x0004_0103),
    version("REXX_INTERPRETER_4_2_0", 0x0004_0200),
    version("REXX_INTERPRETER_4_3_0", 0x0004_0300),
    version("REXX_INTERPRETER_5_0_0", 0x0005_0000),
    version("REXX_INTERPRETER_5_1_0", 0x0005_0100),
    version("REXX_INTERPRETER_5_2_0", 0x0005_0200),
    version("REXX_INTERPRETER_5_3_0", 0x0005_0300),
    version(
        "REXX_CURRENT_INTERPRETER_VERSION",
        i64_from_i32(CURRENT_INTERPRETER_VERSION),
    ),
    version("REXX_LANGUAGE_6_03", 0x0603),
    version("REXX_LANGUAGE_6_04", 0x0604),
    version("REXX_LANGUAGE_6_05", 0x0605),
    version("REXX_LANGUAGE_6_06", 0x0606),
    version("REXX_LANGUAGE_6_07", 0x0607),
    version(
        "REXX_CURRENT_LANGUAGE_LEVEL",
        u64_from_usize(CURRENT_LANGUAGE_LEVEL).cast_signed(),
    ),
];

/// Second names of value codes, each a platform's name for the C type of
/// another: the alias and the name it stands for.
pub const VALUE_ALIASES: &[(&str, &str)] = &[
    ("__int32_t", "int32_t"),
    ("__int64_t", "int64_t"),
    ("__uint32_t", "uint32_t"),
    ("__uint64_t", "uint64_t"),
    ("__uintptr_t", "uintptr_t"),
];

/// The value codes that have an optional form, `REXX_VALUE_OPTIONAL_<name>`.
/// A code that takes no argument, and the variable reference code, have none.
pub const OPTIONAL_VALUES: &[&str] = &[
    "RexxObjectPtr",
    "int",
    "wholenumber_t",
    "double",
    "CSTRING",
    "POINTER",
    "RexxStringObject",
    "stringsize_t",
    "float",
    "int8_t",
    "int16_t",
    "int32_t",
    "int64_t",
    "uint8_t",
    "uint16_t",
    "uint32_t",
    "uint64_t",
    "intptr_t",
    "uintptr_t",
    "logical_t",
    "RexxArrayObject",
    "RexxStemObject",
    "size_t",
    "ssize_t",
    "POINTERSTRING",
    "RexxClassObject",
    "RexxMutableBufferObject",
    "positive_wholenumber_t",
    "nonnegative_wholenumber_t",
];

/// The value codes, in `oorexxapi.h`: the optional bit, `REXX_VALUE_<name>`
/// for every row of the conversion table (`values.rs`) and every alias, and
/// the optional forms.
///
/// # Panics
/// If an alias or an optional form names a code the conversion table lacks.
#[must_use]
pub fn value_codes() -> Vec<CConstant> {
    let code_of = |name: &str| -> i64 {
        rows()
            .find(|(_, row)| *row == name)
            .map(|(code, _)| i64::from(code))
            .unwrap_or_else(|| panic!("the conversion table has no code {name}"))
    };
    let named = |name: String, value: i64| CConstant {
        name: Cow::Owned(name),
        value,
        form: Form::Decimal,
    };
    let mut out = vec![CConstant {
        name: Cow::Borrowed("REXX_OPTIONAL_ARGUMENT"),
        value: i64::from(OPTIONAL_ARGUMENT),
        form: Form::Hex { digits: 4 },
    }];
    for (code, name) in rows() {
        out.push(named(format!("REXX_VALUE_{name}"), i64::from(code)));
    }
    for (alias, name) in VALUE_ALIASES {
        out.push(named(format!("REXX_VALUE_{alias}"), code_of(name)));
    }
    for name in OPTIONAL_VALUES {
        out.push(CConstant {
            name: Cow::Owned(format!("REXX_VALUE_OPTIONAL_{name}")),
            value: i64::from(OPTIONAL_ARGUMENT) | code_of(name),
            form: Form::Optional(name),
        });
    }
    out
}

/// The names of the options of a new interpreter instance, each the C name
/// and the string it stands for, in `oorexxapi.h`.
pub const OPTION_NAMES: &[(&str, &str)] = &[
    ("INITIAL_ADDRESS_ENVIRONMENT", "InitialAddress"),
    ("APPLICATION_DATA", "ApplicationData"),
    ("EXTERNAL_CALL_PATH", "ExternalCallPath"),
    ("EXTERNAL_CALL_EXTENSIONS", "ExternalCallPathExt"),
    ("REGISTERED_EXITS", "RegisteredExits"),
    ("DIRECT_EXITS", "DirectExits"),
    ("REGISTERED_ENVIRONMENTS", "RegisteredEnvironments"),
    ("DIRECT_ENVIRONMENTS", "DirectEnvironments"),
    ("REDIRECTING_ENVIRONMENTS", "RedirectingEnvironments"),
    ("LOAD_REQUIRED_LIBRARY", "LoadRequiredLibrary"),
    ("REGISTER_LIBRARY", "RegisterLibrary"),
];

/// The kinds of command environment `AddCommandEnvironment` adds, in
/// `oorexxapi.h`.
pub const ENVIRONMENT_KINDS: &[CConstant] = &[
    decimal(
        "DIRECT_COMMAND_ENVIRONMENT",
        i64_from_i32(DIRECT_COMMAND_ENVIRONMENT),
    ),
    decimal(
        "REDIRECTING_COMMAND_ENVIRONMENT",
        i64_from_i32(REDIRECTING_COMMAND_ENVIRONMENT),
    ),
];

/// The constants of a library's entry tables and of the argument descriptors its
/// stubs read, in `oorexxapi.h`.
pub const ENTRY_CONSTANTS: &[CConstant] = &[
    // The type word that ends a stub's signature, and the one a result of no
    // type carries.
    decimal(
        "REXX_ARGUMENT_TERMINATOR",
        i64_from_u32(u32_from_u16(ARGUMENT_TERMINATOR)),
    ),
    // The flag bits of an argument descriptor.
    hex(
        "ARGUMENT_EXISTS",
        i64_from_u32(u32_from_u16(ARGUMENT_EXISTS)),
    ),
    hex(
        "SPECIAL_ARGUMENT",
        i64_from_u32(u32_from_u16(SPECIAL_ARGUMENT)),
    ),
    // The styles of a routine row and of a method row.
    decimal("ROUTINE_TYPED_STYLE", i64_from_i32(ROUTINE_TYPED_STYLE)),
    decimal("ROUTINE_CLASSIC_STYLE", i64_from_i32(ROUTINE_CLASSIC_STYLE)),
    decimal("METHOD_TYPED_STYLE", i64_from_i32(METHOD_TYPED_STYLE)),
    // The `apiVersion` of a package entry.
    decimal("REXX_PACKAGE_API_NO", i64_from_i32(PACKAGE_API_NO)),
];

/// The names a routine or method declaration may give as a special argument's
/// type, and `BUFFER`, each with the tokens of the Rust type it names, in
/// `oorexxapi.h`. Each special argument's name is the type of the argument
/// its descriptor carries.
pub const ARGUMENT_TYPEDEFS: &[(&str, &str)] = &[
    ("OSELF", "RexxObjectPtr"),
    ("CSELF", "POINTER"),
    ("BUFFER", "POINTER"),
    ("ARGLIST", "RexxArrayObject"),
    ("SCOPE", "RexxObjectPtr"),
    ("SUPER", "RexxClassObject"),
    ("NAME", "CSTRING"),
];

/// Defines [`ARGUMENT_TYPES`] and [`argument_type_names`] from each value
/// code's name and the Rust type of the argument an entry receives for it.
macro_rules! argument_types {
    ($($name:ident: $ty:ty,)*) => {
        /// Each value code's name and the tokens of the Rust type of the
        /// argument an entry receives for it, `ARGUMENT_TYPE_<name>` in
        /// `oorexxapi.h`.
        pub const ARGUMENT_TYPES: &[(&str, &str)] = &[$((stringify!($name), stringify!($ty))),*];

        /// Each value code's name and the Rust type of its argument, in the
        /// order of [`ARGUMENT_TYPES`].
        #[must_use]
        pub fn argument_type_names() -> Vec<(&'static str, &'static str)> {
            vec![$((stringify!($name), std::any::type_name::<$ty>())),*]
        }
    };
}

argument_types! {
    ARGLIST: RexxArrayObject,
    NAME: CSTRING,
    SCOPE: RexxObjectPtr,
    CSELF: POINTER,
    OSELF: RexxObjectPtr,
    SUPER: RexxClassObject,
    RexxObjectPtr: RexxObjectPtr,
    int: c_int,
    wholenumber_t: wholenumber_t,
    double: f64,
    CSTRING: CSTRING,
    POINTER: POINTER,
    RexxStringObject: RexxStringObject,
    stringsize_t: stringsize_t,
    float: f32,
    int8_t: i8,
    int16_t: i16,
    int32_t: i32,
    int64_t: i64,
    __int32_t: i32,
    __int64_t: i64,
    uint8_t: u8,
    uint16_t: u16,
    uint32_t: u32,
    uint64_t: u64,
    __uint32_t: u32,
    __uint64_t: u64,
    intptr_t: intptr_t,
    uintptr_t: uintptr_t,
    __uintptr_t: uintptr_t,
    logical_t: logical_t,
    RexxArrayObject: RexxArrayObject,
    RexxStemObject: RexxStemObject,
    size_t: usize,
    ssize_t: isize,
    POINTERSTRING: POINTER,
    RexxClassObject: RexxClassObject,
    RexxMutableBufferObject: RexxMutableBufferObject,
    positive_wholenumber_t: wholenumber_t,
    nonnegative_wholenumber_t: wholenumber_t,
    RexxVariableReferenceObject: RexxVariableReferenceObject,
}

/// The value codes whose argument has an optional form,
/// `ARGUMENT_TYPE_OPTIONAL_<name>`, of the same type as the required one:
/// those of [`OPTIONAL_VALUES`] and the variable reference.
#[must_use]
pub fn optional_argument_types() -> Vec<&'static str> {
    let mut out = OPTIONAL_VALUES.to_vec();
    out.push("RexxVariableReferenceObject");
    out
}

/// The value of the error code `Rexx_<symbol>` for message `major.sub`, which
/// `oorexxerrors.h` defines for every message whose major code is below 100.
#[must_use]
pub fn error_code(major: u16, sub: u16) -> i64 {
    i64::from(major) * 1000 + i64::from(sub)
}

/// Whether `oorexxerrors.h` defines a code for a message of major code
/// `major`.
#[must_use]
pub fn has_error_code(major: u16) -> bool {
    major < 100
}
