//! Embeds the Rexx-written part of the interpreter's own library from the C++
//! tree at build time, with the classes `REPLACEMENTS` names taken out of it
//! and this crate's own `rexx/` sources put in their place.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
#[path = "src/lines.rs"]
mod lines;
#[allow(dead_code)]
#[path = "src/sha256.rs"]
mod sha256;
#[path = "src/splice.rs"]
mod splice;

use sha256::sha256_hex;
use splice::{Dropped, Host, Replacement};

/// One embedded program: the name a `CALL` inside the bootstrap resolves it
/// by, the path it is read from relative to this crate, and its pinned
/// sha256.
struct Source {
    name: &'static str,
    path: &'static str,
    sha256: &'static str,
}

const SOURCES: &[Source] = &[
    Source {
        name: "CoreClasses.orx",
        path: "../../../third_party/ooRexx/interpreter/RexxClasses/CoreClasses.orx",
        sha256: "b7c7b71409657c7aa0751408045cb0cad130b51bf7659f362feb919b25f2afad",
    },
    Source {
        name: "StreamClasses.orx",
        path: "../../../third_party/ooRexx/interpreter/RexxClasses/StreamClasses.orx",
        sha256: "b61440a82bd3b2c198a84f44f8f9f0057aed847750c7b9e2c7fc05a819b92bba",
    },
    Source {
        name: "CommandClasses.orx",
        path: "../../../third_party/ooRexx/interpreter/RexxClasses/CommandClasses.orx",
        sha256: "d54af93d041310e890b12d8323dd01035aaac3bbde940bfd057c47c1d1b941e5",
    },
    Source {
        name: "PlatformObjects.orx",
        path: "../../../third_party/ooRexx/interpreter/platform/unix/PlatformObjects.orx",
        sha256: "04f89bb9b1ff2f5ee06f774f6b1b4159699198cc7da48c8703a9a0ab7fe8435e",
    },
];

/// The replacement files under `rexx/`, in order. Adding a file here is the
/// swap: every class it declares replaces the host's class of that name.
const REPLACEMENTS: &[&str] = &[
    "collection.rex",
    "ordering.rex",
    "supplier.rex",
    "datetime.rex",
    "file.rex",
    "streamsupplier.rex",
    "command.rex",
    "sets.rex",
    "timespan.rex",
    "misc.rex",
    "traceobject.rex",
    "timer.rex",
    "monitor.rex",
    "stream.rex",
    "queue.rex",
];

/// One host member a replacement may leave out.
struct Allowed {
    class: &'static str,
    member: &'static str,
    /// `true` for a class-side member.
    class_side: bool,
    /// Why nothing needs it.
    reason: &'static str,
    /// How many places in the spliced hosts send a message of that name, as
    /// reviewed: the build fails when the count differs.
    sent_sites: usize,
}

/// Host members a replacement may leave out. Every entry must match a dropped
/// member of a replaced class, and carry a reason.
const ALLOW_DROPPED: &[Allowed] = &[
    Allowed {
        class: "ArgUtil",
        member: "validateClass",
        class_side: true,
        reason: "ruling 9: ArgUtil is an empty class",
        sent_sites: 0,
    },
    Allowed {
        class: "ArgUtil",
        member: "validateLength",
        class_side: true,
        reason: "ruling 9: ArgUtil is an empty class",
        sent_sites: 0,
    },
    Allowed {
        class: "ArgUtil",
        member: "validateNonNegative",
        class_side: true,
        reason: "ruling 9: ArgUtil is an empty class",
        sent_sites: 0,
    },
    Allowed {
        class: "ArgUtil",
        member: "validateNumber",
        class_side: true,
        reason: "ruling 9: ArgUtil is an empty class",
        sent_sites: 0,
    },
    Allowed {
        class: "ArgUtil",
        member: "validateNumberRange",
        class_side: true,
        reason: "ruling 9: ArgUtil is an empty class",
        sent_sites: 0,
    },
    Allowed {
        class: "ArgUtil",
        member: "validatePosition",
        class_side: true,
        reason: "ruling 9: ArgUtil is an empty class",
        sent_sites: 0,
    },
    Allowed {
        class: "ArgUtil",
        member: "validatePositive",
        class_side: true,
        reason: "ruling 9: ArgUtil is an empty class",
        sent_sites: 0,
    },
    Allowed {
        class: "ArgUtil",
        member: "validateWhole",
        class_side: true,
        reason: "ruling 9: ArgUtil is an empty class",
        sent_sites: 0,
    },
    Allowed {
        class: "ArgUtil",
        member: "validateWholeRange",
        class_side: true,
        reason: "ruling 9: ArgUtil is an empty class",
        sent_sites: 0,
    },
    Allowed {
        class: "DateTime",
        member: "convertDate",
        class_side: false,
        reason: "private helper; interrexx's DateTime converts with the DATE builtin",
        sent_sites: 0,
    },
    Allowed {
        class: "DateTime",
        member: "convertTime",
        class_side: false,
        reason: "private helper; interrexx's DateTime converts with the TIME builtin",
        sent_sites: 0,
    },
    Allowed {
        class: "File",
        member: "createPath",
        class_side: false,
        reason: "private; makeDirs recurses through parentFile (r[class.file.makedir])",
        sent_sites: 0,
    },
    Allowed {
        class: "File",
        member: "normalizePathSyntax",
        class_side: false,
        reason: "private; init forms the path itself (r[class.file.new])",
        sent_sites: 0,
    },
    Allowed {
        class: "File",
        member: "qualifiedPath",
        class_side: false,
        reason: "private; the absolute path is an instance variable of file.rex",
        sent_sites: 0,
    },
    Allowed {
        class: "File",
        member: "readImpl",
        class_side: true,
        reason: "private; readChars and readLines each read through a Stream",
        sent_sites: 0,
    },
];

fn main() {
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    println!("cargo::rerun-if-changed=rexx");
    let mut inputs: Vec<Vec<u8>> = Vec::new();
    for source in SOURCES {
        println!("cargo::rerun-if-changed={}", source.path);
        let path = Path::new(source.path);
        let bytes =
            std::fs::read(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        let actual = sha256_hex(&bytes);
        if actual != source.sha256 {
            panic!(
                "{} has sha256 {actual}, and this crate pins {}. The interpreter's own \
                 .orx sources are read-only here; if one really did change upstream, the \
                 pin moves in the same commit that re-verifies the bootstrap against it.",
                path.display(),
                source.sha256
            );
        }
        inputs.push(bytes);
    }

    let replacement_texts: Vec<Vec<u8>> = REPLACEMENTS
        .iter()
        .map(|name| {
            let path = Path::new("rexx").join(name);
            std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
        })
        .collect();
    let replacements: Vec<Replacement<'_>> = REPLACEMENTS
        .iter()
        .zip(&replacement_texts)
        .map(|(file, text)| Replacement { file, text })
        .collect();
    let dropped: Vec<Dropped<'_>> = ALLOW_DROPPED
        .iter()
        .map(|entry| Dropped {
            class: entry.class,
            member: entry.member,
            class_side: entry.class_side,
            reason: entry.reason,
            sent_sites: entry.sent_sites,
        })
        .collect();
    let hosts: Vec<Host<'_>> = SOURCES
        .iter()
        .zip(&inputs)
        .map(|(source, bytes)| Host {
            name: source.name,
            bytes,
        })
        .collect();
    let spliced = splice::splice_all(&hosts, &replacements, &dropped)
        .unwrap_or_else(|error| panic!("the library splice is refused: {error}"));

    let mut generated = String::from(
        "// Generated by build.rs from the interpreter's own sources and this\n\
         // crate's replacements. Do not edit.\n\
         pub static PROGRAMS: &[Program] = &[\n",
    );
    let mut table = String::from("pub static SPLICES: &[SpliceRange] = &[\n");
    for (source, spliced) in SOURCES.iter().zip(&spliced) {
        let built = out.join(format!("{}.spliced", source.name));
        std::fs::write(&built, &spliced.bytes)
            .unwrap_or_else(|e| panic!("cannot write {}: {e}", built.display()));
        writeln!(
            generated,
            "    Program {{ name: {:?}, source: include_bytes!({:?}), sha256: {:?} }},",
            source.name, built, source.sha256,
        )
        .expect("writing to a String cannot fail");
        for splice in &spliced.splices {
            writeln!(
                table,
                "    SpliceRange {{ program: {:?}, class: {:?}, file: {:?}, start: {}, end: {}, \
                 first_line: {} }},",
                source.name,
                splice.class,
                splice.file,
                splice.range.start,
                splice.range.end,
                splice.first_line,
            )
            .expect("writing to a String cannot fail");
        }
    }
    generated.push_str("];\n");
    table.push_str("];\n");
    generated.push_str(&table);
    std::fs::write(out.join("programs.rs"), generated).expect("writing programs.rs");
}
