//! The C declarations of `include/`, generated from the Rust definitions in
//! `layout.rs`.
//!
//! Each header in `include/` is hand-written text around generated sections.
//! A section opens with the line `/* BEGIN GENERATED <name> */` and closes with
//! `/* END GENERATED <name> */`; [`splice`] replaces what lies between with what
//! [`sections`] renders. The test `headers_are_current` (`tests/headers.rs`)
//! checks every file, and with `REXX_HEADERS_REFRESH=1` writes it.
//!
//! Every type a generated declaration names comes from the tokens of the Rust
//! type through [`c_type`], never from a C string kept beside the Rust one,
//! so a change to a Rust definition changes the header and the ABI dump
//! (`tests/abi_dump.rs`) measures the two against each other.
//!
//! The constants come from `constants.rs`, the exports from `exports.rs`, and
//! the error codes of `oorexxerrors.h` from the error table
//! (`interrexx-inventory/data/errors.tsv`), whose rows [`sections`] takes.

use crate::constants::{
    API_VERSIONS, ARGUMENT_TYPEDEFS, ARGUMENT_TYPES, CConstant, CLASSIC_CONSTANTS, CLASSIC_LIMITS,
    ENTRY_CONSTANTS, ENVIRONMENT_KINDS, OPTION_NAMES, error_code, has_error_code,
    optional_argument_types, value_codes,
};
use crate::exports::{Declared, EXPORTS, Export, pointer_types};
use crate::layout::{
    API_HANDLERS, C_HANDLERS, C_POINTER_TYPEDEFS, C_TYPEDEFS, CField, CMember, CONSTRXSTRING,
    CSpelling, CallContextInterface, CxxMethod, EXIT_FLAG_BITS, ExitContextInterface,
    FORWARDED_RESPELLED, HANDLES, IORedirectorInterface, MethodContextInterface, REXXDATETIME,
    RXCMDHST_PARM, RXFNCCAL_PARM, RXHLTTST_PARM, RXMSQNAM_PARM, RXMSQPLL_PARM, RXMSQPSH_PARM,
    RXMSQSIZ_PARM, RXOFNCCAL_PARM, RXSIODTR_PARM, RXSIOSAY_PARM, RXSIOTRD_PARM, RXSTRING,
    RXSYSEXIT, RXTRCTST_PARM, RXVALCALL_PARM, RXVARNOVALUE_PARM, RexxCallContext_, RexxCondition,
    RexxConditionData, RexxContextEnvironment, RexxContextExit, RexxExitContext_,
    RexxIORedirectorContext_, RexxInstance_, RexxInstanceInterface, RexxLibraryPackage,
    RexxMethodContext_, RexxMethodEntry, RexxOption, RexxPackageEntry, RexxQueueTime,
    RexxRedirectingEnvironment, RexxRegisteredEnvironment, RexxRoutineEntry, RexxThreadContext_,
    RexxThreadInterface, SHVBLOCK, THREAD_ALIASES, THREAD_INSTANCE_FIELDS,
    THREAD_INSTANCE_FORWARDS, THREAD_ONLY, THREAD_RESPELLED, THREAD_UNNAMED,
    THREAD_VALUE_ACCESSORS, VALUE_ASSIGNMENTS, ValueDescriptor, ValueUnion, c_type,
};
use crate::values::rows;
use std::fmt::Write as _;

/// The directory of the platform headers of each platform, under `include/`.
pub const PLATFORM_DIR: &str = "platform";

/// What differs between platforms in the header set: the declarations a
/// platform makes in its own directory (`include/platform/<name>/`), which the
/// top-level headers include by bare name.
pub struct Platform {
    /// The directory name.
    pub name: &'static str,
    /// The system headers the platform's declarations rest on.
    pub system_headers: &'static [&'static str],
    /// Each platform typedef: its name and its C type.
    pub typedefs: &'static [(&'static str, &'static str)],
    /// The structs whose layout is this platform's: on another platform the
    /// name may stand for a system type.
    pub structs: &'static [Struct],
    /// Each calling-convention and export macro: its name and its expansion.
    pub defines: &'static [(&'static str, &'static str)],
}

/// The platforms the header set is written for.
pub const PLATFORMS: &[Platform] = &[Platform {
    name: "unix",
    system_headers: &[
        "stdlib.h",
        "stddef.h",
        "stdint.h",
        "limits.h",
        "unistd.h",
        "sys/types.h",
        "pthread.h",
    ],
    typedefs: &[
        ("REXXPFN", "void *"),
        ("process_id_t", "pid_t"),
        ("thread_id_t", "pthread_t"),
    ],
    structs: &[Struct {
        name: "REXXDATETIME",
        tag: Tag::Named("_REXXDATETIME"),
        fields: REXXDATETIME::C_FIELDS,
        retype: &[],
        assignments: &[],
    }],
    defines: &[("REXXENTRY", ""), ("REXXEXPORT", "")],
}];

/// How a struct is introduced.
#[derive(Clone, Copy)]
pub enum Tag {
    /// `struct <tag>`, with a typedef of the struct's name declared ahead of
    /// every definition, so a struct may point at itself or at a later one.
    Named(&'static str),
    /// A struct with no tag, known only by its typedef.
    Anonymous,
}

/// A struct the header defines from a Rust `repr(C)` struct.
pub struct Struct {
    /// The typedef name.
    pub name: &'static str,
    /// How it is introduced.
    pub tag: Tag,
    /// Its fields.
    pub fields: &'static [CField],
    /// Rust type tokens to read as other tokens, for a header struct that
    /// shares a Rust struct with another but names its own flag word.
    pub retype: &'static [(&'static str, &'static str)],
    /// The C++ assignments the struct defines: for each, the Rust type tokens
    /// of the value and the name of the value code it is stored as.
    pub assignments: &'static [(&'static str, &'static str)],
}

/// A struct with a tag of its name preceded by an underscore.
const fn named(name: &'static str, tag: &'static str, fields: &'static [CField]) -> Struct {
    Struct {
        name,
        tag: Tag::Named(tag),
        fields,
        retype: &[],
        assignments: &[],
    }
}

/// A struct with no tag.
const fn anonymous(name: &'static str, fields: &'static [CField]) -> Struct {
    Struct {
        name,
        tag: Tag::Anonymous,
        fields,
        retype: &[],
        assignments: &[],
    }
}

/// The classic API's data structs, in `rexx.h`.
const CLASSIC_STRUCTS: &[Struct] = &[
    named("RXSTRING", "_RXSTRING", RXSTRING::C_FIELDS),
    named("CONSTRXSTRING", "_CONSTRXSTRING", CONSTRXSTRING::C_FIELDS),
    named("RXSYSEXIT", "_RXSYSEXIT", RXSYSEXIT::C_FIELDS),
    named("SHVBLOCK", "_SHVBLOCK", SHVBLOCK::C_FIELDS),
    named("RexxQueueTime", "_RexxQueueTime", RexxQueueTime::C_FIELDS),
    named(
        "RexxConditionData",
        "_RexxConditionData",
        RexxConditionData::C_FIELDS,
    ),
];

/// The exit parameter blocks that have a Rust struct, in `rexx.h`, after the
/// flag structs.
const EXIT_STRUCTS: &[Struct] = &[
    named("RXFNCCAL_PARM", "_RXFNCCAL_PARM", RXFNCCAL_PARM::C_FIELDS),
    named(
        "RXOFNCCAL_PARM",
        "_RXOFNCCAL_PARM",
        RXOFNCCAL_PARM::C_FIELDS,
    ),
    Struct {
        name: "RXEXFCAL_PARM",
        tag: Tag::Named("_RXEXFCAL_PARM"),
        fields: RXOFNCCAL_PARM::C_FIELDS,
        retype: &[("RXOFNC_FLAGS", "RXEXF_FLAGS")],
        assignments: &[],
    },
    named("RXCMDHST_PARM", "_RXCMDHST_PARM", RXCMDHST_PARM::C_FIELDS),
    named("RXMSQPSH_PARM", "_RXMSQPSH_PARM", RXMSQPSH_PARM::C_FIELDS),
    named("RXSIOSAY_PARM", "_RXSIOSAY_PARM", RXSIOSAY_PARM::C_FIELDS),
    named("RXSIOTRC_PARM", "_RXSIOTRC_PARM", RXSIOSAY_PARM::C_FIELDS),
    named(
        "RXVARNOVALUE_PARM",
        "_RXVARNOVALUE_PARM",
        RXVARNOVALUE_PARM::C_FIELDS,
    ),
    named(
        "RXVALCALL_PARM",
        "_RXVALCALL_PARM",
        RXVALCALL_PARM::C_FIELDS,
    ),
    named("RXMSQPLL_PARM", "_RXMSQPLL_PARM", RXMSQPLL_PARM::C_FIELDS),
    named("RXMSQSIZ_PARM", "_RXMSQSIZ_PARM", RXMSQSIZ_PARM::C_FIELDS),
    named("RXMSQNAM_PARM", "_RXMSQNAM_PARM", RXMSQNAM_PARM::C_FIELDS),
    named("RXSIOTRD_PARM", "_RXSIOTRD_PARM", RXSIOTRD_PARM::C_FIELDS),
    named("RXSIODTR_PARM", "_RXSIODTR_PARM", RXSIODTR_PARM::C_FIELDS),
    named("RXHLTTST_PARM", "_RXHLTTST_PARM", RXHLTTST_PARM::C_FIELDS),
    named("RXTRCTST_PARM", "_RXTRCTST_PARM", RXTRCTST_PARM::C_FIELDS),
];

/// The object-oriented API's data structs, in `oorexxapi.h`.
const API_STRUCTS: &[Struct] = &[
    Struct {
        name: "ValueDescriptor",
        tag: Tag::Named("_ValueDescriptor"),
        fields: ValueDescriptor::C_FIELDS,
        retype: &[],
        assignments: VALUE_ASSIGNMENTS,
    },
    anonymous("RexxCondition", RexxCondition::C_FIELDS),
    named(
        "RexxContextExit",
        "_RexxContextExit",
        RexxContextExit::C_FIELDS,
    ),
    named(
        "RexxRoutineEntry",
        "_RexxRoutineEntry",
        RexxRoutineEntry::C_FIELDS,
    ),
    named(
        "RexxMethodEntry",
        "_RexxMethodEntry",
        RexxMethodEntry::C_FIELDS,
    ),
    named(
        "RexxPackageEntry",
        "_RexxPackageEntry",
        RexxPackageEntry::C_FIELDS,
    ),
    anonymous("RexxContextEnvironment", RexxContextEnvironment::C_FIELDS),
    anonymous(
        "RexxRedirectingEnvironment",
        RexxRedirectingEnvironment::C_FIELDS,
    ),
    anonymous(
        "RexxRegisteredEnvironment",
        RexxRegisteredEnvironment::C_FIELDS,
    ),
    anonymous("RexxLibraryPackage", RexxLibraryPackage::C_FIELDS),
    anonymous("RexxOption", RexxOption::C_FIELDS),
];

/// The context structs, by their Rust name (the C struct tag), in
/// `oorexxapi.h`, in the order of their tables in [`TABLES`].
pub const CONTEXTS: &[(&str, &[CField])] = &[
    ("RexxInstance_", RexxInstance_::C_FIELDS),
    ("RexxThreadContext_", RexxThreadContext_::C_FIELDS),
    ("RexxMethodContext_", RexxMethodContext_::C_FIELDS),
    ("RexxCallContext_", RexxCallContext_::C_FIELDS),
    ("RexxExitContext_", RexxExitContext_::C_FIELDS),
    (
        "RexxIORedirectorContext_",
        RexxIORedirectorContext_::C_FIELDS,
    ),
];

/// The interface tables, by name, in `oorexxapi.h`: each the `functions` of
/// the context of the same place in [`CONTEXTS`].
pub const TABLES: &[(&str, &[(&str, CMember)])] = &[
    ("RexxInstanceInterface", RexxInstanceInterface::C_DECLS),
    ("RexxThreadInterface", RexxThreadInterface::C_DECLS),
    ("MethodContextInterface", MethodContextInterface::C_DECLS),
    ("CallContextInterface", CallContextInterface::C_DECLS),
    ("ExitContextInterface", ExitContextInterface::C_DECLS),
    ("IORedirectorInterface", IORedirectorInterface::C_DECLS),
];

/// The generated text of one section of one file.
pub struct Section {
    /// The file, relative to `include/`.
    pub file: String,
    /// The section's name, as its markers spell it.
    pub name: &'static str,
    /// What lies between the markers, ending in a newline.
    pub text: String,
}

/// One message of the error table: its major and minor code and its
/// symbolic name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ErrorRow<'a> {
    /// The major code.
    pub major: u16,
    /// The minor code.
    pub sub: u16,
    /// The symbolic name, `Error_...`.
    pub symbol: &'a str,
}

/// Every generated section of every header, with `errors` the rows of the
/// error table (`interrexx-inventory/data/errors.tsv`), from which
/// `oorexxerrors.h` is written.
#[must_use]
pub fn sections(errors: &[ErrorRow<'_>]) -> Vec<Section> {
    let section = |file: &str, name: &'static str, text: String| Section {
        file: file.to_string(),
        name,
        text,
    };
    let mut out = vec![
        section("rexx.h", "typedefs", typedefs(C_TYPEDEFS)),
        section("rexx.h", "handles", handles()),
        section("rexx.h", "structs", classic_structs()),
        section("rexx.h", "pointers", typedefs(C_POINTER_TYPEDEFS)),
        section("rexx.h", "handlers", handler_typedefs(C_HANDLERS)),
        section("rexx.h", "constants", constants(CLASSIC_LIMITS)),
        section("rexx.h", "exports", exports(Declared::In("rexx.h"))),
        section("rexxapidefs.h", "constants", constants(CLASSIC_CONSTANTS)),
        section("oorexxapi.h", "contexts", context_typedefs()),
        section("oorexxapi.h", "handlers", handler_typedefs(API_HANDLERS)),
        section("oorexxapi.h", "structs", structs(API_STRUCTS)),
        section("oorexxapi.h", "tables", tables()),
        section("oorexxapi.h", "context-structs", context_structs()),
        section("oorexxapi.h", "constants", api_constants()),
        section("oorexxapi.h", "argument-types", argument_types()),
        section(
            "oorexxapi.h",
            "exports",
            exports(Declared::In("oorexxapi.h")),
        ),
        section("oorexxerrors.h", "errors", error_codes(errors)),
    ];
    for platform in PLATFORMS {
        let dir = format!("{PLATFORM_DIR}/{}", platform.name);
        out.push(section(
            &format!("{dir}/rexxapitypes.h"),
            "platform-types",
            platform_types(platform),
        ));
        out.push(section(
            &format!("{dir}/rexxplatformdefs.h"),
            "platform-defs",
            platform_defs(platform),
        ));
        out.push(section(
            &format!("{dir}/rexxplatformapis.h"),
            "platform-exports",
            exports(Declared::Platform),
        ));
    }
    out
}

/// The files [`sections`] writes into, relative to `include/`, each once.
#[must_use]
pub fn files() -> Vec<String> {
    let mut files: Vec<String> = Vec::new();
    for section in sections(&[]) {
        if !files.contains(&section.file) {
            files.push(section.file);
        }
    }
    files
}

/// The opening marker line of section `name`.
#[must_use]
pub fn begin_marker(name: &str) -> String {
    format!("/* BEGIN GENERATED {name} */")
}

/// The closing marker line of section `name`.
#[must_use]
pub fn end_marker(name: &str) -> String {
    format!("/* END GENERATED {name} */")
}

/// `text` with the lines between each pair of markers replaced by that
/// section's generated text.
///
/// # Errors
/// When a section's markers are missing, out of order or repeated, or `text`
/// has markers of a section that is not given.
pub fn splice(text: &str, sections: &[(&str, &str)]) -> Result<String, String> {
    let mut out = String::new();
    let mut lines = text.split_inclusive('\n');
    let mut seen: Vec<&str> = Vec::new();
    while let Some(line) = lines.next() {
        out.push_str(line);
        let trimmed = line.trim_end();
        let Some(name) = trimmed
            .strip_prefix("/* BEGIN GENERATED ")
            .and_then(|rest| rest.strip_suffix(" */"))
        else {
            if trimmed.starts_with("/* END GENERATED ") {
                return Err(format!("{trimmed} closes no open section"));
            }
            continue;
        };
        let Some((_, generated)) = sections.iter().find(|(given, _)| *given == name) else {
            return Err(format!("no section {name} is generated for this file"));
        };
        if seen.contains(&name) {
            return Err(format!("section {name} appears twice"));
        }
        seen.push(name);
        let end = end_marker(name);
        loop {
            let Some(skipped) = lines.next() else {
                return Err(format!("section {name} has no {end}"));
            };
            if skipped.trim_end() == end {
                out.push_str(generated);
                out.push_str(skipped);
                break;
            }
            if skipped.starts_with("/* BEGIN GENERATED ") {
                return Err(format!(
                    "section {name} is not closed before the next opens"
                ));
            }
        }
    }
    if let Some((missing, _)) = sections.iter().find(|(name, _)| !seen.contains(name)) {
        return Err(format!("the file has no markers for section {missing}"));
    }
    Ok(out)
}

/// `c_type` of `tokens`, after `retype`.
fn spelled(tokens: &str, retype: &[(&str, &str)]) -> String {
    let tokens = retype
        .iter()
        .find(|(from, _)| *from == tokens)
        .map_or(tokens, |(_, to)| *to);
    c_type(tokens)
}

/// A C declarator: `ty` and `name`, with no space after a star.
fn declarator(ty: &str, name: &str) -> String {
    if ty.ends_with('*') {
        format!("{ty}{name}")
    } else {
        format!("{ty} {name}")
    }
}

/// A typedef of each name of `list` for the C type of its Rust tokens.
fn typedefs(list: &[(&str, &str)]) -> String {
    let mut out = String::new();
    for (name, tokens) in list {
        writeln!(out, "typedef {};", declarator(&c_type(tokens), name)).unwrap();
    }
    out
}

/// The result and the parameter list of a function declaration, as C types:
/// `void` for no result and for no parameters.
fn signature(declaration: &CMember) -> (String, String) {
    let CMember::Function {
        returns,
        parameters,
    } = declaration
    else {
        panic!("{declaration:?} declares no function");
    };
    let returns = returns.map_or_else(|| "void".to_string(), c_type);
    let parameters = if parameters.is_empty() {
        "void".to_string()
    } else {
        parameters
            .iter()
            .map(|tokens| c_type(tokens))
            .collect::<Vec<_>>()
            .join(", ")
    };
    (returns, parameters)
}

/// A function type of each handler of `list`, under its name.
fn handler_typedefs(list: &[(&str, CMember)]) -> String {
    let mut out = String::new();
    for (name, declaration) in list {
        let (returns, parameters) = signature(declaration);
        writeln!(out, "typedef {returns} REXXENTRY {name}({parameters});").unwrap();
    }
    out
}

/// A `#define` of each constant of `list`.
fn constants(list: &[CConstant]) -> String {
    let mut out = String::new();
    for constant in list {
        writeln!(out, "#define {} {}", constant.name, constant.text()).unwrap();
    }
    out
}

/// The constants of `oorexxapi.h`: the versions, the value codes, the option
/// names and the kinds of command environment.
fn api_constants() -> String {
    let mut out = constants(API_VERSIONS);
    out.push('\n');
    out.push_str(&constants(&value_codes()));
    out.push('\n');
    for (name, text) in OPTION_NAMES {
        writeln!(out, "#define {name} \"{text}\"").unwrap();
    }
    out.push_str(&constants(ENVIRONMENT_KINDS));
    out.push('\n');
    out.push_str(&constants(ENTRY_CONSTANTS));
    out
}

/// `ARGUMENT_TYPE_<name>` for every value code and its optional form, each
/// the C type of the argument an entry receives for it, then the typedef of
/// each special argument's name.
fn argument_types() -> String {
    let mut out = String::new();
    for (name, tokens) in ARGUMENT_TYPES {
        writeln!(out, "#define ARGUMENT_TYPE_{name} {}", c_type(tokens)).unwrap();
    }
    for name in optional_argument_types() {
        let Some((_, tokens)) = ARGUMENT_TYPES.iter().find(|(row, _)| *row == name) else {
            panic!("the optional argument type {name} has no required one");
        };
        writeln!(
            out,
            "#define ARGUMENT_TYPE_OPTIONAL_{name} {}",
            c_type(tokens)
        )
        .unwrap();
    }
    out.push('\n');
    for (name, tokens) in ARGUMENT_TYPEDEFS {
        writeln!(out, "typedef {};", declarator(&c_type(tokens), name)).unwrap();
    }
    out
}

/// The declaration of `export`.
#[must_use]
pub fn prototype(export: &Export) -> String {
    let (returns, parameters) = signature(&export.declaration);
    format!(
        "{};",
        declarator(
            &returns,
            &format!("REXXENTRY {}({parameters})", export.name)
        )
    )
}

/// The exports declared where `declared` says: each prototype, then the
/// typedef of a pointer to each and the upper-case alias of each, where they
/// have one.
fn exports(declared: Declared) -> String {
    let mine: Vec<&Export> = EXPORTS.iter().filter(|e| e.declared == declared).collect();
    let mut out = String::new();
    for export in &mine {
        writeln!(out, "{}", prototype(export)).unwrap();
    }
    // A pointer typedef is declared beside the classic exports, so that every
    // one is in `rexx.h`, its function's platform declaration included.
    if declared == Declared::In("rexx.h") {
        out.push('\n');
        for (name, declaration, _) in pointer_types() {
            let (returns, parameters) = signature(&declaration);
            writeln!(
                out,
                "typedef {};",
                declarator(&returns, &format!("(REXXENTRY *{name})({parameters})"))
            )
            .unwrap();
        }
    }
    let aliases: Vec<(&str, &str)> = mine
        .iter()
        .filter_map(|export| Some((export.alias?, export.name)))
        .collect();
    if !aliases.is_empty() {
        out.push('\n');
        for (alias, name) in aliases {
            writeln!(out, "#define {alias} {name}").unwrap();
        }
    }
    out
}

/// `Rexx_<symbol>` for every row of `errors` that has a code.
fn error_codes(errors: &[ErrorRow<'_>]) -> String {
    let mut out = String::new();
    for row in errors.iter().filter(|row| has_error_code(row.major)) {
        writeln!(
            out,
            "#define Rexx_{} {}",
            row.symbol,
            error_code(row.major, row.sub)
        )
        .unwrap();
    }
    out
}

/// The handle types: in C a pointer to an incomplete struct; in C++ a pointer
/// to an empty struct derived from the struct of the handle it converts to.
///
/// The C++ structs sit outside the file's `extern "C"` block: an empty struct is
/// size 0 in C and size 1 in C++, which clang's `-Wextern-c-compat` rejects
/// under `-Werror` inside a C-linkage block. The structs have no linkage of
/// their own, so closing the block changes no symbol.
fn handles() -> String {
    let mut out = String::new();
    out.push_str("#ifdef __cplusplus\n");
    out.push_str("}\n");
    for (at, (handle, base)) in HANDLES.iter().enumerate() {
        match base {
            None => writeln!(out, "struct _{handle} {{}};").unwrap(),
            Some(base) => {
                assert!(
                    HANDLES[..at].iter().any(|(earlier, _)| earlier == base),
                    "{handle} derives from {base}, which is not declared before it"
                );
                writeln!(out, "struct _{handle} : public _{base} {{}};").unwrap();
            }
        }
    }
    for (handle, _) in HANDLES {
        writeln!(out, "typedef _{handle} *{handle};").unwrap();
    }
    out.push_str("extern \"C\" {\n");
    out.push_str("#else\n");
    for (handle, _) in HANDLES {
        writeln!(out, "typedef struct _{handle} *{handle};").unwrap();
    }
    out.push_str("#endif\n");
    out
}

/// The body lines of a struct or union with `fields`, indented by `indent`.
fn fields(out: &mut String, fields: &[CField], retype: &[(&str, &str)], indent: &str) {
    for field in fields {
        match field.ty {
            CSpelling::Rust("ValueUnion") => {
                writeln!(out, "{indent}union {{").unwrap();
                self::fields(out, ValueUnion::C_FIELDS, &[], &format!("{indent}    "));
                writeln!(out, "{indent}}} {};", field.name).unwrap();
            }
            CSpelling::Rust(tokens) => {
                writeln!(
                    out,
                    "{indent}{};",
                    declarator(&spelled(tokens, retype), field.name)
                )
                .unwrap();
            }
            CSpelling::C(ty) => {
                writeln!(out, "{indent}{};", declarator(ty, field.name)).unwrap();
            }
        }
    }
}

/// The C++ assignment operators of `list`, each storing its value in the union
/// member of its value code and setting `type` to that code. The code is
/// written as a number: the `REXX_VALUE_` names are defined after the structs.
fn assignments(out: &mut String, list: &[(&str, &str)]) {
    if list.is_empty() {
        return;
    }
    out.push_str("#ifdef __cplusplus\n");
    for (tokens, name) in list {
        let Some((code, _)) = rows().find(|(_, row)| row == name) else {
            panic!("the conversion table has no code {name}");
        };
        writeln!(
            out,
            "    void operator=({} v) {{ type = {code}; value.value_{name} = v; }}",
            c_type(tokens)
        )
        .unwrap();
    }
    out.push_str("#endif\n");
}

/// Every struct of `list`: the typedefs of the tagged ones first, then each
/// definition in order.
fn structs(list: &[Struct]) -> String {
    let mut out = String::new();
    for s in list {
        if let Tag::Named(tag) = s.tag {
            writeln!(out, "typedef struct {tag} {};", s.name).unwrap();
        }
    }
    for s in list {
        out.push('\n');
        match s.tag {
            Tag::Named(tag) => {
                writeln!(out, "struct {tag} {{").unwrap();
                fields(&mut out, s.fields, s.retype, "    ");
                assignments(&mut out, s.assignments);
                out.push_str("};\n");
            }
            Tag::Anonymous => {
                out.push_str("typedef struct {\n");
                fields(&mut out, s.fields, s.retype, "    ");
                writeln!(out, "}} {};", s.name).unwrap();
            }
        }
    }
    out
}

/// The classic structs, the exit flag words as one-bit fields from bit 0
/// upward, and the exit parameter blocks.
fn classic_structs() -> String {
    let mut out = structs(CLASSIC_STRUCTS);
    out.push('\n');
    for (name, members) in EXIT_FLAG_BITS {
        writeln!(out, "typedef struct _{name} {{").unwrap();
        for (at, (member, bit)) in members.iter().enumerate() {
            assert_eq!(
                usize::try_from(*bit).ok(),
                Some(at),
                "{name}.{member}: a bit field is laid out in declaration order from bit 0, \
                 so its bits must be 0, 1, 2, ..."
            );
            writeln!(out, "    unsigned {member} : 1;").unwrap();
        }
        writeln!(out, "}} {name};").unwrap();
    }
    out.push('\n');
    out.push_str(&structs(EXIT_STRUCTS));
    out
}

/// The context names: in C++ the struct, in C a pointer to the constant
/// struct.
fn context_typedefs() -> String {
    let mut out = String::new();
    out.push_str("#ifdef __cplusplus\n");
    for (tag, _) in CONTEXTS {
        writeln!(out, "struct {tag};").unwrap();
        writeln!(out, "typedef {tag} {};", c_type(tag)).unwrap();
    }
    out.push_str("#else\n");
    for (tag, _) in CONTEXTS {
        writeln!(out, "typedef const struct {tag} *{};", c_type(tag)).unwrap();
    }
    out.push_str("#endif\n");
    out
}

/// The interface tables: one function pointer per function member, in order.
fn tables() -> String {
    let mut out = String::new();
    for (at, (name, members)) in TABLES.iter().enumerate() {
        if at > 0 {
            out.push('\n');
        }
        out.push_str("typedef struct {\n");
        for (member, decl) in *members {
            match decl {
                CMember::Value(tokens) => {
                    writeln!(out, "    {};", declarator(&c_type(tokens), member)).unwrap();
                }
                CMember::Function { .. } => {
                    let (returns, parameters) = signature(decl);
                    writeln!(out, "    {returns} (RexxEntry *{member})({parameters});").unwrap();
                }
            }
        }
        writeln!(out, "}} {name};").unwrap();
    }
    out
}

/// The context structs' fields and, in C++, their methods.
fn context_structs() -> String {
    let mut out = String::new();
    for (at, (tag, members)) in CONTEXTS.iter().enumerate() {
        if at > 0 {
            out.push('\n');
        }
        writeln!(out, "struct {tag} {{").unwrap();
        fields(&mut out, members, &[], "    ");
        out.push_str("#ifdef __cplusplus\n");
        for method in methods(tag) {
            writeln!(out, "    {}", method.definition()).unwrap();
        }
        out.push_str("#endif\n");
        out.push_str("};\n");
    }
    out
}

/// How a context struct's C++ method reaches the interface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Body {
    /// Calls this function member of the context's own table, the context
    /// first and the method's arguments after it.
    Table(&'static str),
    /// Answers this value member of the context's own table.
    Value(&'static str),
    /// Calls the method of the same name and parameters on the context's
    /// `threadContext`.
    Thread,
    /// Calls the method of this name on the context's `instance`.
    Instance(&'static str),
    /// Answers this field of the context's `instance`.
    InstanceField(&'static str),
}

/// One C++ method of a context struct.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Method {
    /// Its name.
    pub name: &'static str,
    /// The C type of its result, `void` for none.
    pub returns: String,
    /// The C type of each parameter.
    pub parameters: Vec<String>,
    /// What it calls or reads.
    pub body: Body,
}

impl Method {
    /// The name and the parameter types, which tell overloads apart.
    #[must_use]
    pub fn signature(&self) -> String {
        format!("{}({})", self.name, self.parameters.join(", "))
    }

    /// The inline definition, on one line.
    #[must_use]
    pub fn definition(&self) -> String {
        let parameters: Vec<String> = self
            .parameters
            .iter()
            .enumerate()
            .map(|(at, ty)| declarator(ty, &format!("a{}", at + 1)))
            .collect();
        let arguments: Vec<String> = (1..=self.parameters.len())
            .map(|at| format!("a{at}"))
            .collect();
        let arguments = arguments.join(", ");
        let expression = match self.body {
            Body::Table(member) if arguments.is_empty() => format!("functions->{member}(this)"),
            Body::Table(member) => format!("functions->{member}(this, {arguments})"),
            Body::Value(member) => format!("functions->{member}"),
            Body::Thread => format!("threadContext->{}({arguments})", self.name),
            Body::Instance(method) => format!("instance->{method}({arguments})"),
            Body::InstanceField(field) => format!("instance->{field}"),
        };
        let statement = if self.returns == "void" {
            expression
        } else {
            format!("return {expression}")
        };
        format!(
            "{} {{ {statement}; }}",
            declarator(
                &self.returns,
                &format!("{}({})", self.name, parameters.join(", "))
            )
        )
    }
}

/// The interface table of context `context`.
///
/// # Panics
/// If `context` is not one of [`CONTEXTS`].
fn table_of(context: &str) -> &'static [(&'static str, CMember)] {
    let at = CONTEXTS
        .iter()
        .position(|(tag, _)| *tag == context)
        .unwrap_or_else(|| panic!("{context} is not a context struct"));
    TABLES[at].1
}

/// The result and the parameters after the context of function member
/// `member` of `table`, as C types.
///
/// # Panics
/// If `table` has no function member `member`.
fn function_of(table: &[(&str, CMember)], member: &str) -> (String, Vec<String>) {
    let Some((
        _,
        CMember::Function {
            returns,
            parameters,
        },
    )) = table.iter().find(|(name, _)| *name == member)
    else {
        panic!("{member} is not a function member of its table");
    };
    let returns = returns.map_or_else(|| "void".to_string(), c_type);
    let parameters = parameters.iter().skip(1).map(|ty| c_type(ty)).collect();
    (returns, parameters)
}

/// A method calling `spelling.target` of `table`, under `spelling`'s name and
/// with its spelling where it gives one.
fn spelled_method(table: &[(&str, CMember)], spelling: &CxxMethod, body: Body) -> Method {
    let (returns, parameters) = function_of(table, spelling.target);
    Method {
        name: spelling.name,
        returns: spelling.returns.map_or(returns, c_type),
        parameters: spelling.parameters.map_or(parameters, |tokens| {
            tokens.iter().map(|ty| c_type(ty)).collect()
        }),
        body,
    }
}

/// A method of each function member of `table` not in `unnamed`, under the
/// member's name, spelled as `respelled` says where it names the member.
fn own_methods(
    table: &'static [(&'static str, CMember)],
    unnamed: &[&str],
    respelled: &[CxxMethod],
) -> Vec<Method> {
    table
        .iter()
        .filter(|(name, decl)| matches!(decl, CMember::Function { .. }) && !unnamed.contains(name))
        .map(|(name, _)| {
            let spelling = respelled
                .iter()
                .find(|spelling| spelling.target == *name)
                .copied()
                .unwrap_or(CxxMethod {
                    name,
                    target: name,
                    parameters: None,
                    returns: None,
                });
            spelled_method(table, &spelling, Body::Table(name))
        })
        .collect()
}

/// The thread context's methods: one per function member, the convenience
/// aliases, the constant-object accessors and the forwards to its instance.
fn thread_methods() -> Vec<Method> {
    let table = table_of("RexxThreadContext_");
    let instance = table_of("RexxInstance_");
    let mut out = own_methods(table, THREAD_UNNAMED, THREAD_RESPELLED);
    for alias in THREAD_ALIASES {
        out.push(spelled_method(table, alias, Body::Table(alias.target)));
    }
    for (name, member) in THREAD_VALUE_ACCESSORS {
        let Some((_, CMember::Value(tokens))) = table.iter().find(|(field, _)| field == member)
        else {
            panic!("{member} is not a value member of the thread table");
        };
        out.push(Method {
            name,
            returns: c_type(tokens),
            parameters: Vec::new(),
            body: Body::Value(member),
        });
    }
    for name in THREAD_INSTANCE_FORWARDS {
        let (returns, parameters) = function_of(instance, name);
        out.push(Method {
            name,
            returns,
            parameters,
            body: Body::Instance(name),
        });
    }
    for (name, field, returns) in THREAD_INSTANCE_FIELDS {
        assert!(
            RexxInstance_::C_FIELDS.iter().any(|f| f.name == *field),
            "{field} is not a field of the instance"
        );
        out.push(Method {
            name,
            returns: c_type(returns),
            parameters: Vec::new(),
            body: Body::InstanceField(field),
        });
    }
    out
}

/// The C++ methods of context struct `context`, in the order the header
/// defines them.
///
/// The instance and the redirector context have one method per function member
/// of their table. The thread context adds to those the methods of
/// [`thread_methods`]. The method, call and exit contexts have one per function
/// member of their own table and forward every thread context method but
/// [`THREAD_ONLY`] to their `threadContext`.
///
/// # Panics
/// If `context` is not one of [`CONTEXTS`].
#[must_use]
pub fn methods(context: &str) -> Vec<Method> {
    let table = table_of(context);
    match context {
        "RexxThreadContext_" => thread_methods(),
        "RexxMethodContext_" | "RexxCallContext_" | "RexxExitContext_" => {
            let mut out = own_methods(table, &[], &[]);
            for method in thread_methods() {
                if THREAD_ONLY.contains(&method.name) {
                    continue;
                }
                let returns = FORWARDED_RESPELLED
                    .iter()
                    .find(|spelling| {
                        spelling.name == method.name && method.body == Body::Table(spelling.target)
                    })
                    .and_then(|spelling| spelling.returns)
                    .map_or(method.returns, c_type);
                out.push(Method {
                    returns,
                    body: Body::Thread,
                    ..method
                });
            }
            out
        }
        _ => own_methods(table, &[], &[]),
    }
}

fn platform_types(platform: &Platform) -> String {
    let mut out = String::new();
    for header in platform.system_headers {
        writeln!(out, "#include <{header}>").unwrap();
    }
    out.push('\n');
    for (name, ty) in platform.typedefs {
        writeln!(out, "typedef {};", declarator(ty, name)).unwrap();
    }
    if !platform.structs.is_empty() {
        out.push('\n');
        out.push_str(&structs(platform.structs));
    }
    out
}

fn platform_defs(platform: &Platform) -> String {
    let mut out = String::new();
    for (name, expansion) in platform.defines {
        if expansion.is_empty() {
            writeln!(out, "#define {name}").unwrap();
        } else {
            writeln!(out, "#define {name} {expansion}").unwrap();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A section's lines are replaced and the text around them is kept.
    #[test]
    fn splice_replaces_only_between_markers() {
        let text = "a\n/* BEGIN GENERATED s */\nold\n/* END GENERATED s */\nb\n";
        assert_eq!(
            splice(text, &[("s", "new\n")]).unwrap(),
            "a\n/* BEGIN GENERATED s */\nnew\n/* END GENERATED s */\nb\n"
        );
    }

    /// Missing, unknown, repeated and unclosed sections are refused.
    #[test]
    fn splice_refuses_a_malformed_file() {
        let one = "/* BEGIN GENERATED s */\n/* END GENERATED s */\n";
        assert!(splice(one, &[("s", ""), ("t", "")]).is_err());
        assert!(splice(one, &[("t", "")]).is_err());
        assert!(splice(&format!("{one}{one}"), &[("s", "")]).is_err());
        assert!(splice("/* BEGIN GENERATED s */\n", &[("s", "")]).is_err());
        assert!(splice("/* END GENERATED s */\n", &[]).is_err());
    }

    /// The mapping keys on tokens: two Rust aliases of `usize` keep their own
    /// C names, and `usize` itself is `size_t`.
    #[test]
    fn the_mapping_keeps_each_alias_spelling() {
        assert_eq!(c_type("stringsize_t"), "stringsize_t");
        assert_eq!(c_type("logical_t"), "logical_t");
        assert_eq!(c_type("usize"), "size_t");
        assert_eq!(c_type("*const c_char"), "const char *");
        assert_eq!(
            c_type("*mut *mut RexxThreadContext_"),
            "RexxThreadContext **"
        );
        assert_eq!(c_type("Option<RexxPackageLoader>"), "RexxPackageLoader");
        assert_eq!(c_type("Option < RexxPackageLoader >"), "RexxPackageLoader");
    }

    /// A table member's declaration comes from the tokens of its Rust type.
    #[test]
    fn a_table_member_is_declared_from_its_rust_type() {
        let text = tables();
        assert!(
            text.contains("    stringsize_t (RexxEntry *GetContextDigits)(RexxCallContext *);\n"),
            "{text}"
        );
        assert!(
            text.contains(
                "    logical_t (RexxEntry *AttachThread)(RexxInstance *, RexxThreadContext **);\n"
            ),
            "{text}"
        );
        assert!(
            text.contains("    wholenumber_t interfaceVersion;\n"),
            "{text}"
        );
    }
}
