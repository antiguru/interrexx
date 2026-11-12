//! The Rexx-written part of the interpreter's own library: the Rexx sources run
//! to finish building the image, embedded at build time from this repository's
//! tracked copies.

/// One embedded program.
pub struct Program {
    /// The name it is resolved by, which is what a `CALL` inside the bootstrap
    /// writes (a quoted program name followed by the package). Compared
    /// case-insensitively by [`lookup`], because program-name resolution is.
    pub name: &'static str,
    /// The file's bytes, exactly as tracked.
    pub source: &'static [u8],
    /// The sha256 `build.rs` checked the program's input against, lower-case
    /// hex. With no replacement the input and `source` are the same bytes.
    pub sha256: &'static str,
}

/// Where the text of one replaced class sits in a program's `source`.
pub struct SpliceRange {
    /// The program, by [`Program::name`].
    pub program: &'static str,
    pub class: &'static str,
    /// The replacement file, under this crate's `rexx/`.
    pub file: &'static str,
    /// The byte range in the program's `source`.
    pub start: usize,
    pub end: usize,
    /// The line of the class directive in `file`, 1-based.
    pub first_line: usize,
}

include!(concat!(env!("OUT_DIR"), "/programs.rs"));

mod lines;
#[cfg(test)]
mod sha256;
#[cfg(test)]
mod splice;

/// The program the bootstrap starts at: it is resolved and run with the Rexx
/// package as its one argument.
pub const ENTRY: &str = "CoreClasses.orx";

/// The embedded program a `CALL` target names, or `None` for a name that is
/// not one of these.
pub fn lookup(name: &str) -> Option<&'static Program> {
    PROGRAMS
        .iter()
        .find(|program| program.name.eq_ignore_ascii_case(name))
}

/// Where byte `byte` of `program`'s source is, as `file:line`: the program's
/// own name and line, or, inside a replaced class, the replacement file's name
/// and line. Names and numbers only, never source text.
pub fn site(program: &Program, byte: usize) -> String {
    site_in(program.name, program.source, SPLICES, byte)
}

fn site_in(name: &str, source: &[u8], splices: &[SpliceRange], byte: usize) -> String {
    let line = lines::line_at(source, byte);
    for splice in splices.iter().filter(|s| s.program == name) {
        if (splice.start..splice.end).contains(&byte) {
            let first = lines::line_at(source, splice.start);
            return format!("{}:{}", splice.file, splice.first_line + (line - first));
        }
    }
    format!("{name}:{line}")
}

/// What a parse failure of an embedded program reports: where, and the error
/// code, and nothing of the source.
pub fn describe_parse_error(program: &Program, byte: usize, code: u16, sub: u16) -> String {
    format!("{}: {code}.{sub}", site(program, byte))
}

/// The environment variable that turns [`debug_frame`] on, in a build with
/// debug assertions.
pub const DEBUG_VARIABLE: &str = "INTERREXX_LIBRARY_DEBUG";

/// Where the replacement sources are, for [`debug_frame`].
const REXX_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/rexx");

/// For a library frame at `line` of the embedded program named `program`:
/// `<file>:<line in file>` and that line of the replacement file, when the
/// line lies inside a replaced class. `None` for a line of the host's own
/// text, and always `None` unless the build has debug assertions and
/// [`DEBUG_VARIABLE`] is set, so a release build never reads the sources.
pub fn debug_frame(program: &str, line: usize) -> Option<String> {
    if !cfg!(debug_assertions) || std::env::var_os(DEBUG_VARIABLE).is_none() {
        return None;
    }
    let found = lookup(program)?;
    debug_frame_in(
        found.name,
        found.source,
        SPLICES,
        line,
        std::path::Path::new(REXX_DIR),
    )
}

fn debug_frame_in(
    name: &str,
    source: &[u8],
    splices: &[SpliceRange],
    line: usize,
    dir: &std::path::Path,
) -> Option<String> {
    let byte = lines::line_start(source, line)?;
    let site = site_in(name, source, splices, byte);
    let (file, in_file) = site.rsplit_once(':')?;
    if file == name {
        return None;
    }
    let in_file: usize = in_file.parse().ok()?;
    let text = std::fs::read(dir.join(file)).ok()?;
    let shown = text
        .split(|b| *b == b'\n')
        .nth(in_file.checked_sub(1)?)
        .map(|l| {
            String::from_utf8_lossy(l)
                .trim_end_matches('\r')
                .to_string()
        })?;
    Some(format!("{site}: {shown}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// With no replacement, every program is its input byte for byte, and no
    /// splice is recorded.
    #[test]
    fn with_no_replacement_every_program_is_its_input() {
        assert!(SPLICES.is_empty());
        for program in PROGRAMS {
            assert_eq!(
                crate::sha256::sha256_hex(program.source),
                program.sha256,
                "{}",
                program.name
            );
        }
    }

    /// A frame in a host's own text stays sourceless with the switch on, and a
    /// frame inside a replaced class shows the replacement file's line.
    #[test]
    fn the_debug_frame_shows_only_replacement_lines() {
        let dir = std::env::temp_dir().join(format!("interrexx-lib-debug-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("r.rex"), "one\ntwo\nthree\n").unwrap();
        // Host lines 1-2, then a spliced block of two lines (the replacement's
        // lines 2 and 3), then host lines.
        let source = b"h1\nh2\nB1\nB2\nh5\n";
        let splices = [SpliceRange {
            program: "P",
            class: "C",
            file: "r.rex",
            start: 6,
            end: 12,
            first_line: 2,
        }];
        let shown = |line| debug_frame_in("P", source, &splices, line, &dir);
        assert_eq!(shown(1), None);
        assert_eq!(shown(2), None);
        assert_eq!(shown(3).as_deref(), Some("r.rex:2: two"));
        assert_eq!(shown(4).as_deref(), Some("r.rex:3: three"));
        assert_eq!(shown(5), None);
        assert_eq!(shown(9), None);
        assert_eq!(site_in("P", source, &splices, 7), "r.rex:2");
        assert_eq!(site_in("P", source, &[], 7), "P:3");
        std::fs::remove_dir_all(&dir).unwrap();
        // The embedded host stays sourceless: no splice covers it.
        assert_eq!(
            debug_frame_in(ENTRY, lookup(ENTRY).unwrap().source, SPLICES, 10, &dir),
            None
        );
    }

    /// Every embedded source is non-empty and reachable by its own name.
    #[test]
    fn every_program_is_reachable_by_name() {
        for program in PROGRAMS {
            assert!(
                !program.source.is_empty(),
                "{} embedded as zero bytes",
                program.name
            );
            let found = lookup(program.name).expect("a program is reachable by its own name");
            assert_eq!(found.sha256, program.sha256);
        }
        assert!(lookup("nosuch.orx").is_none());
    }

    /// The entry point names one of the embedded programs. Without this,
    /// renaming a row leaves [`ENTRY`] pointing at nothing and the bootstrap
    /// fails at run time instead of at test time.
    #[test]
    fn the_entry_point_is_one_of_the_embedded_programs() {
        assert!(lookup(ENTRY).is_some(), "{ENTRY} is not embedded");
    }

    /// Every quoted `CALL` target in `program`, in source order.
    fn quoted_call_targets(program: &Program) -> Vec<String> {
        let text = String::from_utf8(program.source.to_vec()).expect("the source is UTF-8");
        let lower = text.to_ascii_lowercase();
        let mut called = Vec::new();
        let mut at = 0usize;
        while let Some(found) = lower[at..].find("call ") {
            let after = at + found + "call ".len();
            at = after;
            let rest = text[after..].trim_start();
            let Some(quoted) = rest.strip_prefix('\'') else {
                continue;
            };
            let Some(end) = quoted.find('\'') else {
                continue;
            };
            called.push(quoted[..end].to_string());
        }
        called
    }

    /// The entry program's two `CALL`s name the other two rows, and neither
    /// of those two calls anything embedded.
    #[test]
    fn the_entry_program_calls_exactly_the_other_embedded_programs() {
        let entry = lookup(ENTRY).expect("the entry point is embedded");
        let mut called = quoted_call_targets(entry);
        called.sort();
        let mut expected: Vec<String> = PROGRAMS
            .iter()
            .map(|p| p.name.to_string())
            .filter(|name| name != ENTRY)
            .collect();
        expected.sort();
        assert_eq!(called, expected);

        for program in PROGRAMS.iter().filter(|p| p.name != ENTRY) {
            for target in quoted_call_targets(program) {
                assert!(
                    lookup(&target).is_none(),
                    "{} calls the embedded {target}, so the bootstrap can recurse and \
                     `enter_library_program` needs a depth guard it does not have",
                    program.name
                );
            }
        }
    }
}
