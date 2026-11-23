use std::fs;
use std::os::unix::ffi::OsStrExt as _;
use std::os::unix::fs::PermissionsExt as _;
use std::path::PathBuf;

use rustix::fs::FileType;

use super::{TreeOptions, entries, entry_line, matches};

/// A directory of this run's own under the system's temporary directory,
/// removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Scratch {
        use std::hash::BuildHasher as _;
        let unique = std::hash::RandomState::new().hash_one(std::process::id());
        let path = std::env::temp_dir().join(format!(
            "rexxutil-tree-{}-{unique:016x}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .unwrap_or_else(|e| panic!("cannot create {}: {e}", path.display()));
        Scratch(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// **A dangling link is listed as a link, with the link's own size**, and
/// counts as a file for `F` and `D`. The oracle's `stat64` fails on it, as on
/// any name `stat` fails on (a dangling link, `ELOOP`), and the line carries
/// whatever its buffer last held (`corpus/oracle-crashes.txt` entry 40), so
/// this is this crate's own answer. The regular file beside it
/// is the control that the listing reached the directory.
#[test]
fn a_dangling_link_is_listed_by_its_own_status() {
    let scratch = Scratch::new();
    fs::write(scratch.0.join("plain"), b"abc").expect("a file");
    std::os::unix::fs::symlink("nowhere", scratch.0.join("dangling")).expect("a link");
    let mut directory = scratch.0.as_os_str().as_bytes().to_vec();
    directory.push(b'/');

    let found = entries(&directory, |_| true);
    let kind = |wanted: &[u8]| {
        let (_, stat) = found
            .iter()
            .find(|(name, _)| name == wanted)
            .unwrap_or_else(|| panic!("{} is listed", String::from_utf8_lossy(wanted)));
        (FileType::from_raw_mode(stat.st_mode), stat.st_size)
    };
    assert_eq!(kind(b"plain"), (FileType::RegularFile, 3));
    assert_eq!(kind(b"dangling"), (FileType::Symlink, 7));

    let (_, stat) = found
        .iter()
        .find(|(name, _)| name == b"dangling")
        .expect("listed");
    let options = TreeOptions::parse(b"F").expect("valid");
    let zone = tz::TimeZone::utc();
    let line = entry_line(options, &zone, b"/x/dangling".to_vec(), stat).expect("a file");
    // A link's own permissions are the host's: Linux gives every link all of them, Apple's
    // systems what the creating process's umask leaves. The expectation is the link's own
    // `lstat`, so it holds under any umask.
    let bits = fs::symlink_metadata(scratch.0.join("dangling"))
        .expect("the link's own status")
        .permissions()
        .mode();
    let mode: String = std::iter::once('l')
        .chain((0..9).rev().map(|bit| {
            if bits & (1 << bit) == 0 {
                '-'
            } else {
                ['x', 'w', 'r'][bit % 3]
            }
        }))
        .collect();
    assert!(
        line.ends_with(format!("           7  {mode}  /x/dangling").as_bytes()),
        "{}",
        String::from_utf8_lossy(&line)
    );
    let directories = TreeOptions::parse(b"D").expect("valid");
    assert!(entry_line(directories, &zone, b"/x/dangling".to_vec(), stat).is_none());
}

/// Bracket forms the corpus witness does not reach: `^` negation, a `]`
/// first, an unterminated `[`, a class, an unknown class, and a folded range
/// whose ends straddle the letters, which `FNM_CASEFOLD` folds to lower case.
#[test]
fn bracket_expressions_match_as_fnmatch_does() {
    assert!(matches(b"[^a]x", b"bx", false));
    assert!(!matches(b"[^a]x", b"ax", false));
    assert!(matches(b"[]x]", b"]", false));
    assert!(matches(b"a[", b"a[", false));
    assert!(!matches(b"a[", b"ab", false));
    assert!(matches(b"[[:digit:]]*", b"7up", false));
    assert!(!matches(b"[[:digit:]]*", b"up", false));
    assert!(!matches(b"[[:nosuch:]a]", b"a", false));
    assert!(matches(b"[_-a]", b"`", true));
    assert!(matches(b"[_-a]", b"A", true));
    assert!(!matches(b"[_-a]", b"B", true));
}

/// **A spec whose `~` does not resolve searches nothing when more than one
/// component follows its slash**, as `util.sysfiletree.spec.dir` says, for
/// `HOME` unset as for a user no account has: the directory part qualifies
/// to nothing, and `/` is not searched for the last component in its place.
/// The adjacent success is one component after the slash, searched in `/`.
#[test]
fn an_unresolved_tilde_with_a_nested_path_matches_nothing() {
    let source = b"'unset HOME'\n\
        call SysFileTree '~/tmp', 'a.', 'B'\n\
        call SysFileTree '~/no-such-dir/tmp', 'b.', 'B'\n\
        call SysFileTree '~spec_tree_nouser/no-such-dir/tmp', 'c.', 'B'\n\
        say a.0 b.0 c.0\n";
    let outcome = crate::run_program("/t.rex", source.to_vec(), crate::Invocation::none());
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(String::from_utf8_lossy(&outcome.stdout), "1 0 0\n");
}
