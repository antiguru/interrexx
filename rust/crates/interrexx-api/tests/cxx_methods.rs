// SPDX-License-Identifier: MIT OR Apache-2.0

//! The C++ methods of the context structs in `include/oorexxapi.h` reach the table members they
//! stand for.
//!
//! An inline method has no symbol and no layout, so neither the ABI dump nor a comparison of names
//! sees a method that calls the wrong member of the same type. This test writes a C++ program
//! that fills every interface table with recording stubs (each function member records its own
//! name, the context it was handed and its arguments; each value member holds a value of its
//! own), links the six context structs to one another, calls every method the header generator
//! lists once and prints what each call reached. The member each method must reach is worked out
//! here from the tables of `layout.rs`, not from what the generator emits.

use interrexx_api::header::{CONTEXTS, Method, TABLES, methods};
use interrexx_api::layout::{
    CMember, CxxMethod, THREAD_ALIASES, THREAD_INSTANCE_FIELDS, THREAD_INSTANCE_FORWARDS,
    THREAD_ONLY, THREAD_VALUE_ACCESSORS, c_type,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The value a value member of a table holds in the program: this plus its index in
/// [`value_members`].
const VALUE_BASE: usize = 0x1000;

/// The value the instance's `applicationData` field holds in the program.
const APPLICATION_DATA: usize = 0x2000;

/// The repository's `include/` directory and this platform's directory below it.
fn include_dirs() -> [PathBuf; 2] {
    let include = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../include");
    [include.join("platform/unix"), include]
}

/// The variable the program holds context struct `context` in.
fn context_variable(context: &str) -> String {
    format!("ctx_{context}")
}

/// The variable the program holds the table of the context at `at` of [`CONTEXTS`] in.
fn table_variable(at: usize) -> String {
    format!("table_{}", TABLES[at].0)
}

/// The table of context struct `context`, and its name.
fn table_of(context: &str) -> (&'static str, &'static [(&'static str, CMember)]) {
    let at = CONTEXTS
        .iter()
        .position(|(tag, _)| *tag == context)
        .unwrap_or_else(|| panic!("{context} is not a context struct"));
    TABLES[at]
}

/// Every value member of every table but `interfaceVersion`, as `Table.member`.
fn value_members() -> Vec<String> {
    let mut out = Vec::new();
    for (table, members) in TABLES {
        for (member, decl) in *members {
            if matches!(decl, CMember::Value(_)) && *member != "interfaceVersion" {
                out.push(format!("{table}.{member}"));
            }
        }
    }
    out
}

/// Every function member of every table, as `Table.member`.
fn function_members() -> Vec<String> {
    let mut out = Vec::new();
    for (table, members) in TABLES {
        for (member, decl) in *members {
            if matches!(decl, CMember::Function { .. }) {
                out.push(format!("{table}.{member}"));
            }
        }
    }
    out
}

/// What one call reached, as the program prints it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Reached {
    /// The member called, `Table.member`, or `-` when none was.
    member: String,
    /// The context struct the member was handed, or `-`.
    context: String,
    /// The arguments it was handed after the context, as numbers.
    arguments: String,
    /// The method's result as a number, or `-` for `void`.
    result: String,
}

/// What a call of `method` on `context` must reach.
fn expected(context: &str, method: &Method) -> Reached {
    let result = |value: usize| {
        if method.returns == "void" {
            "-".to_string()
        } else {
            value.to_string()
        }
    };
    let call = |table: &str, member: &str, on: &str| Reached {
        member: format!("{table}.{member}"),
        context: on.to_string(),
        arguments: (1..=method.parameters.len())
            .map(|at| at.to_string())
            .collect::<Vec<_>>()
            .join(","),
        result: result(0),
    };
    let read = |value: usize| Reached {
        member: "-".to_string(),
        context: "-".to_string(),
        arguments: String::new(),
        result: result(value),
    };
    let (table, members) = table_of(context);
    let own_function = members
        .iter()
        .any(|(name, decl)| *name == method.name && matches!(decl, CMember::Function { .. }));
    match context {
        "RexxThreadContext_" => {
            if let Some((_, member)) = THREAD_VALUE_ACCESSORS
                .iter()
                .find(|(name, _)| *name == method.name)
            {
                let at = value_members()
                    .iter()
                    .position(|value| *value == format!("{table}.{member}"))
                    .expect("an accessor reads a value member");
                return read(VALUE_BASE + at);
            }
            if THREAD_INSTANCE_FIELDS
                .iter()
                .any(|(name, _, _)| *name == method.name)
            {
                return read(APPLICATION_DATA);
            }
            if THREAD_INSTANCE_FORWARDS.contains(&method.name) {
                let (instance, _) = table_of("RexxInstance_");
                return call(instance, method.name, "RexxInstance_");
            }
            if let Some(alias) = THREAD_ALIASES.iter().find(|alias| {
                alias.name == method.name && alias_parameters(alias) == method.parameters
            }) {
                return call(table, alias.target, context);
            }
            assert!(
                own_function,
                "{} is no thread context method",
                method.signature()
            );
            call(table, method.name, context)
        }
        "RexxMethodContext_" | "RexxCallContext_" | "RexxExitContext_" if !own_function => {
            expected("RexxThreadContext_", method)
        }
        _ => {
            assert!(own_function, "{context} has no member {}", method.name);
            call(table, method.name, context)
        }
    }
}

/// The C types of an alias's parameters after the context.
fn alias_parameters(alias: &CxxMethod) -> Vec<String> {
    if let Some(tokens) = alias.parameters {
        return tokens.iter().map(|ty| c_type(ty)).collect();
    }
    let (_, members) = table_of("RexxThreadContext_");
    let Some((_, CMember::Function { parameters, .. })) =
        members.iter().find(|(name, _)| *name == alias.target)
    else {
        panic!(
            "{} is not a function member of the thread table",
            alias.target
        );
    };
    parameters.iter().skip(1).map(|ty| c_type(ty)).collect()
}

/// The program's fixed part: the recording, the argument values, the stubs.
const PREAMBLE: &str = r#"
#include <cstdint>
#include <cstdio>
#include <type_traits>
#include "rexx.h"
#include "oorexxapi.h"

static int reached = -1;
static const void *reached_context = nullptr;
static uintptr_t reached_arguments[16];
static size_t reached_count = 0;

template <class T> static T make(uintptr_t k) {
    if constexpr (std::is_pointer<T>::value) {
        return reinterpret_cast<T>(k);
    } else {
        return static_cast<T>(k);
    }
}

template <class T> static uintptr_t bits(T value) {
    if constexpr (std::is_pointer<T>::value) {
        return reinterpret_cast<uintptr_t>(value);
    } else {
        return static_cast<uintptr_t>(value);
    }
}

template <class... A> static void record(int member, const void *context, A... arguments) {
    reached = member;
    reached_context = context;
    uintptr_t values[] = {0, bits(arguments)...};
    reached_count = sizeof...(A);
    for (size_t at = 0; at < reached_count; at++) {
        reached_arguments[at] = values[at + 1];
    }
}

template <int N, class F> struct Stub;
template <int N, class R, class C, class... A> struct Stub<N, R (*)(C, A...)> {
    static R call(C context, A... arguments) {
        record(N, context, arguments...);
        return R();
    }
};

static void reset() {
    reached = -1;
    reached_context = nullptr;
    reached_count = 0;
}
"#;

/// The C++ program that calls every method and prints, per call, one line
/// `context|signature|member|context reached|arguments|result`.
fn program(calls: &[(&str, Method)]) -> String {
    let mut out = String::from(PREAMBLE);
    let functions = function_members();
    out.push_str("static const char *const MEMBERS[] = {\n");
    for member in &functions {
        writeln!(out, "    \"{member}\",").unwrap();
    }
    out.push_str("};\n\n");
    for (at, (table, _)) in TABLES.iter().enumerate() {
        writeln!(out, "static {table} {};", table_variable(at)).unwrap();
    }
    for (context, _) in CONTEXTS {
        writeln!(out, "static {context} {};", context_variable(context)).unwrap();
    }
    out.push_str("\nstatic const char *context_name(const void *context) {\n");
    for (context, _) in CONTEXTS {
        writeln!(
            out,
            "    if (context == &{}) return \"{context}\";",
            context_variable(context)
        )
        .unwrap();
    }
    out.push_str("    return \"?\";\n}\n\n");
    out.push_str(
        r#"static void report(const char *context, const char *signature, bool has_result, uintptr_t result) {
    printf("%s|%s|", context, signature);
    if (reached < 0) {
        printf("-|-|");
    } else {
        printf("%s|%s|", MEMBERS[reached], context_name(reached_context));
    }
    for (size_t at = 0; at < reached_count; at++) {
        printf(at ? ",%lu" : "%lu", (unsigned long)reached_arguments[at]);
    }
    if (has_result) {
        printf("|%lu\n", (unsigned long)result);
    } else {
        printf("|-\n");
    }
}

int main() {
"#,
    );
    let values = value_members();
    for (at, (table, members)) in TABLES.iter().enumerate() {
        let variable = table_variable(at);
        for (member, decl) in *members {
            let full = format!("{table}.{member}");
            match decl {
                CMember::Function { .. } => {
                    let index = functions.iter().position(|f| *f == full).unwrap();
                    writeln!(
                        out,
                        "    {variable}.{member} = &Stub<{index}, decltype({variable}.{member})>::call;"
                    )
                    .unwrap();
                }
                CMember::Value(_) => {
                    if let Some(index) = values.iter().position(|v| *v == full) {
                        writeln!(
                            out,
                            "    {variable}.{member} = make<decltype({variable}.{member})>({});",
                            VALUE_BASE + index
                        )
                        .unwrap();
                    }
                }
            }
        }
    }
    for (at, (context, fields)) in CONTEXTS.iter().enumerate() {
        let variable = context_variable(context);
        writeln!(out, "    {variable}.functions = &{};", table_variable(at)).unwrap();
        for field in *fields {
            match field.name {
                "instance" => {
                    writeln!(
                        out,
                        "    {variable}.instance = &{};",
                        context_variable("RexxInstance_")
                    )
                    .unwrap();
                }
                "threadContext" => writeln!(
                    out,
                    "    {variable}.threadContext = &{};",
                    context_variable("RexxThreadContext_")
                )
                .unwrap(),
                "applicationData" => writeln!(
                    out,
                    "    {variable}.applicationData = make<void *>({APPLICATION_DATA});"
                )
                .unwrap(),
                "arguments" => writeln!(out, "    {variable}.arguments = nullptr;").unwrap(),
                _ => {}
            }
        }
    }
    out.push('\n');
    for (context, method) in calls {
        let arguments: Vec<String> = method
            .parameters
            .iter()
            .enumerate()
            .map(|(at, ty)| format!("make<{ty}>({})", at + 1))
            .collect();
        let call = format!(
            "{}.{}({})",
            context_variable(context),
            method.name,
            arguments.join(", ")
        );
        let signature = method.signature();
        if method.returns == "void" {
            writeln!(
                out,
                "    reset(); {call}; report(\"{context}\", \"{signature}\", false, 0);"
            )
            .unwrap();
        } else {
            writeln!(
                out,
                "    reset(); {{ uintptr_t r = bits({call}); report(\"{context}\", \"{signature}\", true, r); }}"
            )
            .unwrap();
        }
    }
    out.push_str("    return 0;\n}\n");
    out
}

/// Compiles `source` as C++ against `include/`, runs it and answers its standard output.
fn compile_and_run(source: &str) -> String {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cxx_methods");
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
    let path = dir.join("methods.cc");
    std::fs::write(&path, source).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let binary = dir.join("methods");
    let mut command = Command::new("c++");
    command.args(["-std=c++17", "-Wall", "-Wextra", "-pedantic", "-Werror"]);
    for dir in include_dirs() {
        command.arg("-I").arg(dir);
    }
    let built = command
        .arg(&path)
        .arg("-o")
        .arg(&binary)
        .output()
        .expect("c++ runs");
    assert!(
        built.status.success(),
        "the method program does not compile:\n{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let run = Command::new(&binary)
        .output()
        .unwrap_or_else(|e| panic!("{}: {e}", binary.display()));
    assert!(run.status.success(), "{} failed", binary.display());
    String::from_utf8(run.stdout).expect("the program prints text")
}

/// Every method of every context struct, with its context.
fn all_methods() -> Vec<(&'static str, Method)> {
    CONTEXTS
        .iter()
        .flat_map(|(context, _)| methods(context).into_iter().map(move |m| (*context, m)))
        .collect()
}

/// What each call reached, by context and signature.
fn reached(output: &str) -> BTreeMap<(String, String), Reached> {
    let mut out = BTreeMap::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        assert_eq!(parts.len(), 6, "malformed line {line:?}");
        let previous = out.insert(
            (parts[0].to_string(), parts[1].to_string()),
            Reached {
                member: parts[2].to_string(),
                context: parts[3].to_string(),
                arguments: parts[4].to_string(),
                result: parts[5].to_string(),
            },
        );
        assert!(previous.is_none(), "{line:?} is printed twice");
    }
    out
}

/// **Every C++ method reaches the member it stands for**: each method of each context struct,
/// called once with distinct arguments, calls the table member of its own name, its alias's
/// target, the thread context's method or the instance's, handing it the right context and its
/// arguments in order, or answers the value member or instance field it reads.
#[test]
fn every_context_method_reaches_its_member() {
    let calls = all_methods();
    let seen = reached(&compile_and_run(&program(&calls)));
    let mut wrong = Vec::new();
    for (context, method) in &calls {
        let key = (context.to_string(), method.signature());
        let want = expected(context, method);
        match seen.get(&key) {
            None => wrong.push(format!("{context}::{} was not called", key.1)),
            Some(got) if *got != want => wrong.push(format!(
                "{context}::{}: reached {got:?}, expected {want:?}",
                key.1
            )),
            Some(_) => {}
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    assert_eq!(seen.len(), calls.len(), "each method is called once");
}

/// **Every member is reachable through a method of its context**: each context struct has
/// methods reaching every function and value member of its own table; the thread context also
/// reaches the instance members it forwards and the instance's application data; the method, call
/// and exit contexts reach everything the thread context does but [`THREAD_ONLY`].
#[test]
fn every_member_has_a_method() {
    let values = value_members();
    let reach_of = |context: &str| -> BTreeSet<String> {
        methods(context)
            .iter()
            .map(|method| {
                let reached = expected(context, method);
                if reached.member != "-" {
                    return reached.member;
                }
                let value: usize = reached.result.parse().unwrap();
                if value == APPLICATION_DATA {
                    return "RexxInstance_.applicationData".to_string();
                }
                values[value - VALUE_BASE].clone()
            })
            .collect()
    };
    let own = |context: &str| -> BTreeSet<String> {
        let (table, members) = table_of(context);
        members
            .iter()
            .filter(|(member, _)| *member != "interfaceVersion")
            .map(|(member, _)| format!("{table}.{member}"))
            .collect()
    };
    let thread = reach_of("RexxThreadContext_");
    let mut missing = Vec::new();
    for (context, _) in CONTEXTS {
        let reach = reach_of(context);
        let mut need = own(context);
        if *context == "RexxThreadContext_" {
            let (instance, _) = table_of("RexxInstance_");
            need.extend(
                THREAD_INSTANCE_FORWARDS
                    .iter()
                    .map(|member| format!("{instance}.{member}")),
            );
            need.insert("RexxInstance_.applicationData".to_string());
        }
        if matches!(
            *context,
            "RexxMethodContext_" | "RexxCallContext_" | "RexxExitContext_"
        ) {
            let (table, _) = table_of("RexxThreadContext_");
            need.extend(
                thread
                    .iter()
                    .filter(|member| {
                        !THREAD_ONLY
                            .iter()
                            .any(|only| **member == format!("{table}.{only}"))
                    })
                    .cloned(),
            );
        }
        for member in need.difference(&reach) {
            missing.push(format!("{context}: no method reaches {member}"));
        }
    }
    assert!(missing.is_empty(), "{}", missing.join("\n"));
}
