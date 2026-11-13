// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `#[repr(C)]` surface measured by a C and a C++ compiler.
//!
//! The test writes a dump program from the Rust definitions (sizes, alignments, offsets, member
//! types and bit positions), compiles it as C and as C++ against the API headers, runs it and
//! compares what the compiler measured with what Rust reports. The headers are read by the
//! compiler alone: nothing here parses their text.
//!
//! The dump compiles against the repository's own headers (`include/`, generated from the same
//! Rust definitions by `src/header.rs`), so the generator is checked against Rust. The tests named
//! `..._upstream_headers_...` compile it against the pinned upstream tree through `REXX_UPSTREAM`
//! instead, which makes them source-tier tests: they check that the Rust layout, and so the
//! generated headers, keep the upstream interface.
//!
//! The Rust-to-C type mapping below keys on `std::any::type_name`, so `usize` stands for
//! `size_t`, `stringsize_t` and `logical_t` alike; they are one C type on this platform, and
//! the dump's typedef facts check each of them separately.

use interrexx_api::layout::{
    CONSTRXSTRING, CallContextInterface, EXIT_FLAG_BITS, ExitContextInterface, ExitFlags,
    IORedirectorInterface, MethodContextInterface, REXXDATETIME, RXCMDHST_PARM, RXFNCCAL_PARM,
    RXMSQPSH_PARM, RXOFNCCAL_PARM, RXSIOSAY_PARM, RXSTRING, RXSYSEXIT, RXVALCALL_PARM,
    RXVARNOVALUE_PARM, RexxCallContext_, RexxCondition, RexxContextExit, RexxExitContext_,
    RexxIORedirectorContext_, RexxInstance_, RexxInstanceInterface, RexxMethodContext_,
    RexxMethodEntry, RexxPackageEntry, RexxQueueTime, RexxReturnCode, RexxRoutineEntry,
    RexxThreadContext_, RexxThreadInterface, SHVBLOCK, ValueDescriptor, ValueUnion, logical_t,
    stringsize_t, wholenumber_t,
};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::mem::{align_of, offset_of, size_of};
use std::path::{Path, PathBuf};
use std::process::Command;

/// One line of the dump: what was measured, and the value.
type Facts = BTreeMap<String, String>;

/// A language the dump is compiled as.
#[derive(Clone, Copy)]
enum Language {
    C,
    Cxx,
}

impl Language {
    fn name(self) -> &'static str {
        match self {
            Language::C => "c",
            Language::Cxx => "c++",
        }
    }

    fn compiler(self) -> &'static str {
        match self {
            Language::C => "cc",
            Language::Cxx => "c++",
        }
    }

    fn source_name(self) -> &'static str {
        match self {
            Language::C => "dump.c",
            Language::Cxx => "dump.cc",
        }
    }
}

/// The header set a dump compiles against.
#[derive(Clone, Copy)]
enum Headers {
    /// The repository's `include/`.
    Ours,
    /// The pinned upstream tree's `api/`, `$REXX_UPSTREAM/api`.
    Upstream,
}

impl Headers {
    /// The directories to put on the include path: the headers and this platform's headers.
    fn dirs(self) -> [PathBuf; 2] {
        let api = match self {
            Headers::Ours => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../include"),
            Headers::Upstream => {
                let upstream = std::env::var_os("REXX_UPSTREAM").expect(
                    "REXX_UPSTREAM names the upstream tree whose api/ directory the dump \
                     compiles against",
                );
                PathBuf::from(upstream).join("api")
            }
        };
        assert!(api.is_dir(), "{} is not a directory", api.display());
        let platform = api.join("platform/unix");
        [api, platform]
    }

    fn name(self) -> &'static str {
        match self {
            Headers::Ours => "include",
            Headers::Upstream => "upstream",
        }
    }
}

/// A scratch directory for one test's sources and binaries.
fn scratch(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("abi_dump")
        .join(test);
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
    dir
}

/// Compiles `source` as `language` into `dir`, against `headers` if given, runs it and answers its
/// standard output.
fn compile_and_run(
    language: Language,
    dir: &Path,
    source: &str,
    headers: Option<Headers>,
) -> String {
    let path = dir.join(language.source_name());
    std::fs::write(&path, source).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let program = dir.join(format!("dump-{}", language.name().replace('+', "x")));
    let mut command = Command::new(language.compiler());
    command.arg("-Wall");
    if matches!(language, Language::Cxx) {
        command.arg("-std=c++17");
    }
    if let Some(headers) = headers {
        for dir in headers.dirs() {
            command.arg("-I").arg(dir);
        }
    }
    let built = command
        .arg(&path)
        .arg("-o")
        .arg(&program)
        .output()
        .unwrap_or_else(|e| panic!("{}: {e}", language.compiler()));
    assert!(
        built.status.success(),
        "the dump does not compile as {}:\n{}",
        language.name(),
        String::from_utf8_lossy(&built.stderr)
    );
    let run = Command::new(&program)
        .output()
        .unwrap_or_else(|e| panic!("{}: {e}", program.display()));
    assert!(run.status.success(), "{} failed", program.display());
    String::from_utf8(run.stdout).expect("the dump prints text")
}

/// The preamble both languages share: the headers, the printing helpers and `SAME`, which is `1`
/// when the type of an expression is the given type and `0` otherwise (`typeid` in C++, which
/// ignores language linkage; `_Generic` in C).
const PREAMBLE: &str = r#"
#include <stdio.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>
#include <sys/types.h>
#ifdef __cplusplus
#include <typeinfo>
#define ALIGNOF(T) alignof(T)
#define SAME(EXPR, ...) (typeid(decltype(EXPR)) == typeid(__VA_ARGS__) ? 1 : 0)
#else
#define ALIGNOF(T) _Alignof(T)
#define SAME(EXPR, ...) _Generic((EXPR), __VA_ARGS__: 1, default: 0)
#endif
"#;

/// The headers the dump includes.
const HEADERS: &str = "#include \"rexx.h\"\n#include \"oorexxapi.h\"\n";

/// The C source line that reports the bit a one-bit field occupies in its struct: the position of
/// the only bit set once the field is assigned 1, or -1 if none or several are.
const BIT_HELPER: &str = r#"
static int bit_of(const unsigned char *bytes, size_t size) {
    int found = -1;
    for (size_t at = 0; at < size * 8; at++) {
        if (bytes[at / 8] & (1u << (at % 8))) {
            if (found != -1) return -1;
            found = (int)at;
        }
    }
    return found;
}
"#;

/// The fields of one struct as Rust lays them out.
struct Field {
    c_name: &'static str,
    offset: usize,
    size: usize,
}

/// A struct as Rust lays it out, under the name the header gives it.
struct Struct {
    c_name: &'static str,
    size: usize,
    align: usize,
    fields: Vec<Field>,
}

/// The size of a field of `T` without a value to measure it on.
fn size_of_field<T, F>(_field: fn(&T) -> &F) -> usize {
    size_of::<F>()
}

/// A [`Struct`] from a Rust type, its C name and its fields, each named as Rust names it and,
/// where C spells it differently, `=> "name"`.
macro_rules! strukt {
    ($c:literal = $t:ty { $($f:ident $(=> $cf:literal)?),* $(,)? }) => {
        Struct {
            c_name: $c,
            size: size_of::<$t>(),
            align: align_of::<$t>(),
            fields: vec![$(Field {
                c_name: strukt!(@name $f $($cf)?),
                offset: offset_of!($t, $f),
                size: size_of_field::<$t, _>(|s| &s.$f),
            }),*],
        }
    };
    (@name $f:ident) => { stringify!($f) };
    (@name $f:ident $cf:literal) => { $cf };
}

/// Every `repr(C)` struct that has a header twin.
///
/// The header's `RXEXFCAL_PARM` and `RXSIOTRC_PARM` share the layout of `RXOFNCCAL_PARM` and
/// `RXSIOSAY_PARM`. The blocks of a lone `RXSTRING` or `size_t` member have no Rust type.
fn structs() -> Vec<Struct> {
    vec![
        strukt!("RXSTRING" = RXSTRING { strlength, strptr }),
        strukt!("CONSTRXSTRING" = CONSTRXSTRING { strlength, strptr }),
        strukt!(
            "RXSYSEXIT" = RXSYSEXIT {
                sysexit_name,
                sysexit_code
            }
        ),
        strukt!(
            "RexxContextExit" = RexxContextExit {
                handler,
                sysexit_code
            }
        ),
        strukt!(
            "RXFNCCAL_PARM" = RXFNCCAL_PARM {
                rxfnc_flags,
                rxfnc_name,
                rxfnc_namel,
                rxfnc_que,
                rxfnc_quel,
                rxfnc_argc,
                rxfnc_argv,
                rxfnc_retc
            }
        ),
        strukt!(
            "RXOFNCCAL_PARM" = RXOFNCCAL_PARM {
                rxfnc_flags,
                rxfnc_name,
                rxfnc_argc,
                rxfnc_argv,
                rxfnc_retc
            }
        ),
        strukt!(
            "RXEXFCAL_PARM" = RXOFNCCAL_PARM {
                rxfnc_flags,
                rxfnc_name,
                rxfnc_argc,
                rxfnc_argv,
                rxfnc_retc
            }
        ),
        strukt!(
            "RXCMDHST_PARM" = RXCMDHST_PARM {
                rxcmd_flags,
                rxcmd_address,
                rxcmd_addressl,
                rxcmd_dll,
                rxcmd_dll_len,
                rxcmd_command,
                rxcmd_retc
            }
        ),
        strukt!(
            "RXMSQPSH_PARM" = RXMSQPSH_PARM {
                rxmsq_flags,
                rxmsq_value
            }
        ),
        strukt!("RXSIOSAY_PARM" = RXSIOSAY_PARM { rxsio_string }),
        strukt!("RXSIOTRC_PARM" = RXSIOSAY_PARM { rxsio_string }),
        strukt!(
            "RXVALCALL_PARM" = RXVALCALL_PARM {
                selector,
                variable_name,
                value
            }
        ),
        strukt!(
            "RXVARNOVALUE_PARM" = RXVARNOVALUE_PARM {
                variable_name,
                value
            }
        ),
        strukt!(
            "SHVBLOCK" = SHVBLOCK {
                shvnext,
                shvname,
                shvvalue,
                shvnamelen,
                shvvaluelen,
                shvcode,
                shvret
            }
        ),
        strukt!(
            "RexxQueueTime" = RexxQueueTime {
                hours,
                minutes,
                seconds,
                hundredths,
                day,
                month,
                year,
                weekday,
                microseconds,
                yearday
            }
        ),
        strukt!(
            "REXXDATETIME" = REXXDATETIME {
                hours,
                minutes,
                seconds,
                hundredths,
                day,
                month,
                year,
                weekday,
                microseconds,
                yearday,
                valid
            }
        ),
        strukt!(
            "RexxCondition" = RexxCondition {
                code,
                rc,
                position,
                conditionName,
                message,
                errortext,
                program,
                description,
                additional
            }
        ),
        strukt!(
            "ValueDescriptor" = ValueDescriptor {
                value,
                r#type => "type",
                flags
            }
        ),
        strukt!(
            "RexxRoutineEntry" = RexxRoutineEntry {
                style,
                reserved1,
                name,
                entry_point => "entryPoint",
                reserved2,
                reserved3
            }
        ),
        strukt!(
            "RexxMethodEntry" = RexxMethodEntry {
                style,
                reserved1,
                name,
                entry_point => "entryPoint",
                reserved2,
                reserved3
            }
        ),
        strukt!(
            "RexxPackageEntry" = RexxPackageEntry {
                size,
                api_version => "apiVersion",
                required_version => "requiredVersion",
                package_name => "packageName",
                package_version => "packageVersion",
                loader,
                unloader,
                routines,
                methods
            }
        ),
        strukt!(
            "struct RexxInstance_" = RexxInstance_ {
                functions,
                applicationData
            }
        ),
        strukt!(
            "struct RexxThreadContext_" = RexxThreadContext_ {
                instance,
                functions
            }
        ),
        strukt!(
            "struct RexxMethodContext_" = RexxMethodContext_ {
                threadContext,
                functions,
                arguments
            }
        ),
        strukt!(
            "struct RexxCallContext_" = RexxCallContext_ {
                threadContext,
                functions,
                arguments
            }
        ),
        strukt!(
            "struct RexxExitContext_" = RexxExitContext_ {
                threadContext,
                functions,
                arguments
            }
        ),
        strukt!("struct RexxIORedirectorContext_" = RexxIORedirectorContext_ { functions }),
    ]
}

/// A typedef and the width, alignment and (for an integer) signedness Rust gives it.
struct Typedef {
    c_name: &'static str,
    size: usize,
    align: usize,
    signed: Option<bool>,
}

/// A [`Typedef`] for a pointer, which has no signedness.
macro_rules! pointer_typedef {
    ($c:literal, $t:ty) => {
        Typedef {
            c_name: $c,
            size: size_of::<$t>(),
            align: align_of::<$t>(),
            signed: None,
        }
    };
}

/// A [`Typedef`] for an integer.
macro_rules! integer_typedef {
    ($c:literal, $t:ty) => {
        Typedef {
            c_name: $c,
            size: size_of::<$t>(),
            align: align_of::<$t>(),
            signed: Some(<$t>::MIN != 0),
        }
    };
}

/// Every scalar and handle typedef the Rust definitions carry.
fn typedefs() -> Vec<Typedef> {
    use interrexx_api::layout::{
        CSTRING, POINTER, REXXPFN, RexxArrayObject, RexxBufferObject, RexxBufferStringObject,
        RexxClassObject, RexxDirectoryObject, RexxMethodObject, RexxMutableBufferObject,
        RexxObjectPtr, RexxPackageObject, RexxPointerObject, RexxRoutineObject, RexxStemObject,
        RexxStringObject, RexxStringTableObject, RexxSupplierObject, RexxVariableReferenceObject,
    };
    vec![
        integer_typedef!("wholenumber_t", wholenumber_t),
        integer_typedef!("stringsize_t", stringsize_t),
        integer_typedef!("logical_t", logical_t),
        integer_typedef!("RexxReturnCode", RexxReturnCode),
        pointer_typedef!("CSTRING", CSTRING),
        pointer_typedef!("POINTER", POINTER),
        pointer_typedef!("REXXPFN", REXXPFN),
        pointer_typedef!("RexxObjectPtr", RexxObjectPtr),
        pointer_typedef!("RexxStringObject", RexxStringObject),
        pointer_typedef!("RexxBufferStringObject", RexxBufferStringObject),
        pointer_typedef!("RexxArrayObject", RexxArrayObject),
        pointer_typedef!("RexxBufferObject", RexxBufferObject),
        pointer_typedef!("RexxPointerObject", RexxPointerObject),
        pointer_typedef!("RexxMethodObject", RexxMethodObject),
        pointer_typedef!("RexxRoutineObject", RexxRoutineObject),
        pointer_typedef!("RexxPackageObject", RexxPackageObject),
        pointer_typedef!("RexxClassObject", RexxClassObject),
        pointer_typedef!("RexxDirectoryObject", RexxDirectoryObject),
        pointer_typedef!("RexxStringTableObject", RexxStringTableObject),
        pointer_typedef!("RexxSupplierObject", RexxSupplierObject),
        pointer_typedef!("RexxStemObject", RexxStemObject),
        pointer_typedef!("RexxMutableBufferObject", RexxMutableBufferObject),
        pointer_typedef!("RexxVariableReferenceObject", RexxVariableReferenceObject),
    ]
}

/// An interface table: its header name and, for each member, its name and Rust type.
struct Table {
    c_name: &'static str,
    members: Vec<(&'static str, &'static str)>,
}

fn tables() -> Vec<Table> {
    macro_rules! table {
        ($t:ty) => {
            Table {
                c_name: stringify!($t),
                members: <$t>::FIELDS
                    .iter()
                    .copied()
                    .zip(<$t>::member_types())
                    .collect(),
            }
        };
    }
    vec![
        table!(RexxInstanceInterface),
        table!(RexxThreadInterface),
        table!(MethodContextInterface),
        table!(CallContextInterface),
        table!(ExitContextInterface),
        table!(IORedirectorInterface),
    ]
}

/// Drops every module path from `type_name`'s spelling.
fn unqualified(type_name: &str) -> String {
    let mut out = String::new();
    let mut word = String::new();
    let mut chars = type_name.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            word.push(ch);
        } else if ch == ':' && chars.peek() == Some(&':') {
            chars.next();
            word.clear();
        } else {
            out.push_str(&word);
            word.clear();
            out.push(ch);
        }
    }
    out.push_str(&word);
    out
}

/// The header's C type for a Rust type spelled as `type_name` spells it, once module paths are
/// dropped.
///
/// A pointer to a handle struct is the handle typedef (the handle is already a pointer); a
/// pointer to a context or instance struct is its typedef name and a star. In C the typedef is
/// itself a pointer (the compiler reports `sizeof` 8) and in C++ it is the struct, so the same
/// spelling is a different type in each language and a pointer at the same place in both.
fn c_type(rust: &str) -> String {
    if let Some(pointee) = rust.strip_prefix("*mut ") {
        if let Some(handle) = pointee.strip_suffix('_').filter(|name| is_handle(name)) {
            return handle.to_string();
        }
        return pointer_to(&c_type(pointee), false);
    }
    if let Some(pointee) = rust.strip_prefix("*const ") {
        return pointer_to(&c_type(pointee), true);
    }
    match rust {
        "i8" => "int8_t".to_string(),
        "u8" => "uint8_t".to_string(),
        "i16" => "int16_t".to_string(),
        "u16" => "uint16_t".to_string(),
        "i32" => "int32_t".to_string(),
        "u32" => "uint32_t".to_string(),
        "i64" => "int64_t".to_string(),
        "u64" => "uint64_t".to_string(),
        "isize" => "ssize_t".to_string(),
        "usize" => "size_t".to_string(),
        "f32" => "float".to_string(),
        "f64" => "double".to_string(),
        "c_void" => "void".to_string(),
        "RexxCondition" | "ValueDescriptor" | "RexxPackageEntry" => rust.to_string(),
        "RexxInstance_"
        | "RexxThreadContext_"
        | "RexxMethodContext_"
        | "RexxCallContext_"
        | "RexxExitContext_"
        | "RexxIORedirectorContext_" => rust
            .strip_suffix('_')
            .expect("a trailing underscore")
            .to_string(),
        other => panic!("{other} is not a type this test maps to C"),
    }
}

/// Whether `name` (without its trailing underscore) names one of the opaque object handles.
fn is_handle(name: &str) -> bool {
    name.starts_with("Rexx") && (name.ends_with("Object") || name.ends_with("ObjectPtr"))
}

/// `pointee` behind a pointer. A character is `char` behind a pointer, where C's `char` is the
/// type of a string, and `int8_t` or `uint8_t` elsewhere.
fn pointer_to(pointee: &str, constant: bool) -> String {
    let base = match pointee {
        "int8_t" | "uint8_t" => "char",
        other => other,
    };
    let qualified = if constant {
        format!("const {base}")
    } else {
        base.to_string()
    };
    if qualified.ends_with('*') {
        format!("{qualified}*")
    } else {
        format!("{qualified} *")
    }
}

/// The C type of one table member: a function pointer's `RET (*)(ARGS)` or a data member's type.
fn c_member_type(rust: &str) -> String {
    let rust = unqualified(rust);
    let Some(rest) = rust
        .strip_prefix("unsafe extern \"C\" fn(")
        .or_else(|| rust.strip_prefix("unsafe extern \"C-unwind\" fn("))
    else {
        return c_type(&rust);
    };
    let mut depth = 1;
    let mut close = None;
    for (at, ch) in rest.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(at);
                    break;
                }
            }
            _ => {}
        }
    }
    let close = close.expect("the parameter list closes");
    let arguments: Vec<String> = rest[..close]
        .split(", ")
        .filter(|argument| !argument.is_empty())
        .map(c_type)
        .collect();
    let returns = match rest[close + 1..].trim().strip_prefix("-> ") {
        Some(ty) => c_type(ty),
        None => "void".to_string(),
    };
    let arguments = if arguments.is_empty() {
        "void".to_string()
    } else {
        arguments.join(", ")
    };
    format!("{returns} (*)({arguments})")
}

/// The dump program for the current Rust definitions.
fn dump_program() -> String {
    let mut out = String::new();
    out.push_str(PREAMBLE);
    out.push_str(HEADERS);
    out.push_str(BIT_HELPER);
    out.push_str("int main(void) {\n");
    for s in structs() {
        let name = s.c_name;
        writeln!(
            out,
            "  printf(\"layout {name} size\\t%zu\\n\", sizeof({name}));"
        )
        .unwrap();
        writeln!(
            out,
            "  printf(\"layout {name} align\\t%zu\\n\", (size_t)ALIGNOF({name}));"
        )
        .unwrap();
        for f in &s.fields {
            let field = f.c_name;
            writeln!(
                out,
                "  printf(\"field {name}.{field} offset\\t%zu\\n\", offsetof({name}, {field}));"
            )
            .unwrap();
            writeln!(
                out,
                "  printf(\"field {name}.{field} size\\t%zu\\n\", sizeof(((({name} *)0)->{field})));"
            )
            .unwrap();
        }
    }
    for (name, members) in EXIT_FLAG_BITS {
        writeln!(
            out,
            "  printf(\"layout {name} size\\t%zu\\n\", sizeof({name}));"
        )
        .unwrap();
        writeln!(
            out,
            "  printf(\"layout {name} align\\t%zu\\n\", (size_t)ALIGNOF({name}));"
        )
        .unwrap();
        for (member, _) in *members {
            writeln!(
                out,
                "  {{ {name} s; memset(&s, 0, sizeof s); s.{member} = 1; \
                 printf(\"bit {name}.{member}\\t%d\\n\", bit_of((const unsigned char *)&s, sizeof s)); }}"
            )
            .unwrap();
        }
    }
    for t in typedefs() {
        let name = t.c_name;
        writeln!(
            out,
            "  printf(\"typedef {name} size\\t%zu\\n\", sizeof({name}));"
        )
        .unwrap();
        writeln!(
            out,
            "  printf(\"typedef {name} align\\t%zu\\n\", (size_t)ALIGNOF({name}));"
        )
        .unwrap();
        if t.signed.is_some() {
            writeln!(
                out,
                "  printf(\"typedef {name} signed\\t%d\\n\", (({name})-1) < 0 ? 1 : 0);"
            )
            .unwrap();
        }
    }
    for table in tables() {
        let name = table.c_name;
        writeln!(
            out,
            "  printf(\"layout {name} size\\t%zu\\n\", sizeof({name}));"
        )
        .unwrap();
        for (member, rust) in &table.members {
            let expected = c_member_type(rust);
            writeln!(
                out,
                "  printf(\"field {name}.{member} offset\\t%zu\\n\", offsetof({name}, {member}));"
            )
            .unwrap();
            writeln!(
                out,
                "  printf(\"type {name}.{member}\\t%d\\n\", SAME((({name} *)0)->{member}, {expected}));"
            )
            .unwrap();
        }
    }
    for (member, rust) in ValueUnion::member_types() {
        let expected = c_member_type(rust);
        writeln!(
            out,
            "  printf(\"type ValueDescriptor.value.{member}\\t%d\\n\", SAME(((ValueDescriptor *)0)->value.{member}, {expected}));"
        )
        .unwrap();
    }
    out.push_str("  return 0;\n}\n");
    out
}

/// The facts Rust reports about the same things, in the dump's keys.
fn rust_facts() -> Facts {
    let mut facts = Facts::new();
    let mut put = |key: String, value: String| {
        facts.insert(key, value);
    };
    for s in structs() {
        put(format!("layout {} size", s.c_name), s.size.to_string());
        put(format!("layout {} align", s.c_name), s.align.to_string());
        for f in &s.fields {
            put(
                format!("field {}.{} offset", s.c_name, f.c_name),
                f.offset.to_string(),
            );
            put(
                format!("field {}.{} size", s.c_name, f.c_name),
                f.size.to_string(),
            );
        }
    }
    for (name, members) in EXIT_FLAG_BITS {
        put(
            format!("layout {name} size"),
            size_of::<ExitFlags>().to_string(),
        );
        put(
            format!("layout {name} align"),
            align_of::<ExitFlags>().to_string(),
        );
        for (member, bit) in *members {
            put(format!("bit {name}.{member}"), bit.to_string());
        }
    }
    for t in typedefs() {
        put(format!("typedef {} size", t.c_name), t.size.to_string());
        put(format!("typedef {} align", t.c_name), t.align.to_string());
        if let Some(signed) = t.signed {
            put(
                format!("typedef {} signed", t.c_name),
                u8::from(signed).to_string(),
            );
        }
    }
    for table in tables() {
        put(
            format!("layout {} size", table.c_name),
            (table.members.len() * size_of::<usize>()).to_string(),
        );
        // Every member is one word, which `tests/layout.rs` checks, so a member's offset is its
        // position in the table.
        for (at, (member, _)) in table.members.iter().enumerate() {
            put(
                format!("field {}.{member} offset", table.c_name),
                (at * size_of::<usize>()).to_string(),
            );
            put(format!("type {}.{member}", table.c_name), "1".to_string());
        }
    }
    for (member, _) in ValueUnion::member_types() {
        put(
            format!("type ValueDescriptor.value.{member}"),
            "1".to_string(),
        );
    }
    facts
}

/// Parses the dump's output.
fn parse(output: &str) -> Facts {
    output
        .lines()
        .map(|line| {
            let (key, value) = line
                .split_once('\t')
                .unwrap_or_else(|| panic!("{line:?} is not a fact"));
            (key.to_string(), value.to_string())
        })
        .collect()
}

/// Each fact on which `measured` and `expected` differ, as a line for a failure message.
fn differences(measured: &Facts, expected: &Facts) -> Vec<String> {
    let mut found = Vec::new();
    for (key, value) in expected {
        match measured.get(key) {
            Some(got) if got == value => {}
            Some(got) => found.push(format!("{key}: the compiler says {got}, Rust says {value}")),
            None => found.push(format!("{key}: the compiler reported nothing")),
        }
    }
    for key in measured.keys().filter(|key| !expected.contains_key(*key)) {
        found.push(format!("{key}: Rust reports nothing"));
    }
    found
}

fn check(language: Language, headers: Headers) {
    let dir = scratch(&format!(
        "{}-{}",
        headers.name(),
        language.name().replace('+', "x")
    ));
    let measured = parse(&compile_and_run(
        language,
        &dir,
        &dump_program(),
        Some(headers),
    ));
    let found = differences(&measured, &rust_facts());
    assert!(
        found.is_empty(),
        "the Rust layout differs from the {} headers as {} sees them:\n{}",
        headers.name(),
        language.name(),
        found.join("\n")
    );
}

/// **The Rust surface is the header's, as a C compiler measures it**: every struct's size,
/// alignment, field offsets and field sizes, every exit flag's bit, every typedef's width,
/// alignment and signedness, and every table member's type, measured on `include/`.
#[test]
fn the_rust_layout_is_the_headers_as_c_measures_it() {
    check(Language::C, Headers::Ours);
}

/// The same facts, as a C++ compiler measures them (the types by `typeid`).
#[test]
fn the_rust_layout_is_the_headers_as_cxx_measures_it() {
    check(Language::Cxx, Headers::Ours);
}

/// The same facts measured on the upstream headers, as C.
#[test]
fn the_rust_layout_is_the_upstream_headers_as_c_measures_it() {
    check(Language::C, Headers::Upstream);
}

/// The same facts measured on the upstream headers, as C++.
#[test]
fn the_rust_layout_is_the_upstream_headers_as_cxx_measures_it() {
    check(Language::Cxx, Headers::Upstream);
}

/// The dump covers what it claims to: a struct from every group, the typedefs that share a Rust
/// type, a function member, a data member and an exit flag.
#[test]
fn the_dump_names_what_it_measures() {
    let facts = rust_facts();
    for key in [
        "layout RexxCondition size",
        "field RexxPackageEntry.apiVersion offset",
        "field ValueDescriptor.type offset",
        "bit RXFNC_FLAGS.rxffsub",
        "typedef wholenumber_t signed",
        "typedef stringsize_t signed",
        "typedef logical_t size",
        "type RexxThreadInterface.SendMessage",
        "type RexxInstanceInterface.interfaceVersion",
        "type ValueDescriptor.value.value_int",
    ] {
        assert!(facts.contains_key(key), "the dump does not measure {key}");
    }
    assert_eq!(facts["typedef wholenumber_t signed"], "1");
    assert_eq!(facts["typedef stringsize_t signed"], "0");
}

/// **A wrong Rust layout is reported**: a pair of members swapped in a table changes the
/// comparison. The mutation is applied to the facts, which are what the compiler's output is
/// compared with.
#[test]
fn a_swapped_pair_of_members_is_reported() {
    let dir = scratch("control-swap");
    let measured = parse(&compile_and_run(
        Language::C,
        &dir,
        &dump_program(),
        Some(Headers::Ours),
    ));
    let mut mutated = rust_facts();
    let first = "field RexxCondition.conditionName offset".to_string();
    let second = "field RexxCondition.message offset".to_string();
    let a = mutated[&first].clone();
    let b = mutated[&second].clone();
    mutated.insert(first.clone(), b);
    mutated.insert(second.clone(), a);
    let found = differences(&measured, &mutated);
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(found[0].contains("RexxCondition"), "{found:?}");
    // A changed typedef width is reported the same way.
    let mut mutated = rust_facts();
    mutated.insert("typedef logical_t size".to_string(), "4".to_string());
    assert_eq!(differences(&measured, &mutated).len(), 1);
}

/// **The type comparison tells types apart**: `SAME` answers `1` for the type given and `0` for a
/// different width, signedness, pointee or argument list, in each language.
#[test]
fn the_type_check_tells_types_apart() {
    let program = format!(
        "{PREAMBLE}
int main(void) {{
  int32_t i = 0; uint32_t u = 0; int64_t l = 0; const char *s = 0; void *p = 0;
  void (*f)(int32_t, size_t) = 0;
  printf(\"same\\t%d\\n\", SAME(i, int32_t));
  printf(\"width\\t%d\\n\", SAME(i, int64_t));
  printf(\"sign\\t%d\\n\", SAME(i, uint32_t));
  printf(\"wide\\t%d\\n\", SAME(l, int64_t));
  printf(\"pointee\\t%d\\n\", SAME(s, char *));
  printf(\"pointer\\t%d\\n\", SAME(p, void *));
  printf(\"function\\t%d\\n\", SAME(f, void (*)(int32_t, size_t)));
  printf(\"arguments\\t%d\\n\", SAME(f, void (*)(int32_t, int32_t)));
  printf(\"result\\t%d\\n\", SAME(f, int32_t (*)(int32_t, size_t)));
  (void)u;
  return 0;
}}
"
    );
    for language in [Language::C, Language::Cxx] {
        let dir = scratch(&format!(
            "control-types-{}",
            language.name().replace('+', "x")
        ));
        let facts = parse(&compile_and_run(language, &dir, &program, None));
        for (key, expected) in [
            ("same", "1"),
            ("width", "0"),
            ("sign", "0"),
            ("wide", "1"),
            ("pointee", "0"),
            ("pointer", "1"),
            ("function", "1"),
            ("arguments", "0"),
            ("result", "0"),
        ] {
            assert_eq!(facts[key], expected, "{key} as {}", language.name());
        }
    }
}

/// The mapping from Rust types to C types gives what a table member needs.
#[test]
fn the_type_mapping_spells_a_function_member() {
    assert_eq!(
        c_member_type(
            "unsafe extern \"C\" fn(*mut interrexx_api::layout::RexxThreadContext_, *mut *mut interrexx_api::layout::RexxThreadContext_, *const i8, usize) -> *mut interrexx_api::layout::RexxObjectPtr_"
        ),
        "RexxObjectPtr (*)(RexxThreadContext *, RexxThreadContext **, const char *, size_t)"
    );
    assert_eq!(
        c_member_type("unsafe extern \"C-unwind\" fn(usize)"),
        "void (*)(size_t)"
    );
    assert_eq!(c_member_type("isize"), "ssize_t");
    assert_eq!(c_member_type("*mut core::ffi::c_void"), "void *");
}
