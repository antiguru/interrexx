// SPDX-License-Identifier: MIT OR Apache-2.0

//! The declaration machinery of `include/`, used by an extension and run.
//!
//! `header_macros/macros.c` uses every family of the header's hand-written macros: the routine
//! and method declarations of every arity with every value code, the special arguments, the
//! optional forms, the table rows and prototypes, the package entry, the classic function
//! declaration, the `RXSTRING` macros and the descriptor helpers. The test compiles it as C and
//! as C++ against `include/` alone, each into a library, and runs `header_macros/header_macros.rex`
//! against each: every line the program says is a value the library computed through the macros.
//!
//! spec: native.header.files native.header.languages native.header.handles
//! spec: native.header.platform native.header.platform.unix native.header.declaration
//! spec: native.header.special-types
//! spec: native.header.argument-exists native.header.descriptors native.header.tables
//! spec: native.header.package native.header.rxstring native.header.linkage

#![cfg(target_os = "linux")]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use interrexx::{Invocation, run_program};

/// The program's lines for either build, up to the C++-only lines.
const BEFORE_CXX: &str = "\
routines 0 1 12 123 1234 12345 123456 1234567
routines 12345678 123456789 1234567890 42
methods 0 1 12 123 1234 12345 123456 1234567
methods 12345678 123456789 1234567890 42
signed -7 -7 -7 -7 -7 -7 -7 -7 -7 -7
unsigned 7 7 7 7 7 7 7 7 7 7 1
restricted 7 0
floating 2.5 2.5
strings abc def
objects 1 3 1 Array 1 referenced
pointers 1 1 1 1
omitted 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
given 3 3 3 3 3 3 3 3 3 3 3 3 3 3 3 1 3 3 3 3
null 1 1 1 1 1 1 1 1 1
not null 0 0 0 0 0 0 0 0 0
mixed -8|16|-32|64|0.5|str|1|9|-9|99
name HMNAME HmName
arglist 3 2 1 3
self 1 super Object scope HM cself 1
message MESSAGE margs 2
helpers 1 1 1 1 1 1 1 1
";

/// The lines only the C++ build says: `argumentExists` and `argumentOmitted`, the assignments
/// to a `ValueDescriptor` (the codes of `RexxObjectPtr`, `RexxStringObject`, `RexxArrayObject`,
/// `RexxStemObject`, `RexxMutableBufferObject`, `CSTRING`, `POINTER`, `wholenumber_t`, `float`
/// and `double`, then whether the flags were kept), and a handle widened without a cast.
const CXX_ONLY: &str = "\
exists OOO EOO OEO EEE O E
assign 11 17 31 32 37 15 16 13 19 14 1
widen 1
";

/// The program's last lines for either build.
const AFTER_CXX: &str = "\
classic 0 HMINFO:3 V3 N Z
rows 1 HmSum0 1 2 HmClassicRow 1 0 null 1 | 1 HmMSum0 1
package 1 1 1 headermacros 1
platform 1 1
";

/// A language the extension is compiled as.
#[derive(Clone, Copy)]
enum Language {
    C,
    Cxx,
}

fn sources() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/header_macros")
}

/// Compiles `macros.c` as `language` against `include/` into `dir/lib<name>.so`.
fn build_library(language: Language, dir: &Path, name: &str) {
    let include = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../include");
    let (compiler, flag) = match language {
        Language::C => ("cc", "c"),
        Language::Cxx => ("c++", "c++"),
    };
    let output = Command::new(compiler)
        .args(["-x", flag, "-shared", "-fPIC", "-O1"])
        .args(["-Wall", "-Wextra", "-Wno-unused-parameter", "-Werror"])
        .arg(format!("-I{}", include.display()))
        .arg(format!("-I{}", include.join("platform/unix").display()))
        .arg(sources().join("macros.c"))
        .arg("-o")
        .arg(dir.join(format!("lib{name}.so")))
        .output()
        .unwrap_or_else(|e| panic!("{compiler} runs: {e}"));
    assert!(
        output.status.success(),
        "macros.c does not compile as {flag}:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Runs the program against the library `name` in `dir` and answers what it says.
fn run(dir: &Path, name: &str, cxx: bool) -> String {
    let template =
        fs::read_to_string(sources().join("header_macros.rex")).expect("the program template");
    let text = template
        .replace("@LIB@", name)
        .replace("@CXX@", if cxx { "1" } else { "0" });
    let program = dir.join(format!("{name}.rex"));
    fs::write(&program, &text).expect("the program");
    let mut environment: Vec<(Vec<u8>, Vec<u8>)> = std::env::vars()
        .filter(|(variable, _)| variable != "LD_LIBRARY_PATH")
        .map(|(variable, value)| (variable.into_bytes(), value.into_bytes()))
        .collect();
    environment.push((
        b"LD_LIBRARY_PATH".to_vec(),
        dir.display().to_string().into_bytes(),
    ));
    let invocation = Invocation::none()
        .with_directory(dir.to_path_buf())
        .with_environment(environment);
    let outcome = run_program(&program.to_string_lossy(), text.into_bytes(), invocation);
    assert_eq!(
        outcome.exit_code,
        0,
        "{name}: stderr {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    String::from_utf8_lossy(&outcome.stdout).into_owned()
}

fn scratch(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("header-macros-{test}-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("the scratch directory");
    dir
}

#[test]
fn the_macros_compiled_as_c_declare_what_the_interpreter_calls() {
    let dir = scratch("c");
    build_library(Language::C, &dir, "hmc");
    assert_eq!(run(&dir, "hmc", false), format!("{BEFORE_CXX}{AFTER_CXX}"));
    fs::remove_dir_all(&dir).expect("removing the scratch directory");
}

#[test]
fn the_macros_compiled_as_cxx_declare_what_the_interpreter_calls() {
    let dir = scratch("cxx");
    build_library(Language::Cxx, &dir, "hmcxx");
    assert_eq!(
        run(&dir, "hmcxx", true),
        format!("{BEFORE_CXX}{CXX_ONLY}{AFTER_CXX}")
    );
    fs::remove_dir_all(&dir).expect("removing the scratch directory");
}

/// `REXX_TYPED_ROUTINE_PROTOTYPE` and `REXX_METHOD_PROTOTYPE` end in a semicolon, so a use
/// compiles with and without its own, in C and in C++, under the warnings of the other builds.
#[test]
fn a_prototype_macro_use_compiles_with_and_without_its_own_semicolon() {
    let dir = scratch("prototype-semicolon");
    let include = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../include");
    for (flag, standard) in [("c", "-std=c11"), ("c++", "-std=c++17")] {
        let compiler = if flag == "c" { "cc" } else { "c++" };
        for (case, terminator) in [("bare", ""), ("semicolon", ";")] {
            // A declaration follows each use, so a missing semicolon in the macro would
            // merge the two into one malformed declaration.
            let source = dir.join(format!("{}-{case}.src", flag.replace('+', "x")));
            fs::write(
                &source,
                format!(
                    "#include \"oorexxapi.h\"\n\
                     REXX_TYPED_ROUTINE_PROTOTYPE(HmProtoRoutine){terminator}\n\
                     int hm_after_routine;\n\
                     REXX_METHOD_PROTOTYPE(HmProtoMethod){terminator}\n\
                     int hm_after_method;\n"
                ),
            )
            .expect("the source");
            let output = Command::new(compiler)
                .args(["-x", flag, standard, "-fsyntax-only"])
                .args(["-Wall", "-Wextra", "-Werror"])
                .arg(format!("-I{}", include.display()))
                .arg(format!("-I{}", include.join("platform/unix").display()))
                .arg(&source)
                .output()
                .unwrap_or_else(|e| panic!("{compiler} runs: {e}"));
            assert!(
                output.status.success(),
                "a {case} use does not compile as {flag}:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
    fs::remove_dir_all(&dir).expect("removing the scratch directory");
}
