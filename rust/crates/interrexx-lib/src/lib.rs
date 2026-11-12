//! The Rexx-written part of the interpreter's own library: one program, run
//! at startup, built from this crate's `rexx/` sources.

/// The library program.
pub struct Program {
    /// The name it runs under.
    pub name: &'static str,
    /// Its bytes: the `rexx/` sources, one after another.
    pub source: &'static [u8],
    /// `source` parsed when this crate was built, as
    /// `interrexx_parse::program_image` writes it; empty where it did not
    /// parse.
    pub image: &'static [u8],
}

/// Where one source file's text sits in [`LIBRARY`]'s `source`.
pub struct FileRange {
    /// The file, under this crate's `rexx/`.
    pub file: &'static str,
    /// The byte range in the program's `source`.
    pub start: usize,
    pub end: usize,
}

include!(concat!(env!("OUT_DIR"), "/library.rs"));

mod lines;

/// The source file whose text holds byte `byte` of the library program.
pub fn file_at(byte: usize) -> Option<&'static str> {
    file_range_at(FILES, byte).map(|range| range.file)
}

fn file_range_at(files: &[FileRange], byte: usize) -> Option<&FileRange> {
    files
        .iter()
        .find(|range| (range.start..range.end).contains(&byte))
}

/// Where byte `byte` of the library program is, as `file:line` in its own
/// source file. Names and numbers only, never source text.
pub fn site(byte: usize) -> String {
    site_in(LIBRARY.name, LIBRARY.source, FILES, byte)
}

fn site_in(name: &str, source: &[u8], files: &[FileRange], byte: usize) -> String {
    let line = lines::line_at(source, byte);
    match file_range_at(files, byte) {
        Some(range) => {
            let first = lines::line_at(source, range.start);
            format!("{}:{}", range.file, 1 + line - first)
        }
        None => format!("{name}:{line}"),
    }
}

/// What a parse failure of the library program reports: where, and the error
/// code, and nothing of the source.
pub fn describe_parse_error(byte: usize, code: u16, sub: u16) -> String {
    format!("{}: {code}.{sub}", site(byte))
}

/// The environment variable that turns [`debug_frame`] on, in a build with
/// debug assertions.
pub const DEBUG_VARIABLE: &str = "INTERREXX_LIBRARY_DEBUG";

/// Where the sources are, for [`debug_frame`].
const REXX_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/rexx");

/// For a library frame at `line` of the library program: `<file>:<line in
/// file>` and that line of the source file. Always `None` unless the build
/// has debug assertions and [`DEBUG_VARIABLE`] is set, so a release build
/// never reads the sources.
pub fn debug_frame(line: usize) -> Option<String> {
    if !debug_on(std::env::var_os(DEBUG_VARIABLE).is_some()) {
        return None;
    }
    debug_frame_in(
        LIBRARY.name,
        LIBRARY.source,
        FILES,
        line,
        std::path::Path::new(REXX_DIR),
    )
}

/// Whether [`debug_frame`] may read the sources: only in a build with debug
/// assertions, and only with the variable set.
fn debug_on(variable_set: bool) -> bool {
    cfg!(debug_assertions) && variable_set
}

fn debug_frame_in(
    name: &str,
    source: &[u8],
    files: &[FileRange],
    line: usize,
    dir: &std::path::Path,
) -> Option<String> {
    let byte = lines::line_start(source, line)?;
    let site = site_in(name, source, files, byte);
    let (file, in_file) = site.rsplit_once(':')?;
    if file == name {
        return None;
    }
    let in_file: usize = in_file.parse().ok()?;
    let text = std::fs::read(dir.join(file)).ok()?;
    // The same line ends as the rest of the crate counts.
    let from = lines::line_start(&text, in_file)?;
    let rest = &text[from..];
    let end = rest
        .iter()
        .position(|&b| b == b'\n' || b == b'\r')
        .unwrap_or(rest.len());
    let shown = String::from_utf8_lossy(&rest[..end]).into_owned();
    Some(format!("{site}: {shown}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A frame shows its source file's line, counted from that file's first
    /// line; a line past every file stays sourceless.
    #[test]
    fn the_debug_frame_shows_the_files_own_line() {
        let dir = std::env::temp_dir().join(format!("interrexx-lib-debug-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.rex"), "a1\na2\n").unwrap();
        std::fs::write(dir.join("b.rex"), "one\ntwo\nthree\n").unwrap();
        // a.rex's two lines, then b.rex's three, then a line of neither.
        let source = b"a1\na2\none\ntwo\nthree\nx\n";
        let files = [
            FileRange {
                file: "a.rex",
                start: 0,
                end: 6,
            },
            FileRange {
                file: "b.rex",
                start: 6,
                end: 20,
            },
        ];
        let shown = |line| debug_frame_in("P", source, &files, line, &dir);
        assert_eq!(shown(1).as_deref(), Some("a.rex:1: a1"));
        assert_eq!(shown(2).as_deref(), Some("a.rex:2: a2"));
        assert_eq!(shown(3).as_deref(), Some("b.rex:1: one"));
        assert_eq!(shown(5).as_deref(), Some("b.rex:3: three"));
        assert_eq!(shown(6), None);
        assert_eq!(shown(9), None);
        // A source file with CR line ends shows the same lines.
        std::fs::write(dir.join("b.rex"), "one\rtwo\r\nthree").unwrap();
        assert_eq!(shown(4).as_deref(), Some("b.rex:2: two"));
        assert_eq!(shown(5).as_deref(), Some("b.rex:3: three"));
        assert_eq!(site_in("P", source, &files, 7), "b.rex:1");
        assert_eq!(site_in("P", source, &[], 7), "P:3");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// The switch is off unless the variable is set (and in a release build
    /// it is off regardless).
    #[test]
    fn the_debug_gate_needs_the_variable() {
        assert!(!debug_on(false));
        assert_eq!(debug_on(true), cfg!(debug_assertions));
    }

    /// The files cover the whole program, in order and without a gap, and
    /// each starts on a line of its own.
    #[test]
    fn the_files_cover_the_program() {
        assert!(!LIBRARY.source.is_empty());
        let mut at = 0;
        for range in FILES {
            assert_eq!(
                range.start, at,
                "{} does not start where the last ended",
                range.file
            );
            assert!(range.end > range.start, "{} is empty", range.file);
            assert!(
                range.start == 0 || LIBRARY.source[range.start - 1] == b'\n',
                "{} does not start a line",
                range.file
            );
            at = range.end;
        }
        assert_eq!(at, LIBRARY.source.len());
        assert_eq!(file_at(0), Some(FILES[0].file));
        assert_eq!(file_at(LIBRARY.source.len()), None);
    }
}
