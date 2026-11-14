// The `allow` is for the frozen header's own spellings (D5): the interface
// tables, the context structs, `RexxCondition` and `ValueDescriptor` carry the
// member names the header gives them. The entry structs moved here from
// `load.rs` and keep the Rust spellings they arrived with.
#![allow(non_camel_case_types, non_snake_case)]

//! The `#[repr(C)]` surface an extension sees, and the tables it calls
//! through.

use std::cell::Cell;
use std::ffi::{c_char, c_int, c_uchar, c_ushort, c_void};

/// Defines each scalar type the header declares by `typedef`, and beside them
/// each name with the tokens of the Rust type it stands for, from which the
/// header generator (`header.rs`) spells the C declaration.
macro_rules! c_typedefs {
    ($($(#[$meta:meta])* $name:ident = $ty:ty;)*) => {
        $( $(#[$meta])* pub type $name = $ty; )*

        /// Each scalar typedef, as the header names it, and the tokens of its
        /// Rust type, in declaration order.
        pub const C_TYPEDEFS: &[(&str, &str)] = &[$((stringify!($name), stringify!($ty))),*];
    };
}

c_typedefs! {
    /// `wholenumber_t`.
    wholenumber_t = isize;
    /// `stringsize_t`.
    stringsize_t = usize;
    /// `logical_t`.
    logical_t = usize;
    /// `CSTRING`.
    CSTRING = *const c_char;
    /// `POINTER`.
    POINTER = *mut c_void;
    /// `RexxReturnCode`.
    RexxReturnCode = c_int;
}

/// `REXXPFN`. Its C declaration differs by platform, so it is in the header
/// generator's per-platform table rather than in [`C_TYPEDEFS`].
pub type REXXPFN = *mut c_void;

/// `intptr_t`, a C type of its own name: a member that holds one is spelled
/// with it rather than as `ssize_t`.
pub type intptr_t = isize;
/// `uintptr_t`, a C type of its own name: a member that holds one is spelled
/// with it rather than as `size_t`.
pub type uintptr_t = usize;

/// Defines each opaque object handle: the struct an extension never sees
/// inside and the pointer to it that the extension holds. Beside them, each
/// handle's name with the handle whose struct its C++ struct derives from, so
/// that C++ passes a narrower handle where a wider one is expected.
macro_rules! handles {
    ($($Handle:ident = $Struct:ident $(: $Base:ident)?;)*) => {
        $(
            #[doc = concat!("`_", stringify!($Handle), "` and `", stringify!($Handle), "`.")]
            #[repr(C)]
            pub struct $Struct {
                _opaque: [u8; 0],
            }
            /// A handle to an object of that shape; opaque to an extension.
            pub type $Handle = *mut $Struct;
        )*

        /// Each handle, as the header names it, and the handle its C++ struct
        /// derives from (`None` for the root), in declaration order.
        pub const HANDLES: &[(&str, Option<&str>)] =
            &[$((stringify!($Handle), handles!(@base $($Base)?))),*];
    };
    (@base) => { None };
    (@base $Base:ident) => { Some(stringify!($Base)) };
}

handles! {
    RexxObjectPtr = RexxObjectPtr_;
    RexxStringObject = RexxStringObject_ : RexxObjectPtr;
    RexxBufferStringObject = RexxBufferStringObject_ : RexxStringObject;
    RexxArrayObject = RexxArrayObject_ : RexxObjectPtr;
    RexxBufferObject = RexxBufferObject_ : RexxObjectPtr;
    RexxPointerObject = RexxPointerObject_ : RexxObjectPtr;
    RexxMethodObject = RexxMethodObject_ : RexxObjectPtr;
    RexxRoutineObject = RexxRoutineObject_ : RexxObjectPtr;
    RexxPackageObject = RexxPackageObject_ : RexxObjectPtr;
    RexxClassObject = RexxClassObject_ : RexxObjectPtr;
    RexxDirectoryObject = RexxDirectoryObject_ : RexxObjectPtr;
    RexxStringTableObject = RexxStringTableObject_ : RexxObjectPtr;
    RexxSupplierObject = RexxSupplierObject_ : RexxObjectPtr;
    RexxStemObject = RexxStemObject_ : RexxObjectPtr;
    RexxMutableBufferObject = RexxMutableBufferObject_ : RexxObjectPtr;
    RexxVariableReferenceObject = RexxVariableReferenceObject_ : RexxObjectPtr;
}

/// `INSTANCE_INTERFACE_VERSION`.
pub const INSTANCE_INTERFACE_VERSION: wholenumber_t = 102;
/// `THREAD_INTERFACE_VERSION`.
pub const THREAD_INTERFACE_VERSION: wholenumber_t = 104;
/// `METHOD_INTERFACE_VERSION`.
pub const METHOD_INTERFACE_VERSION: wholenumber_t = 102;
/// `CALL_INTERFACE_VERSION`.
pub const CALL_INTERFACE_VERSION: wholenumber_t = 101;
/// `EXIT_INTERFACE_VERSION`.
pub const EXIT_INTERFACE_VERSION: wholenumber_t = 101;
/// `REDIRECT_INTERFACE_VERSION`.
pub const REDIRECT_INTERFACE_VERSION: wholenumber_t = 100;

/// The C spelling of each Rust type token that is not itself the C name.
///
/// A token missing here is a C name already: a typedef the header declares
/// (`logical_t`, `CSTRING`, `RXSTRING`, a handle) or a standard C type of the
/// same name (`intptr_t`). The mapping keys on tokens, not on the type they
/// resolve to, so `logical_t`, `stringsize_t` and `usize` keep three spellings
/// for one Rust type.
pub const C_TYPE_NAMES: &[(&str, &str)] = &[
    ("c_char", "char"),
    ("c_uchar", "unsigned char"),
    ("c_ushort", "unsigned short"),
    ("c_int", "int"),
    ("c_void", "void"),
    ("i8", "int8_t"),
    ("u8", "uint8_t"),
    ("i16", "int16_t"),
    ("u16", "uint16_t"),
    ("i32", "int32_t"),
    ("u32", "uint32_t"),
    ("i64", "int64_t"),
    ("u64", "uint64_t"),
    ("isize", "ssize_t"),
    ("usize", "size_t"),
    ("f32", "float"),
    ("f64", "double"),
    // A context struct is reached through its typedef name, which is the
    // struct in C++ and a pointer to the constant struct in C.
    ("RexxInstance_", "RexxInstance"),
    ("RexxThreadContext_", "RexxThreadContext"),
    ("RexxMethodContext_", "RexxMethodContext"),
    ("RexxCallContext_", "RexxCallContext"),
    ("RexxExitContext_", "RexxExitContext"),
    ("RexxIORedirectorContext_", "RexxIORedirectorContext"),
];

/// The C type the header spells for the Rust type written as `tokens`.
///
/// A raw pointer is the C pointer to the pointee's spelling; an `Option` of a
/// function pointer is that function pointer, which C lets be null.
///
/// # Panics
/// If `tokens` is neither a pointer, an `Option`, a mapped token nor a plain
/// name.
#[must_use]
pub fn c_type(tokens: &str) -> String {
    let tokens = tokens.trim();
    if let Some(pointee) = tokens.strip_prefix("*mut ") {
        return pointer_to(&c_type(pointee), false);
    }
    if let Some(pointee) = tokens.strip_prefix("*const ") {
        return pointer_to(&c_type(pointee), true);
    }
    if let Some(inner) = tokens
        .strip_prefix("Option")
        .map(str::trim_start)
        .and_then(|rest| rest.strip_prefix('<'))
        .and_then(|rest| rest.strip_suffix('>'))
    {
        return c_type(inner);
    }
    if let Some((_, c)) = C_TYPE_NAMES.iter().find(|(rust, _)| *rust == tokens) {
        return (*c).to_string();
    }
    assert!(
        !tokens.is_empty()
            && tokens
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_'),
        "{tokens} is not a type the header can spell"
    );
    tokens.to_string()
}

/// A pointer to `pointee`, constant when `constant` is set.
fn pointer_to(pointee: &str, constant: bool) -> String {
    let pointee = if constant {
        format!("const {pointee}")
    } else {
        pointee.to_string()
    };
    if pointee.ends_with('*') {
        format!("{pointee}*")
    } else {
        format!("{pointee} *")
    }
}

/// How the header spells the type of one struct field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSpelling {
    /// From the tokens of the field's Rust type, through [`c_type`].
    Rust(&'static str),
    /// As given: the C type is narrower than the Rust one, which holds it as an
    /// untyped pointer.
    C(&'static str),
}

/// One field of a `repr(C)` struct as the header declares it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CField {
    /// The field's C name.
    pub name: &'static str,
    /// Its C type.
    pub ty: CSpelling,
}

/// Defines a `repr(C)` struct the header declares and, beside it, its fields as
/// the header declares them ([`CField`]): each field's C name is its Rust name
/// unless `=> "name"` follows the type, and its C type is spelled from the Rust
/// type's tokens unless `as "type"` gives it.
macro_rules! c_struct {
    (
        $(#[$meta:meta])*
        pub struct $Name:ident {
            $( pub $field:ident : $ty:ty $(=> $c_name:literal)? $(as $c_type:literal)? ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        pub struct $Name {
            $( pub $field: $ty, )*
        }

        impl $Name {
            /// Each field as the header declares it, in order.
            pub const C_FIELDS: &'static [CField] = &[$(CField {
                name: c_struct!(@name $field $($c_name)?),
                ty: c_struct!(@type $ty $(, $c_type)?),
            }),*];
        }
    };
    (@name $field:ident) => { stringify!($field) };
    (@name $field:ident $c_name:literal) => { $c_name };
    (@type $ty:ty) => { CSpelling::Rust(stringify!($ty)) };
    (@type $ty:ty, $c_type:literal) => { CSpelling::C($c_type) };
}

c_struct! {
    /// `RXSTRING`.
    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    pub struct RXSTRING {
        pub strlength: usize,
        pub strptr: *mut c_char,
    }
}

c_struct! {
    /// `CONSTRXSTRING`.
    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    pub struct CONSTRXSTRING {
        pub strlength: usize,
        pub strptr: *const c_char,
    }
}

c_struct! {
    /// `RXSYSEXIT`.
    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    pub struct RXSYSEXIT {
        pub sysexit_name: *const c_char,
        pub sysexit_code: c_int,
    }
}

c_struct! {
    /// `RexxContextExit`.
    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    pub struct RexxContextExit {
        pub handler: *mut c_void as "RexxContextExitHandler *",
        pub sysexit_code: c_int,
    }
}

/// The flag word that begins an exit's parameter block: the C bit fields
/// (`RXFNC_FLAGS` and its siblings), the first field in bit 0.
pub type ExitFlags = u32;

/// The bit of the `rxffsub` flag (a function exit's call was made as a
/// subroutine) in the flag word of `RXFNC_FLAGS`.
pub const RXFFSUB_BIT: u32 = 2;

/// The bit of the `rxfmlifo` flag (the line is stacked LIFO) in the flag word of
/// `RXMSQ_FLAGS`.
pub const RXFMLIFO_BIT: u32 = 0;

/// Each exit flag struct, as the header names it, with its one-bit members and
/// the bit of the flag word each one occupies. The dump test checks every
/// position against the header's bit fields.
pub const EXIT_FLAG_BITS: &[(&str, &[(&str, u32)])] = &[
    (
        "RXFNC_FLAGS",
        &[("rxfferr", 0), ("rxffnfnd", 1), ("rxffsub", RXFFSUB_BIT)],
    ),
    (
        "RXOFNC_FLAGS",
        &[("rxfferr", 0), ("rxffnfnd", 1), ("rxffsub", RXFFSUB_BIT)],
    ),
    (
        "RXEXF_FLAGS",
        &[("rxfferr", 0), ("rxffnfnd", 1), ("rxffsub", RXFFSUB_BIT)],
    ),
    ("RXCMD_FLAGS", &[("rxfcfail", 0), ("rxfcerr", 1)]),
    ("RXMSQ_FLAGS", &[("rxfmlifo", RXFMLIFO_BIT)]),
    ("RXHLT_FLAGS", &[("rxfhhalt", 0)]),
    ("RXTRC_FLAGS", &[("rxftrace", 0)]),
];

/// `RXFNC_FLAGS`: an [`ExitFlags`] word with the bits [`EXIT_FLAG_BITS`] names for it.
pub type RXFNC_FLAGS = ExitFlags;
/// `RXOFNC_FLAGS`: an [`ExitFlags`] word with the bits [`EXIT_FLAG_BITS`] names for it.
pub type RXOFNC_FLAGS = ExitFlags;
/// `RXEXF_FLAGS`: an [`ExitFlags`] word with the bits [`EXIT_FLAG_BITS`] names for it.
pub type RXEXF_FLAGS = ExitFlags;
/// `RXCMD_FLAGS`: an [`ExitFlags`] word with the bits [`EXIT_FLAG_BITS`] names for it.
pub type RXCMD_FLAGS = ExitFlags;
/// `RXMSQ_FLAGS`: an [`ExitFlags`] word with the bits [`EXIT_FLAG_BITS`] names for it.
pub type RXMSQ_FLAGS = ExitFlags;
/// `RXHLT_FLAGS`: an [`ExitFlags`] word with the bits [`EXIT_FLAG_BITS`] names for it.
pub type RXHLT_FLAGS = ExitFlags;
/// `RXTRC_FLAGS`: an [`ExitFlags`] word with the bits [`EXIT_FLAG_BITS`] names for it.
pub type RXTRC_FLAGS = ExitFlags;

c_struct! {
    /// `RXFNCCAL_PARM`.
    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    pub struct RXFNCCAL_PARM {
        pub rxfnc_flags: RXFNC_FLAGS,
        pub rxfnc_name: *const c_char,
        pub rxfnc_namel: c_ushort,
        pub rxfnc_que: *const c_char,
        pub rxfnc_quel: c_ushort,
        pub rxfnc_argc: c_ushort,
        pub rxfnc_argv: *const CONSTRXSTRING,
        pub rxfnc_retc: RXSTRING,
    }
}

c_struct! {
    /// `RXOFNCCAL_PARM` and `RXEXFCAL_PARM`, which share
    /// one shape.
    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    pub struct RXOFNCCAL_PARM {
        pub rxfnc_flags: RXOFNC_FLAGS,
        pub rxfnc_name: CONSTRXSTRING,
        pub rxfnc_argc: usize,
        pub rxfnc_argv: *mut RexxObjectPtr,
        pub rxfnc_retc: RexxObjectPtr,
    }
}

c_struct! {
    /// `RXCMDHST_PARM`.
    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    pub struct RXCMDHST_PARM {
        pub rxcmd_flags: RXCMD_FLAGS,
        pub rxcmd_address: *const c_char,
        pub rxcmd_addressl: c_ushort,
        pub rxcmd_dll: *const c_char,
        pub rxcmd_dll_len: c_ushort,
        pub rxcmd_command: CONSTRXSTRING,
        pub rxcmd_retc: RXSTRING,
    }
}

c_struct! {
    /// `RXMSQPSH_PARM`.
    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    pub struct RXMSQPSH_PARM {
        pub rxmsq_flags: RXMSQ_FLAGS,
        pub rxmsq_value: CONSTRXSTRING,
    }
}

c_struct! {
    /// `RXSIOSAY_PARM` and `RXSIOTRC_PARM`. The blocks
    /// of one `RXSTRING` (`RXMSQPLL_PARM`, `RXMSQNAM_PARM`, `RXSIOTRD_PARM`,
    /// `RXSIODTR_PARM`), of one `size_t` (`RXMSQSIZ_PARM`) and of one flag word
    /// (`RXHLTTST_PARM`, `RXTRCTST_PARM`) are that member alone.
    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    pub struct RXSIOSAY_PARM {
        pub rxsio_string: CONSTRXSTRING,
    }
}

c_struct! {
    /// `RXVALCALL_PARM`; `RXVARNOVALUE_PARM` is its last
    /// two members.
    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    pub struct RXVALCALL_PARM {
        pub selector: RexxStringObject,
        pub variable_name: RexxStringObject,
        pub value: RexxObjectPtr,
    }
}

c_struct! {
    /// `RXVARNOVALUE_PARM`.
    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    pub struct RXVARNOVALUE_PARM {
        pub variable_name: RexxStringObject,
        pub value: RexxObjectPtr,
    }
}

c_struct! {
    /// `SHVBLOCK`.
    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    pub struct SHVBLOCK {
        pub shvnext: *mut SHVBLOCK,
        pub shvname: CONSTRXSTRING,
        pub shvvalue: RXSTRING,
        pub shvnamelen: usize,
        pub shvvaluelen: usize,
        pub shvcode: c_uchar,
        pub shvret: c_uchar,
    }
}

c_struct! {
    /// `RexxQueueTime`.
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    pub struct RexxQueueTime {
        pub hours: u16,
        pub minutes: u16,
        pub seconds: u16,
        pub hundredths: u16,
        pub day: u16,
        pub month: u16,
        pub year: u16,
        pub weekday: u16,
        pub microseconds: u32,
        pub yearday: u32,
    }
}

c_struct! {
    /// `REXXDATETIME`.
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    pub struct REXXDATETIME {
        pub hours: u16,
        pub minutes: u16,
        pub seconds: u16,
        pub hundredths: u16,
        pub day: u16,
        pub month: u16,
        pub year: u16,
        pub weekday: u16,
        pub microseconds: usize,
        pub yearday: usize,
        pub valid: u16,
    }
}

c_struct! {
    /// `RexxCondition`.
    #[repr(C)]
    pub struct RexxCondition {
        pub code: wholenumber_t,
        pub rc: wholenumber_t,
        pub position: usize,
        pub conditionName: RexxStringObject,
        pub message: RexxStringObject,
        pub errortext: RexxStringObject,
        pub program: RexxStringObject,
        pub description: RexxStringObject,
        pub additional: RexxArrayObject,
    }
}

/// Defines the anonymous union inside `ValueDescriptor` and, beside it, each
/// member's name and Rust type, which the ABI dump test (`tests/abi_dump.rs`)
/// measures against the header's union, and each member as the header
/// declares it.
macro_rules! value_union {
    ($($member:ident : $ty:ty,)*) => {
        /// The anonymous union inside `ValueDescriptor`.
        ///
        /// Every member is a view of the same word, so which one is live is what
        /// `ValueDescriptor::type` records.
        #[repr(C)]
        pub union ValueUnion {
            $(pub $member: $ty,)*
        }

        impl ValueUnion {
            /// Each member's name and the Rust type it has, in declaration
            /// order.
            #[must_use]
            pub fn member_types() -> Vec<(&'static str, &'static str)> {
                vec![$((stringify!($member), std::any::type_name::<$ty>())),*]
            }

            /// Each member as the header declares it, in order.
            pub const C_FIELDS: &'static [CField] = &[$(CField {
                name: stringify!($member),
                ty: CSpelling::Rust(stringify!($ty)),
            }),*];
        }
    };
}

value_union! {
    value_ARGLIST: RexxArrayObject,
    value_NAME: CSTRING,
    value_SCOPE: RexxObjectPtr,
    value_CSELF: POINTER,
    value_OSELF: RexxClassObject,
    value_SUPER: RexxClassObject,
    value_RexxObjectPtr: RexxObjectPtr,
    value_RexxClassObject: RexxClassObject,
    value_int: c_int,
    value_wholenumber_t: wholenumber_t,
    value_stringsize_t: stringsize_t,
    value_logical_t: logical_t,
    value_double: f64,
    value_CSTRING: CSTRING,
    value_POINTER: POINTER,
    value_RexxStringObject: RexxStringObject,
    value_float: f32,
    value_int8_t: i8,
    value_int16_t: i16,
    value_int32_t: i32,
    value___int32_t: i32,
    value_int64_t: i64,
    value___int64_t: i64,
    value_uint8_t: u8,
    value_uint16_t: u16,
    value_uint32_t: u32,
    value___uint32_t: u32,
    value_uint64_t: u64,
    value___uint64_t: u64,
    value_intptr_t: intptr_t,
    value_uintptr_t: uintptr_t,
    value___uintptr_t: uintptr_t,
    value_size_t: usize,
    value_ssize_t: isize,
    value_RexxArrayObject: RexxArrayObject,
    value_RexxStemObject: RexxStemObject,
    value_POINTERSTRING: POINTER,
    value_RexxMutableBufferObject: RexxMutableBufferObject,
    value_RexxVariableReferenceObject: RexxVariableReferenceObject,
    value_positive_wholenumber_t: wholenumber_t,
    value_nonnegative_wholenumber_t: wholenumber_t,
    value_OPTIONAL_RexxObjectPtr: RexxObjectPtr,
    value_OPTIONAL_int: c_int,
    value_OPTIONAL_wholenumber_t: wholenumber_t,
    value_OPTIONAL_stringsize_t: stringsize_t,
    value_OPTIONAL_logical_t: logical_t,
    value_OPTIONAL_double: f64,
    value_OPTIONAL_CSTRING: CSTRING,
    value_OPTIONAL_RexxClassObject: RexxClassObject,
    value_OPTIONAL_POINTER: POINTER,
    value_OPTIONAL_RexxStringObject: RexxStringObject,
    value_OPTIONAL_float: f32,
    value_OPTIONAL_int8_t: i8,
    value_OPTIONAL_int16_t: i16,
    value_OPTIONAL_int32_t: i32,
    value_OPTIONAL_int64_t: i64,
    value_OPTIONAL_uint8_t: u8,
    value_OPTIONAL_uint16_t: u16,
    value_OPTIONAL_uint32_t: u32,
    value_OPTIONAL_uint64_t: u64,
    value_OPTIONAL_intptr_t: intptr_t,
    value_OPTIONAL_uintptr_t: uintptr_t,
    value_OPTIONAL_ssize_t: isize,
    value_OPTIONAL_size_t: usize,
    value_OPTIONAL_RexxArrayObject: RexxArrayObject,
    value_OPTIONAL_RexxStemObject: RexxStemObject,
    value_OPTIONAL_POINTERSTRING: POINTER,
    value_OPTIONAL_RexxMutableBufferObject: RexxMutableBufferObject,
    value_OPTIONAL_positive_wholenumber_t: wholenumber_t,
    value_OPTIONAL_nonnegative_wholenumber_t: wholenumber_t,
}

c_struct! {
    /// `ValueDescriptor`.
    #[repr(C)]
    pub struct ValueDescriptor {
        pub value: ValueUnion,
        pub r#type: u16 => "type",
        pub flags: u16,
    }
}

/// `RexxPackageLoader` and `RexxPackageUnloader`.
pub type PackageHook = unsafe extern "C-unwind" fn(*mut RexxThreadContext_);
/// `RexxPackageLoader`, the hook a package runs when it is loaded.
pub type RexxPackageLoader = PackageHook;
/// `RexxPackageUnloader`, the hook a package runs when it is unloaded.
pub type RexxPackageUnloader = PackageHook;

c_struct! {
    /// `RexxRoutineEntry`.
    #[repr(C)]
    #[derive(Debug)]
    pub struct RexxRoutineEntry {
        pub style: c_int,
        pub reserved1: c_int,
        pub name: *const c_char,
        pub entry_point: *mut c_void => "entryPoint",
        pub reserved2: c_int,
        pub reserved3: c_int,
    }
}

c_struct! {
    /// `RexxMethodEntry`.
    #[repr(C)]
    #[derive(Debug)]
    pub struct RexxMethodEntry {
        pub style: c_int,
        pub reserved1: c_int,
        pub name: *const c_char,
        pub entry_point: *mut c_void => "entryPoint",
        pub reserved2: c_int,
        pub reserved3: c_int,
    }
}

c_struct! {
    /// `RexxPackageEntry`.
    #[repr(C)]
    #[derive(Debug)]
    pub struct RexxPackageEntry {
        pub size: c_int,
        pub api_version: c_int => "apiVersion",
        pub required_version: c_int => "requiredVersion",
        pub package_name: *const c_char => "packageName",
        pub package_version: *const c_char => "packageVersion",
        pub loader: Option<RexxPackageLoader>,
        pub unloader: Option<RexxPackageUnloader>,
        pub routines: *mut RexxRoutineEntry,
        pub methods: *mut RexxMethodEntry,
    }
}

c_struct! {
    /// `RexxInstance_`.
    #[repr(C)]
    pub struct RexxInstance_ {
        pub functions: *mut RexxInstanceInterface,
        pub applicationData: *mut c_void,
    }
}

c_struct! {
    /// `RexxThreadContext_`.
    #[repr(C)]
    pub struct RexxThreadContext_ {
        pub instance: *mut RexxInstance_,
        pub functions: *mut RexxThreadInterface,
    }
}

c_struct! {
    /// `RexxMethodContext_`.
    #[repr(C)]
    pub struct RexxMethodContext_ {
        pub threadContext: *mut RexxThreadContext_,
        pub functions: *mut MethodContextInterface,
        pub arguments: *mut ValueDescriptor,
    }
}

c_struct! {
    /// `RexxCallContext_`.
    #[repr(C)]
    pub struct RexxCallContext_ {
        pub threadContext: *mut RexxThreadContext_,
        pub functions: *mut CallContextInterface,
        pub arguments: *mut ValueDescriptor,
    }
}

c_struct! {
    /// `RexxExitContext_`.
    #[repr(C)]
    pub struct RexxExitContext_ {
        pub threadContext: *mut RexxThreadContext_,
        pub functions: *mut ExitContextInterface,
        pub arguments: *mut ValueDescriptor,
    }
}

c_struct! {
    /// `RexxIORedirectorContext_`.
    #[repr(C)]
    pub struct RexxIORedirectorContext_ {
        pub functions: *mut IORedirectorInterface,
    }
}

/// A context we handed out, together with the state that owns it.
///
/// `C` is first by value, so the address of the wrapper is the address of the
/// public struct and `ffi::owner_of` recovers `owner` by casting back.
#[repr(C)]
pub struct Owned<C, T> {
    pub context: C,
    pub owner: *mut T,
}

/// The refusal text for `entry`, a stub member no populated table leaves.
fn refusal_text(entry: &str) -> String {
    format!("{entry} is not implemented")
}

/// Abandon the process, naming the entry the caller reached.
///
/// # Panics
/// Always. The stubs below call it, and each is an `extern "C"` frame, so the
/// panic aborts rather than unwinding into the extension's stack.
fn abort(entry: &str) -> ! {
    panic!("{}", refusal_text(entry));
}

/// Abandon the process, naming the entry the caller reached, without
/// unwinding: the stubs that call it are `extern "C-unwind"` frames, which a
/// panic would leave into the extension's stack.
fn abort_now(entry: &str) -> ! {
    eprintln!("{}", refusal_text(entry));
    std::process::abort()
}

thread_local! {
    /// The first refusing slot the running native call or package hook
    /// reached.
    ///
    /// Set by a refusing stub and taken by [`recording_refusals`] around the
    /// one call it records, on the thread making that call, so it holds no
    /// state beyond that call: a nested call saves the outer record and puts it
    /// back. Every place extension code runs reads the record and refuses
    /// loudly on it: `invoke::call` for a method or routine, `invoke::hook` for
    /// a loader or unloader.
    static REFUSED: Cell<Option<&'static str>> = const { Cell::new(None) };
}

/// Records that the running call reached `entry`, keeping an earlier record.
pub(crate) fn refuse(entry: &'static str) {
    if REFUSED.get().is_none() {
        REFUSED.set(Some(entry));
    }
}

/// Runs `call` over a fresh record and answers what it answered with the first
/// refusing slot it reached, restoring the record in force before.
pub(crate) fn recording_refusals<R>(call: impl FnOnce() -> R) -> (R, Option<&'static str>) {
    let outer = REFUSED.replace(None);
    let answered = call();
    let refused = REFUSED.replace(outer);
    (answered, refused)
}

/// What a refusing stub answers after recording itself: the value a stub
/// answers on its failure path, a null handle, a zero or `false`, with any
/// other value named at its member.
trait RefusedValue {
    const REFUSED: Self;
}

impl<T> RefusedValue for *mut T {
    const REFUSED: Self = std::ptr::null_mut();
}

impl RefusedValue for usize {
    const REFUSED: Self = 0;
}

impl RefusedValue for isize {
    const REFUSED: Self = 0;
}

/// The Rust type of one interface member.
///
/// A function member is `unsafe` to call, because it trusts raw pointers that
/// only its caller can vouch for.
macro_rules! entry_type {
    (value $ty:ty = $value:expr) => {
        $ty
    };
    (unwinds call($($arg:ty),*)) => {
        unsafe extern "C-unwind" fn($($arg),*)
    };
    (unwinds call($($arg:ty),*) -> $ret:ty) => {
        unsafe extern "C-unwind" fn($($arg),*) -> $ret
    };
    ($(aborts)? call($($arg:ty),*)) => {
        unsafe extern "C" fn($($arg),*)
    };
    ($(aborts)? call($($arg:ty),*) -> $ret:ty $(, failing $value:expr)?) => {
        unsafe extern "C" fn($($arg),*) -> $ret
    };
}

/// The value one interface member holds in a table nothing has built yet.
///
/// A function member records itself and returns, unless it is marked `aborts`,
/// which answers a data pointer the extension dereferences, or `unwinds`, which
/// leaves the extension by unwinding as an exception does: no value either could
/// return is safe.
macro_rules! entry_stub {
    ($entry:expr, value $ty:ty = $value:expr) => {
        $value
    };
    ($entry:expr, call($($arg:ty),*)) => {{
        extern "C" fn stub($(_: $arg),*) {
            refuse($entry)
        }
        stub
    }};
    ($entry:expr, call($($arg:ty),*) -> $ret:ty) => {{
        extern "C" fn stub($(_: $arg),*) -> $ret {
            refuse($entry);
            <$ret as RefusedValue>::REFUSED
        }
        stub
    }};
    ($entry:expr, call($($arg:ty),*) -> $ret:ty, failing $value:expr) => {{
        extern "C" fn stub($(_: $arg),*) -> $ret {
            refuse($entry);
            $value
        }
        stub
    }};
    ($entry:expr, unwinds call($($arg:ty),*)) => {{
        extern "C-unwind" fn stub($(_: $arg),*) {
            abort_now($entry)
        }
        stub
    }};
    ($entry:expr, unwinds call($($arg:ty),*) -> $ret:ty) => {{
        extern "C-unwind" fn stub($(_: $arg),*) -> $ret {
            refuse($entry);
            <$ret as RefusedValue>::REFUSED
        }
        stub
    }};
    ($entry:expr, aborts call($($arg:ty),*)) => {{
        extern "C" fn stub($(_: $arg),*) {
            abort($entry)
        }
        stub
    }};
    ($entry:expr, aborts call($($arg:ty),*) -> $ret:ty) => {{
        extern "C" fn stub($(_: $arg),*) -> $ret {
            abort($entry)
        }
        stub
    }};
}

/// A function member's address, `None` for a data member.
macro_rules! entry_address {
    ($member:expr, value $ty:ty = $value:expr) => {
        None
    };
    ($member:expr, $($kind:tt)*) => {
        Some($member as usize)
    };
}

/// Whether one interface member's refusing stub aborts: marked `aborts` or
/// `unwinds`.
macro_rules! entry_aborts {
    (aborts $($rest:tt)*) => {
        true
    };
    (unwinds call($($arg:ty),*) -> $ret:ty) => {
        false
    };
    (unwinds $($rest:tt)*) => {
        true
    };
    ($($rest:tt)*) => {
        false
    };
}

/// One interface member as the header declares it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CMember {
    /// A data member, with the tokens of its Rust type.
    Value(&'static str),
    /// A function pointer: the tokens of its result's Rust type (`None` when it
    /// returns nothing) and of each parameter's.
    Function {
        returns: Option<&'static str>,
        parameters: &'static [&'static str],
    },
}

/// One interface member's [`CMember`], from the same tokens that give its Rust
/// type.
macro_rules! entry_c {
    (value $ty:ty = $value:expr) => {
        CMember::Value(stringify!($ty))
    };
    ($(aborts)? $(unwinds)? call($($arg:ty),*)) => {
        CMember::Function { returns: None, parameters: &[$(stringify!($arg)),*] }
    };
    ($(aborts)? $(unwinds)? call($($arg:ty),*) -> $ret:ty $(, failing $value:expr)?) => {
        CMember::Function { returns: Some(stringify!($ret)), parameters: &[$(stringify!($arg)),*] }
    };
}

/// One interface table, and the member names a test measures it against.
///
/// `populated` additionally builds `REFUSING`, in which no member is a null
/// pointer: a function member's type is not nullable, and the literal must
/// name every field.
macro_rules! interface {
    (@shape $(#[$meta:meta])* $Name:ident { $($field:ident : { $($kind:tt)* }),* $(,)? }) => {
        $(#[$meta])*
        #[repr(C)]
        pub struct $Name {
            $( pub $field: entry_type!($($kind)*), )*
        }

        impl $Name {
            /// The member names the frozen header declares, in order.
            pub const FIELDS: &'static [&'static str] = &[$(stringify!($field)),*];

            /// Each member as the header declares it, in order.
            pub const C_DECLS: &'static [(&'static str, CMember)] =
                &[$((stringify!($field), entry_c!($($kind)*))),*];

            /// For each of [`Self::FIELDS`], whether its refusing stub aborts
            /// rather than records and returns.
            pub const ABORTS: &'static [bool] = &[$(entry_aborts!($($kind)*)),*];

            /// For each of [`Self::FIELDS`], the Rust type of the member.
            #[must_use]
            pub fn member_types() -> Vec<&'static str> {
                vec![$(std::any::type_name::<entry_type!($($kind)*)>()),*]
            }

            /// For each function member, its name and the address it holds.
            #[must_use]
            pub fn addresses(&self) -> Vec<(&'static str, usize)> {
                [$( (stringify!($field), entry_address!(self.$field, $($kind)*)) ),*]
                    .into_iter()
                    .filter_map(|(name, address)| Some((name, address?)))
                    .collect()
            }
        }
    };
    (@refusing $Name:ident { $($field:ident : { $($kind:tt)* }),* $(,)? }) => {
        impl $Name {
            /// Every function member refusing loudly, for a table whose
            /// bodies later tasks write.
            ///
            /// The object members are null until an interpreter instance
            /// fills them, which is not this task's.
            pub const REFUSING: Self = Self {
                $( $field: entry_stub!(
                    concat!(stringify!($Name), ".", stringify!($field)),
                    $($kind)*
                ), )*
            };
        }
    };
    ($(#[$meta:meta])* $Name:ident, populated { $($body:tt)* }) => {
        interface!(@shape $(#[$meta])* $Name { $($body)* });
        interface!(@refusing $Name { $($body)* });
    };
}

interface! {
    /// `RexxInstanceInterface`.
    RexxInstanceInterface, populated {
        interfaceVersion: { value wholenumber_t = INSTANCE_INTERFACE_VERSION },
        Terminate: { call(*mut RexxInstance_) },
        AttachThread: { call(*mut RexxInstance_, *mut *mut RexxThreadContext_) -> logical_t },
        InterpreterVersion: { call(*mut RexxInstance_) -> usize },
        LanguageLevel: { call(*mut RexxInstance_) -> usize },
        Halt: { call(*mut RexxInstance_) },
        SetTrace: { call(*mut RexxInstance_, logical_t) },
        AddCommandEnvironment: { call(*mut RexxInstance_, *const c_char, REXXPFN, c_int) },
        StackCommandEnvironment: { call(*mut RexxInstance_, *const c_char, REXXPFN, c_int) -> logical_t },
    }
}

interface! {
    /// `RexxThreadInterface`.
    RexxThreadInterface, populated {
        interfaceVersion: { value wholenumber_t = THREAD_INTERFACE_VERSION },
        DetachThread: { call(*mut RexxThreadContext_) },
        HaltThread: { call(*mut RexxThreadContext_) },
        SetThreadTrace: { call(*mut RexxThreadContext_, logical_t) },
        RequestGlobalReference: { call(*mut RexxThreadContext_, RexxObjectPtr) -> RexxObjectPtr },
        ReleaseGlobalReference: { call(*mut RexxThreadContext_, RexxObjectPtr) },
        ReleaseLocalReference: { call(*mut RexxThreadContext_, RexxObjectPtr) },
        SendMessage: { call(*mut RexxThreadContext_, RexxObjectPtr, CSTRING, RexxArrayObject) -> RexxObjectPtr },
        SendMessage0: { call(*mut RexxThreadContext_, RexxObjectPtr, CSTRING) -> RexxObjectPtr },
        SendMessage1: { call(*mut RexxThreadContext_, RexxObjectPtr, CSTRING, RexxObjectPtr) -> RexxObjectPtr },
        SendMessage2: { call(*mut RexxThreadContext_, RexxObjectPtr, CSTRING, RexxObjectPtr, RexxObjectPtr) -> RexxObjectPtr },
        GetLocalEnvironment: { call(*mut RexxThreadContext_) -> RexxDirectoryObject },
        GetGlobalEnvironment: { call(*mut RexxThreadContext_) -> RexxDirectoryObject },
        IsInstanceOf: { call(*mut RexxThreadContext_, RexxObjectPtr, RexxClassObject) -> logical_t },
        IsOfType: { call(*mut RexxThreadContext_, RexxObjectPtr, CSTRING) -> logical_t },
        HasMethod: { call(*mut RexxThreadContext_, RexxObjectPtr, CSTRING) -> logical_t },
        LoadPackage: { call(*mut RexxThreadContext_, CSTRING) -> RexxPackageObject },
        LoadPackageFromData: { call(*mut RexxThreadContext_, CSTRING, CSTRING, usize) -> RexxPackageObject },
        LoadLibrary: { unwinds call(*mut RexxThreadContext_, CSTRING) -> logical_t },
        RegisterLibrary: { unwinds call(*mut RexxThreadContext_, CSTRING, *mut RexxPackageEntry) -> logical_t },
        FindClass: { call(*mut RexxThreadContext_, CSTRING) -> RexxClassObject },
        FindPackageClass: { call(*mut RexxThreadContext_, RexxPackageObject, CSTRING) -> RexxClassObject },
        GetPackageRoutines: { call(*mut RexxThreadContext_, RexxPackageObject) -> RexxDirectoryObject },
        GetPackagePublicRoutines: { call(*mut RexxThreadContext_, RexxPackageObject) -> RexxDirectoryObject },
        GetPackageClasses: { call(*mut RexxThreadContext_, RexxPackageObject) -> RexxDirectoryObject },
        GetPackagePublicClasses: { call(*mut RexxThreadContext_, RexxPackageObject) -> RexxDirectoryObject },
        GetPackageMethods: { call(*mut RexxThreadContext_, RexxPackageObject) -> RexxDirectoryObject },
        CallRoutine: { call(*mut RexxThreadContext_, RexxRoutineObject, RexxArrayObject) -> RexxObjectPtr },
        CallProgram: { call(*mut RexxThreadContext_, CSTRING, RexxArrayObject) -> RexxObjectPtr },
        NewMethod: { call(*mut RexxThreadContext_, CSTRING, CSTRING, usize) -> RexxMethodObject },
        NewRoutine: { call(*mut RexxThreadContext_, CSTRING, CSTRING, usize) -> RexxRoutineObject },
        IsRoutine: { call(*mut RexxThreadContext_, RexxObjectPtr) -> logical_t },
        IsMethod: { call(*mut RexxThreadContext_, RexxObjectPtr) -> logical_t },
        GetRoutinePackage: { call(*mut RexxThreadContext_, RexxRoutineObject) -> RexxPackageObject },
        GetMethodPackage: { call(*mut RexxThreadContext_, RexxMethodObject) -> RexxPackageObject },
        ObjectToCSelf: { aborts call(*mut RexxThreadContext_, RexxObjectPtr) -> POINTER },
        WholeNumberToObject: { call(*mut RexxThreadContext_, wholenumber_t) -> RexxObjectPtr },
        UintptrToObject: { call(*mut RexxThreadContext_, uintptr_t) -> RexxObjectPtr },
        IntptrToObject: { call(*mut RexxThreadContext_, intptr_t) -> RexxObjectPtr },
        ValueToObject: { call(*mut RexxThreadContext_, *mut ValueDescriptor) -> RexxObjectPtr },
        ValuesToObject: { call(*mut RexxThreadContext_, *mut ValueDescriptor, usize) -> RexxArrayObject },
        ObjectToValue: { call(*mut RexxThreadContext_, RexxObjectPtr, *mut ValueDescriptor) -> logical_t },
        StringSizeToObject: { call(*mut RexxThreadContext_, stringsize_t) -> RexxObjectPtr },
        ObjectToWholeNumber: { call(*mut RexxThreadContext_, RexxObjectPtr, *mut wholenumber_t) -> logical_t },
        ObjectToStringSize: { call(*mut RexxThreadContext_, RexxObjectPtr, *mut stringsize_t) -> logical_t },
        Int64ToObject: { call(*mut RexxThreadContext_, i64) -> RexxObjectPtr },
        UnsignedInt64ToObject: { call(*mut RexxThreadContext_, u64) -> RexxObjectPtr },
        ObjectToInt64: { call(*mut RexxThreadContext_, RexxObjectPtr, *mut i64) -> logical_t },
        ObjectToUnsignedInt64: { call(*mut RexxThreadContext_, RexxObjectPtr, *mut u64) -> logical_t },
        Int32ToObject: { call(*mut RexxThreadContext_, i32) -> RexxObjectPtr },
        UnsignedInt32ToObject: { call(*mut RexxThreadContext_, u32) -> RexxObjectPtr },
        ObjectToInt32: { call(*mut RexxThreadContext_, RexxObjectPtr, *mut i32) -> logical_t },
        ObjectToUnsignedInt32: { call(*mut RexxThreadContext_, RexxObjectPtr, *mut u32) -> logical_t },
        ObjectToUintptr: { call(*mut RexxThreadContext_, RexxObjectPtr, *mut uintptr_t) -> logical_t },
        ObjectToIntptr: { call(*mut RexxThreadContext_, RexxObjectPtr, *mut intptr_t) -> logical_t },
        ObjectToLogical: { call(*mut RexxThreadContext_, RexxObjectPtr, *mut logical_t) -> logical_t },
        LogicalToObject: { call(*mut RexxThreadContext_, logical_t) -> RexxObjectPtr },
        DoubleToObject: { call(*mut RexxThreadContext_, f64) -> RexxObjectPtr },
        DoubleToObjectWithPrecision: { call(*mut RexxThreadContext_, f64, usize) -> RexxObjectPtr },
        ObjectToDouble: { call(*mut RexxThreadContext_, RexxObjectPtr, *mut f64) -> logical_t },
        ObjectToString: { call(*mut RexxThreadContext_, RexxObjectPtr) -> RexxStringObject },
        ObjectToStringValue: { aborts call(*mut RexxThreadContext_, RexxObjectPtr) -> CSTRING },
        StringGet: { call(*mut RexxThreadContext_, RexxStringObject, usize, POINTER, usize) -> usize },
        StringLength: { call(*mut RexxThreadContext_, RexxStringObject) -> usize },
        StringData: { aborts call(*mut RexxThreadContext_, RexxStringObject) -> CSTRING },
        NewString: { call(*mut RexxThreadContext_, CSTRING, usize) -> RexxStringObject },
        NewStringFromAsciiz: { call(*mut RexxThreadContext_, CSTRING) -> RexxStringObject },
        StringUpper: { call(*mut RexxThreadContext_, RexxStringObject) -> RexxStringObject },
        StringLower: { call(*mut RexxThreadContext_, RexxStringObject) -> RexxStringObject },
        IsString: { call(*mut RexxThreadContext_, RexxObjectPtr) -> logical_t },
        NewBufferString: { call(*mut RexxThreadContext_, usize) -> RexxBufferStringObject },
        BufferStringLength: { call(*mut RexxThreadContext_, RexxBufferStringObject) -> usize },
        BufferStringData: { aborts call(*mut RexxThreadContext_, RexxBufferStringObject) -> POINTER },
        FinishBufferString: { call(*mut RexxThreadContext_, RexxBufferStringObject, usize) -> RexxStringObject },
        DirectoryPut: { call(*mut RexxThreadContext_, RexxDirectoryObject, RexxObjectPtr, CSTRING) },
        DirectoryAt: { call(*mut RexxThreadContext_, RexxDirectoryObject, CSTRING) -> RexxObjectPtr },
        DirectoryRemove: { call(*mut RexxThreadContext_, RexxDirectoryObject, CSTRING) -> RexxObjectPtr },
        NewDirectory: { call(*mut RexxThreadContext_) -> RexxDirectoryObject },
        IsDirectory: { call(*mut RexxThreadContext_, RexxObjectPtr) -> logical_t },
        ArrayAt: { call(*mut RexxThreadContext_, RexxArrayObject, usize) -> RexxObjectPtr },
        ArrayPut: { call(*mut RexxThreadContext_, RexxArrayObject, RexxObjectPtr, usize) },
        ArrayAppend: { call(*mut RexxThreadContext_, RexxArrayObject, RexxObjectPtr) -> usize },
        ArrayAppendString: { call(*mut RexxThreadContext_, RexxArrayObject, CSTRING, usize) -> usize },
        ArraySize: { call(*mut RexxThreadContext_, RexxArrayObject) -> usize },
        ArrayItems: { call(*mut RexxThreadContext_, RexxArrayObject) -> usize },
        ArrayDimension: { call(*mut RexxThreadContext_, RexxArrayObject) -> usize },
        NewArray: { call(*mut RexxThreadContext_, usize) -> RexxArrayObject },
        ArrayOfOne: { call(*mut RexxThreadContext_, RexxObjectPtr) -> RexxArrayObject },
        ArrayOfTwo: { call(*mut RexxThreadContext_, RexxObjectPtr, RexxObjectPtr) -> RexxArrayObject },
        ArrayOfThree: { call(*mut RexxThreadContext_, RexxObjectPtr, RexxObjectPtr, RexxObjectPtr) -> RexxArrayObject },
        ArrayOfFour: { call(*mut RexxThreadContext_, RexxObjectPtr, RexxObjectPtr, RexxObjectPtr, RexxObjectPtr) -> RexxArrayObject },
        IsArray: { call(*mut RexxThreadContext_, RexxObjectPtr) -> logical_t },
        BufferData: { aborts call(*mut RexxThreadContext_, RexxBufferObject) -> POINTER },
        BufferLength: { call(*mut RexxThreadContext_, RexxBufferObject) -> usize },
        NewBuffer: { call(*mut RexxThreadContext_, usize) -> RexxBufferObject },
        IsBuffer: { call(*mut RexxThreadContext_, RexxObjectPtr) -> logical_t },
        PointerValue: { aborts call(*mut RexxThreadContext_, RexxPointerObject) -> POINTER },
        NewPointer: { call(*mut RexxThreadContext_, POINTER) -> RexxPointerObject },
        IsPointer: { call(*mut RexxThreadContext_, RexxObjectPtr) -> logical_t },
        SupplierItem: { call(*mut RexxThreadContext_, RexxSupplierObject) -> RexxObjectPtr },
        SupplierIndex: { call(*mut RexxThreadContext_, RexxSupplierObject) -> RexxObjectPtr },
        SupplierAvailable: { call(*mut RexxThreadContext_, RexxSupplierObject) -> logical_t },
        SupplierNext: { call(*mut RexxThreadContext_, RexxSupplierObject) },
        NewSupplier: { call(*mut RexxThreadContext_, RexxArrayObject, RexxArrayObject) -> RexxSupplierObject },
        NewStem: { call(*mut RexxThreadContext_, CSTRING) -> RexxStemObject },
        SetStemElement: { call(*mut RexxThreadContext_, RexxStemObject, CSTRING, RexxObjectPtr) },
        GetStemElement: { call(*mut RexxThreadContext_, RexxStemObject, CSTRING) -> RexxObjectPtr },
        DropStemElement: { call(*mut RexxThreadContext_, RexxStemObject, CSTRING) },
        SetStemArrayElement: { call(*mut RexxThreadContext_, RexxStemObject, usize, RexxObjectPtr) },
        GetStemArrayElement: { call(*mut RexxThreadContext_, RexxStemObject, usize) -> RexxObjectPtr },
        DropStemArrayElement: { call(*mut RexxThreadContext_, RexxStemObject, usize) },
        GetAllStemElements: { call(*mut RexxThreadContext_, RexxStemObject) -> RexxDirectoryObject },
        GetStemValue: { call(*mut RexxThreadContext_, RexxStemObject) -> RexxObjectPtr },
        IsStem: { call(*mut RexxThreadContext_, RexxObjectPtr) -> logical_t },
        RaiseException0: { call(*mut RexxThreadContext_, usize) },
        RaiseException1: { call(*mut RexxThreadContext_, usize, RexxObjectPtr) },
        RaiseException2: { call(*mut RexxThreadContext_, usize, RexxObjectPtr, RexxObjectPtr) },
        RaiseException: { call(*mut RexxThreadContext_, usize, RexxArrayObject) },
        RaiseCondition: { call(*mut RexxThreadContext_, CSTRING, RexxStringObject, RexxObjectPtr, RexxObjectPtr) },
        CheckCondition: { call(*mut RexxThreadContext_) -> logical_t },
        GetConditionInfo: { call(*mut RexxThreadContext_) -> RexxDirectoryObject },
        DecodeConditionInfo: { call(*mut RexxThreadContext_, RexxDirectoryObject, *mut RexxCondition) },
        ClearCondition: { call(*mut RexxThreadContext_) },
        RexxNil: { value RexxObjectPtr = std::ptr::null_mut() },
        RexxTrue: { value RexxObjectPtr = std::ptr::null_mut() },
        RexxFalse: { value RexxObjectPtr = std::ptr::null_mut() },
        RexxNullString: { value RexxStringObject = std::ptr::null_mut() },
        ObjectToCSelfScoped: { aborts call(*mut RexxThreadContext_, RexxObjectPtr, RexxObjectPtr) -> POINTER },
        // `Error_Interpretation/1000`, where
        // `Error_Interpretation` is 49000.
        DisplayCondition: { call(*mut RexxThreadContext_) -> wholenumber_t, failing 49 },
        MutableBufferData: { aborts call(*mut RexxThreadContext_, RexxMutableBufferObject) -> POINTER },
        MutableBufferLength: { call(*mut RexxThreadContext_, RexxMutableBufferObject) -> usize },
        SetMutableBufferLength: { call(*mut RexxThreadContext_, RexxMutableBufferObject, usize) -> usize },
        NewMutableBuffer: { call(*mut RexxThreadContext_, usize) -> RexxMutableBufferObject },
        IsMutableBuffer: { call(*mut RexxThreadContext_, RexxObjectPtr) -> logical_t },
        MutableBufferCapacity: { call(*mut RexxThreadContext_, RexxMutableBufferObject) -> usize },
        SetMutableBufferCapacity: { aborts call(*mut RexxThreadContext_, RexxMutableBufferObject, usize) -> POINTER },
        VariableReferenceName: { call(*mut RexxThreadContext_, RexxVariableReferenceObject) -> RexxStringObject },
        VariableReferenceValue: { call(*mut RexxThreadContext_, RexxVariableReferenceObject) -> RexxObjectPtr },
        SetVariableReferenceValue: { call(*mut RexxThreadContext_, RexxVariableReferenceObject, RexxObjectPtr) },
        IsVariableReference: { call(*mut RexxThreadContext_, RexxObjectPtr) -> logical_t },
        StringTablePut: { call(*mut RexxThreadContext_, RexxStringTableObject, RexxObjectPtr, CSTRING) },
        StringTableAt: { call(*mut RexxThreadContext_, RexxStringTableObject, CSTRING) -> RexxObjectPtr },
        StringTableRemove: { call(*mut RexxThreadContext_, RexxStringTableObject, CSTRING) -> RexxObjectPtr },
        NewStringTable: { call(*mut RexxThreadContext_) -> RexxStringTableObject },
        IsStringTable: { call(*mut RexxThreadContext_, RexxObjectPtr) -> logical_t },
        SendMessageScoped: { call(*mut RexxThreadContext_, RexxObjectPtr, CSTRING, RexxClassObject, RexxArrayObject) -> RexxObjectPtr },
        GetInterpreterInstance: { aborts call(*mut RexxThreadContext_) -> *mut RexxInstance_ },
        RaiseConditionObject: { call(*mut RexxThreadContext_, RexxDirectoryObject) },
    }
}

interface! {
    /// `MethodContextInterface`.
    MethodContextInterface, populated {
        interfaceVersion: { value wholenumber_t = METHOD_INTERFACE_VERSION },
        GetArguments: { call(*mut RexxMethodContext_) -> RexxArrayObject },
        GetArgument: { call(*mut RexxMethodContext_, usize) -> RexxObjectPtr },
        GetMessageName: { aborts call(*mut RexxMethodContext_) -> CSTRING },
        GetMethod: { call(*mut RexxMethodContext_) -> RexxMethodObject },
        GetSelf: { call(*mut RexxMethodContext_) -> RexxObjectPtr },
        GetSuper: { call(*mut RexxMethodContext_) -> RexxClassObject },
        GetScope: { call(*mut RexxMethodContext_) -> RexxObjectPtr },
        SetObjectVariable: { call(*mut RexxMethodContext_, CSTRING, RexxObjectPtr) },
        GetObjectVariable: { call(*mut RexxMethodContext_, CSTRING) -> RexxObjectPtr },
        DropObjectVariable: { call(*mut RexxMethodContext_, CSTRING) },
        ForwardMessage: { call(*mut RexxMethodContext_, RexxObjectPtr, CSTRING, RexxClassObject, RexxArrayObject) -> RexxObjectPtr },
        SetGuardOn: { call(*mut RexxMethodContext_) },
        SetGuardOff: { call(*mut RexxMethodContext_) },
        FindContextClass: { call(*mut RexxMethodContext_, CSTRING) -> RexxClassObject },
        GetCSelf: { aborts call(*mut RexxMethodContext_) -> POINTER },
        AllocateObjectMemory: { aborts call(*mut RexxMethodContext_, usize) -> POINTER },
        FreeObjectMemory: { call(*mut RexxMethodContext_, POINTER) },
        ReallocateObjectMemory: { aborts call(*mut RexxMethodContext_, POINTER, usize) -> POINTER },
        GetObjectVariableReference: { call(*mut RexxMethodContext_, CSTRING) -> RexxVariableReferenceObject },
        SetGuardOnWhenUpdated: { call(*mut RexxMethodContext_, CSTRING) -> RexxObjectPtr },
        SetGuardOffWhenUpdated: { call(*mut RexxMethodContext_, CSTRING) -> RexxObjectPtr },
        ThrowException0: { unwinds call(*mut RexxMethodContext_, usize) },
        ThrowException1: { unwinds call(*mut RexxMethodContext_, usize, RexxObjectPtr) },
        ThrowException2: { unwinds call(*mut RexxMethodContext_, usize, RexxObjectPtr, RexxObjectPtr) },
        ThrowException: { unwinds call(*mut RexxMethodContext_, usize, RexxArrayObject) },
        ThrowCondition: { unwinds call(*mut RexxMethodContext_, CSTRING, RexxStringObject, RexxObjectPtr, RexxObjectPtr) },
    }
}

interface! {
    /// `CallContextInterface`.
    CallContextInterface, populated {
        interfaceVersion: { value wholenumber_t = CALL_INTERFACE_VERSION },
        GetArguments: { call(*mut RexxCallContext_) -> RexxArrayObject },
        GetArgument: { call(*mut RexxCallContext_, usize) -> RexxObjectPtr },
        GetRoutineName: { aborts call(*mut RexxCallContext_) -> CSTRING },
        GetRoutine: { call(*mut RexxCallContext_) -> RexxRoutineObject },
        SetContextVariable: { call(*mut RexxCallContext_, CSTRING, RexxObjectPtr) },
        GetContextVariable: { call(*mut RexxCallContext_, CSTRING) -> RexxObjectPtr },
        DropContextVariable: { call(*mut RexxCallContext_, CSTRING) },
        GetAllContextVariables: { call(*mut RexxCallContext_) -> RexxDirectoryObject },
        ResolveStemVariable: { call(*mut RexxCallContext_, RexxObjectPtr) -> RexxStemObject },
        InvalidRoutine: { call(*mut RexxCallContext_) },
        GetContextDigits: { call(*mut RexxCallContext_) -> stringsize_t },
        GetContextFuzz: { call(*mut RexxCallContext_) -> stringsize_t },
        GetContextForm: { call(*mut RexxCallContext_) -> logical_t },
        GetCallerContext: { call(*mut RexxCallContext_) -> RexxObjectPtr },
        FindContextClass: { call(*mut RexxCallContext_, CSTRING) -> RexxClassObject },
        GetContextVariableReference: { call(*mut RexxCallContext_, CSTRING) -> RexxVariableReferenceObject },
        ThrowException0: { unwinds call(*mut RexxCallContext_, usize) },
        ThrowException1: { unwinds call(*mut RexxCallContext_, usize, RexxObjectPtr) },
        ThrowException2: { unwinds call(*mut RexxCallContext_, usize, RexxObjectPtr, RexxObjectPtr) },
        ThrowException: { unwinds call(*mut RexxCallContext_, usize, RexxArrayObject) },
        ThrowCondition: { unwinds call(*mut RexxCallContext_, CSTRING, RexxStringObject, RexxObjectPtr, RexxObjectPtr) },
    }
}

interface! {
    /// `ExitContextInterface`.
    ExitContextInterface, populated {
        interfaceVersion: { value wholenumber_t = EXIT_INTERFACE_VERSION },
        SetContextVariable: { call(*mut RexxExitContext_, CSTRING, RexxObjectPtr) },
        GetContextVariable: { call(*mut RexxExitContext_, CSTRING) -> RexxObjectPtr },
        DropContextVariable: { call(*mut RexxExitContext_, CSTRING) },
        GetAllContextVariables: { call(*mut RexxExitContext_) -> RexxDirectoryObject },
        GetCallerContext: { call(*mut RexxExitContext_) -> RexxObjectPtr },
        GetContextVariableReference: { call(*mut RexxExitContext_, CSTRING) -> RexxVariableReferenceObject },
        ThrowException0: { unwinds call(*mut RexxExitContext_, usize) },
        ThrowException1: { unwinds call(*mut RexxExitContext_, usize, RexxObjectPtr) },
        ThrowException2: { unwinds call(*mut RexxExitContext_, usize, RexxObjectPtr, RexxObjectPtr) },
        ThrowException: { unwinds call(*mut RexxExitContext_, usize, RexxArrayObject) },
        ThrowCondition: { unwinds call(*mut RexxExitContext_, CSTRING, RexxStringObject, RexxObjectPtr, RexxObjectPtr) },
    }
}

interface! {
    /// `IORedirectorInterface`.
    IORedirectorInterface, populated {
        interfaceVersion: { value wholenumber_t = REDIRECT_INTERFACE_VERSION },
        ReadInput: { call(*mut RexxIORedirectorContext_, *mut CSTRING, *mut usize) },
        ReadInputBuffer: { call(*mut RexxIORedirectorContext_, *mut CSTRING, *mut usize) },
        WriteOutput: { call(*mut RexxIORedirectorContext_, CSTRING, usize) },
        WriteError: { call(*mut RexxIORedirectorContext_, CSTRING, usize) },
        WriteOutputBuffer: { call(*mut RexxIORedirectorContext_, CSTRING, usize) },
        WriteErrorBuffer: { call(*mut RexxIORedirectorContext_, CSTRING, usize) },
        IsInputRedirected: { call(*mut RexxIORedirectorContext_) -> logical_t },
        IsOutputRedirected: { call(*mut RexxIORedirectorContext_) -> logical_t },
        IsErrorRedirected: { call(*mut RexxIORedirectorContext_) -> logical_t },
        AreOutputAndErrorSameTarget: { call(*mut RexxIORedirectorContext_) -> logical_t },
        IsRedirectionRequested: { call(*mut RexxIORedirectorContext_) -> logical_t },
    }
}

/// A C++ method of a context struct that calls one member of an interface
/// table under a name or a spelling the member does not give it.
///
/// The parameters are those after the context; each is spelled from Rust type
/// tokens as a table member's are. The spec states no overload set yet (#649):
/// the names, overloads and targets below are the interface's as extensions
/// call it, with meaning from the rexxapi book.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CxxMethod {
    /// The method's name.
    pub name: &'static str,
    /// The table member it calls.
    pub target: &'static str,
    /// Its parameters after the context, `None` for the target's own.
    pub parameters: Option<&'static [&'static str]>,
    /// Its result, `None` for the target's own.
    pub returns: Option<&'static str>,
}

/// A [`CxxMethod`] with the target's parameters and result.
const fn alias(name: &'static str, target: &'static str) -> CxxMethod {
    CxxMethod {
        name,
        target,
        parameters: None,
        returns: None,
    }
}

/// A [`CxxMethod`] with its own parameters and the target's result.
const fn respelled(
    name: &'static str,
    target: &'static str,
    parameters: &'static [&'static str],
) -> CxxMethod {
    CxxMethod {
        name,
        target,
        parameters: Some(parameters),
        returns: None,
    }
}

/// The thread context's convenience methods: short names for conversions,
/// string and array constructors, each overloaded by its parameters. Every
/// context that forwards to the thread context has them too.
pub const THREAD_ALIASES: &[CxxMethod] = &[
    alias("ObjectToCSelf", "ObjectToCSelfScoped"),
    alias("WholeNumber", "WholeNumberToObject"),
    alias("Uintptr", "UintptrToObject"),
    alias("Intptr", "IntptrToObject"),
    respelled("StringSize", "StringSizeToObject", &["usize"]),
    alias("WholeNumber", "ObjectToWholeNumber"),
    respelled(
        "StringSize",
        "ObjectToStringSize",
        &["RexxObjectPtr", "*mut usize"],
    ),
    alias("Int64", "Int64ToObject"),
    alias("UnsignedInt64", "UnsignedInt64ToObject"),
    alias("Int64", "ObjectToInt64"),
    alias("UnsignedInt64", "ObjectToUnsignedInt64"),
    alias("Int32", "Int32ToObject"),
    alias("UnsignedInt32", "UnsignedInt32ToObject"),
    alias("Int32", "ObjectToInt32"),
    alias("UnsignedInt32", "ObjectToUnsignedInt32"),
    alias("Uintptr", "ObjectToUintptr"),
    alias("Intptr", "ObjectToIntptr"),
    alias("Logical", "ObjectToLogical"),
    alias("Logical", "LogicalToObject"),
    alias("Double", "DoubleToObject"),
    alias("Double", "ObjectToDouble"),
    alias("CString", "ObjectToStringValue"),
    alias("String", "NewString"),
    alias("String", "NewStringFromAsciiz"),
    alias("CString", "NewStringFromAsciiz"),
    alias("Array", "ArrayOfOne"),
    alias("Array", "ArrayOfTwo"),
    alias("Array", "ArrayOfThree"),
    alias("Array", "ArrayOfFour"),
];

/// The thread context's methods of a member's own name whose parameters the
/// method spells otherwise than the member: `size_t` for `stringsize_t`, the
/// same C type, and a signed count where the member takes a size, which the
/// call converts.
pub const THREAD_RESPELLED: &[CxxMethod] = &[
    respelled("StringSizeToObject", "StringSizeToObject", &["usize"]),
    respelled(
        "ObjectToStringSize",
        "ObjectToStringSize",
        &["RexxObjectPtr", "*mut usize"],
    ),
    respelled("NewBuffer", "NewBuffer", &["wholenumber_t"]),
    respelled("NewMutableBuffer", "NewMutableBuffer", &["wholenumber_t"]),
];

/// The thread table's members no method of the member's own name calls: an
/// alias in [`THREAD_ALIASES`] reaches each.
pub const THREAD_UNNAMED: &[&str] = &["ObjectToCSelfScoped"];

/// The thread context's accessors of the constant objects: each method's name
/// and the value member of the thread table it reads.
pub const THREAD_VALUE_ACCESSORS: &[(&str, &str)] = &[
    ("Nil", "RexxNil"),
    ("True", "RexxTrue"),
    ("False", "RexxFalse"),
    ("NullString", "RexxNullString"),
];

/// The instance table's members the thread context forwards to its
/// `instance`, under the member's own name.
pub const THREAD_INSTANCE_FORWARDS: &[&str] = &[
    "InterpreterVersion",
    "LanguageLevel",
    "AddCommandEnvironment",
    "StackCommandEnvironment",
];

/// The thread context's accessors of a field of its instance: the method's
/// name, the field's C name and the method's result.
pub const THREAD_INSTANCE_FIELDS: &[(&str, &str, &str)] =
    &[("GetApplicationData", "applicationData", "POINTER")];

/// The thread context's methods the method, call and exit contexts do not
/// forward: those that act on the thread itself.
pub const THREAD_ONLY: &[&str] = &["DetachThread", "HaltThread", "SetThreadTrace"];

/// The thread context's methods that the method, call and exit contexts
/// forward with another result: `DisplayCondition` answers the error number
/// there as a `logical_t`.
pub const FORWARDED_RESPELLED: &[CxxMethod] = &[CxxMethod {
    name: "DisplayCondition",
    target: "DisplayCondition",
    parameters: None,
    returns: Some("logical_t"),
}];

/// The method-context table an extension calls through, at one address.
pub static METHOD_CONTEXT_INTERFACE: MethodContextInterface = MethodContextInterface::REFUSING;

// The thread table has no `static` beside it: its object members are raw
// pointers, so the type is not `Sync`, and they are filled once the constant
// objects exist. Whatever hands out a thread context owns the table, which is
// also what keeps this phase from mutating a global.
