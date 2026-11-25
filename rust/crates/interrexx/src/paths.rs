//! Turning a name a program wrote into the one path this interpreter opens.
//! Every consumer -- the streams, `.File`, the `Sys*` routines, `QUALIFY`, the
//! external-routine search and a command's working directory -- goes through
//! here, so all of them answer the same question the same way and none reads
//! the process's own directory.

use std::ffi::OsStr;
use std::path::Path;

/// Borrows an OS string from Rexx bytes. Unix accepts arbitrary bytes;
/// Windows files and command-line arguments use UTF-8 at the Rexx boundary.
pub(crate) fn os_str(bytes: &[u8]) -> Option<&OsStr> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt as _;
        Some(OsStr::from_bytes(bytes))
    }
    #[cfg(windows)]
    {
        std::str::from_utf8(bytes).ok().map(OsStr::new)
    }
}

/// A filesystem path. A NUL is rejected by filesystem operations on both
/// hosts, so an invalid UTF-8 name on Windows cannot open a different file.
pub(crate) fn os_path(path: &[u8]) -> &Path {
    Path::new(os_str(path).unwrap_or(OsStr::new("\0")))
}

/// `path` made absolute against `cwd` and reduced to one spelling: the
/// canonical name followed by path normalisation. Purely lexical, so that a
/// path that does not exist still normalises. Each `..` removes the component
/// before it, which the reference loop does not do after a doubled slash or
/// after a component starting with `.` (Deviation 34). A name that ends with a
/// slash keeps it, since `file/` names a directory and not the file `file`
/// (`r[bif.qualify.normal]`). A name that is not UTF-8 keeps its bytes.
#[cfg(unix)]
pub(crate) fn normalize_bytes(path: &[u8], cwd: &[u8]) -> Vec<u8> {
    let mut absolute = Vec::with_capacity(cwd.len() + path.len() + 1);
    if path.first() != Some(&b'/') {
        absolute.extend_from_slice(cwd);
        absolute.push(b'/');
    }
    absolute.extend_from_slice(path);
    let mut parts: Vec<&[u8]> = Vec::new();
    for part in absolute.split(|byte| *byte == b'/') {
        match part {
            b"" | b"." => {}
            b".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    let mut out = Vec::with_capacity(absolute.len());
    for part in &parts {
        out.push(b'/');
        out.extend_from_slice(part);
    }
    if out.is_empty() || path.last() == Some(&b'/') {
        out.push(b'/');
    }
    out
}

/// Qualifies a Windows path against the interpreter's selected directory.
#[cfg(windows)]
pub(crate) fn normalize_bytes(path: &[u8], cwd: &[u8]) -> Vec<u8> {
    use std::path::{Component, PathBuf, Prefix};

    let (Some(path_name), Some(cwd_name)) = (os_str(path), os_str(cwd)) else {
        return Vec::new();
    };
    let path_name = Path::new(path_name);
    let cwd = Path::new(cwd_name);
    let prefix = |part| match part {
        Some(Component::Prefix(prefix)) => Some(prefix.kind()),
        _ => None,
    };
    let drive = prefix(path_name.components().next());
    if matches!(
        drive,
        Some(Prefix::Verbatim(_) | Prefix::VerbatimDisk(_) | Prefix::VerbatimUNC(..))
    ) {
        return path.to_vec();
    }
    let input = if let (false, Some(Prefix::Disk(drive))) = (path_name.has_root(), drive) {
        // A drive-relative name uses the interpreter's directory only if it
        // is on that drive. Never consult the process's per-drive directory.
        if prefix(cwd.components().next()) != Some(Prefix::Disk(drive)) {
            return Vec::new();
        }
        let mut components = path_name.components();
        components.next();
        cwd.join(components.as_path())
    } else {
        cwd.join(path_name)
    };
    if !input.is_absolute() {
        return Vec::new();
    }
    let mut result = PathBuf::new();
    for component in input.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            other => result.push(other.as_os_str()),
        }
    }
    let mut out = result.as_os_str().as_encoded_bytes().to_vec();
    if path.last().is_some_and(|end| matches!(end, b'/' | b'\\')) && !out.ends_with(b"\\") {
        out.push(b'\\');
    }
    out
}

/// Relativize two spellings on the same Windows root without folding names.
/// Windows directories can have distinct children whose names differ by case.
#[cfg(windows)]
pub(crate) fn relative_windows_path(base: &Path, target: &Path) -> Option<std::path::PathBuf> {
    use std::path::{Component, PathBuf};

    let mut base = base.components().peekable();
    let mut target = target.components().peekable();
    match (base.next(), target.next()) {
        (Some(Component::Prefix(left)), Some(Component::Prefix(right))) if left == right => {}
        _ => return None,
    }
    if base.next() != Some(Component::RootDir) || target.next() != Some(Component::RootDir) {
        return None;
    }
    while let (Some(parent), Some(part)) = (base.peek(), target.peek()) {
        if parent != part {
            break;
        }
        base.next();
        target.next();
    }
    let mut relative = PathBuf::new();
    for _ in base {
        relative.push("..");
    }
    for component in target {
        relative.push(component.as_os_str());
    }
    if relative.as_os_str().is_empty() {
        relative.push(".");
    }
    Some(relative)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn normalize(path: &str, cwd: &str) -> String {
        String::from_utf8(normalize_bytes(path.as_bytes(), cwd.as_bytes()))
            .expect("UTF-8 in, UTF-8 out")
    }

    /// `normalizePathName`'s reductions, each with its expected answer.
    #[test]
    fn normalize_makes_one_spelling_of_a_path() {
        assert_eq!(normalize("lib.rex", "/a/b"), "/a/b/lib.rex");
        assert_eq!(normalize("./lib.rex", "/a/b"), "/a/b/lib.rex");
        assert_eq!(normalize("../lib.rex", "/a/b"), "/a/lib.rex");
        assert_eq!(normalize("/a//b/./c/../d", "/x"), "/a/b/d");
        // A trailing slash names a directory, so it stays.
        assert_eq!(normalize("/a/b/", "/x"), "/a/b/");
        assert_eq!(normalize("/..", "/x"), "/");
        // `..b` is a file name, not a parent reference.
        assert_eq!(normalize("/a/..b", "/x"), "/a/..b");
    }

    /// Deviation 34: a `..` removes the component before it, after a doubled
    /// slash and after a component starting with `.` too. The reference
    /// normalisation answers the second column, as `QUALIFY` shows, which names
    /// a different file.
    #[test]
    fn a_parent_reference_removes_one_component_where_the_oracle_does_not() {
        for (path, oracle, this_crate) in [
            ("/a/b//../c", "/a/b/c", "/a/c"),
            ("/a/b///../c", "/a/b/c", "/a/c"),
            ("/a/b/.//../c", "/a/b/c", "/a/c"),
            ("/a/b//..", "/a/b", "/a"),
            ("/a/b/c/..//../d", "/a/b/d", "/a/d"),
            ("/a/b/c//../../d", "/a/b/d", "/a/d"),
            ("/a/.hidden/../c", "/c", "/a/c"),
            ("/a/.config/../x", "/x", "/a/x"),
            ("/a/b/..x/../c", "/a/c", "/a/b/c"),
            ("/a/b/.../../c", "/a/c", "/a/b/c"),
        ] {
            assert_ne!(oracle, this_crate);
            assert_eq!(normalize(path, "/x"), this_crate, "{path}");
        }
    }

    /// A name that is not UTF-8 keeps its bytes.
    #[test]
    fn normalize_bytes_keeps_bytes_that_are_not_utf8() {
        assert_eq!(normalize_bytes(b"a/\xff/../\xfe", b"/x/"), b"/x/a/\xfe");
        assert_eq!(normalize_bytes(b"/\xff/", b"/x"), b"/\xff/");
    }
}

#[cfg(all(test, windows))]
mod windows_tests {
    #[test]
    fn relative_path_keeps_case_only_siblings_distinct() {
        use super::{Path, relative_windows_path};

        let base = Path::new(r"\\?\C:\parent\Foo");
        let target = Path::new(r"\\?\C:\parent\foo\file.txt");
        assert_eq!(
            relative_windows_path(base, target).as_deref(),
            Some(Path::new(r"..\foo\file.txt"))
        );
    }

    use super::*;

    #[test]
    fn qualifies_windows_names_against_the_interpreters_directory() {
        let cwd = br"C:\work\src";
        assert_eq!(
            normalize_bytes(br"../hello.rex", cwd),
            br"C:\work\hello.rex"
        );
        assert_eq!(normalize_bytes(br"\hello.rex", cwd), br"C:\hello.rex");
        assert_eq!(
            normalize_bytes(br"C:hello.rex", cwd),
            br"C:\work\src\hello.rex"
        );
        assert!(normalize_bytes(br"D:hello.rex", cwd).is_empty());
        assert_eq!(
            normalize_bytes(br"\\server\share\one\..\dir\", cwd),
            br"\\server\share\dir\"
        );
        assert!(os_str(b"bad\xffname").is_none());
    }
}
