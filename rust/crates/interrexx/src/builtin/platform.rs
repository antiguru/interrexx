//! The builtins that read or move the interpreter's own platform state:
//! `QUALIFY`, `USERID`, `SETLOCAL`, `ENDLOCAL`, and the external-pool halves of
//! `VALUE`. Every one of them goes through `Interp`'s environment and current
//! directory rather than the process's -- see `tests/no_process_state.rs`.

use interrexx_core::ObjRef;

use super::rexxutil::{self, stems::MAX_WHOLENUMBER};
use super::{Args, arg, required_string};
use crate::environment::EnvScope;
use crate::error::Raised;
use crate::{Failure, Interp};

/// `QUALIFY(name)`: the name resolved against the interpreter's current
/// directory and reduced to one spelling, whether or not it exists (stream-name
/// qualification).
pub(crate) fn qualify(
    interp: &mut Interp,
    _name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let text = required_string(interp, args, 1);
    let qualified = qualify_name(interp, &text);
    Ok(interp.text_built(qualified))
}

/// Stream-name qualification: `name` up to its first NUL, as the C string, with
/// a leading `~` or `~user` resolved, made absolute against the interpreter's
/// current directory and reduced by [`crate::paths::normalize_bytes`]. The null
/// string where qualification fails: an empty name, a `~user` naming no account,
/// or a home directory that is not absolute.
pub(crate) fn qualify_name(interp: &Interp, name: &[u8]) -> Vec<u8> {
    let name = match name.iter().position(|byte| *byte == 0) {
        Some(end) => &name[..end],
        None => name,
    };
    if name.is_empty() {
        return Vec::new();
    }
    if name.first() == Some(&b'~') {
        return match expand_tilde(interp, name) {
            Some(expanded) if expanded.first() == Some(&b'/') => {
                crate::paths::normalize_bytes(&expanded, b"/")
            }
            _ => Vec::new(),
        };
    }
    crate::paths::normalize_bytes(name, interp.cwd_bytes())
}

/// A leading `~` replaced by the environment's
/// `HOME`, a leading `~user` by that account's home directory, and `None`
/// where no account has that name.
pub(crate) fn expand_tilde(interp: &Interp, name: &[u8]) -> Option<Vec<u8>> {
    let rest = &name[1..];
    if rest.is_empty() || rest[0] == b'/' {
        let mut out = interp.env_get(b"HOME").unwrap_or_default().to_vec();
        out.extend_from_slice(rest);
        return Some(out);
    }
    let slash = rest
        .iter()
        .position(|byte| *byte == b'/')
        .unwrap_or(rest.len());
    let mut out = passwd_home(&rest[..slash])?;
    out.extend_from_slice(&rest[slash..]);
    Some(out)
}

/// The home directory the system's user database gives the account `user`,
/// found as `getpwnam` finds it, so an account that only an NSS source
/// (LDAP, sssd, systemd-homed) holds is found too.
pub(crate) fn passwd_home(user: &[u8]) -> Option<Vec<u8>> {
    home_through(user, |name| {
        nix::unistd::User::from_name(name)
            .ok()
            .flatten()
            .map(|account| account.dir)
    })
}

/// [`passwd_home`] over `lookup`, which answers the home directory of the
/// account it is given the name of. A name that is not UTF-8 is no account's:
/// `lookup` takes text, as the user database's own names are.
fn home_through(
    user: &[u8],
    lookup: impl FnOnce(&str) -> Option<std::path::PathBuf>,
) -> Option<Vec<u8>> {
    use std::os::unix::ffi::OsStringExt as _;
    let name = std::str::from_utf8(user).ok()?;
    lookup(name).map(|home| home.into_os_string().into_vec())
}

/// `USERID()`: the effective user's login name, as the system's user database
/// gives it for the effective uid, so an account that only an NSS source (LDAP,
/// sssd, systemd-userdb) holds has its name too. An account the database does
/// not know answers the environment's own `USER` or `LOGNAME` instead; the
/// empty string is answered there.
pub(crate) fn userid(
    interp: &mut Interp,
    _name: &'static [u8],
    _args: Args<'_>,
) -> Result<ObjRef, Failure> {
    if let Some(name) = name_through(nix::unistd::geteuid().as_raw(), |uid| {
        nix::unistd::User::from_uid(nix::unistd::Uid::from_raw(uid))
            .ok()
            .flatten()
            .map(|account| account.name)
    }) {
        return Ok(interp.text_built(name));
    }
    for fallback in [b"USER".as_slice(), b"LOGNAME".as_slice()] {
        if let Some(value) = interp.env_get(fallback) {
            let value = value.to_vec();
            return Ok(interp.text_built(value));
        }
    }
    Ok(interp.text(b""))
}

/// The login name `lookup` gives the account with uid `uid`, as bytes.
fn name_through(uid: u32, lookup: impl FnOnce(u32) -> Option<String>) -> Option<Vec<u8>> {
    lookup(uid).map(String::into_bytes)
}

/// `SETLOCAL()`: saves the directory and the environment, answering 1.
pub(crate) fn setlocal(
    interp: &mut Interp,
    _name: &'static [u8],
    _args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let saved = interp.push_local_environment();
    Ok(interp.counted(usize::from(saved)))
}

/// `ENDLOCAL()`: restores the innermost saved pair, answering 1 when there was
/// one and 0 when there was not.
///
/// **This crate does not crash past the end** (oracle defect,
/// `corpus/oracle-crashes.txt` entry 10): an `ENDLOCAL` with nothing
/// outstanding dereferences null there once any `SETLOCAL` has run, and a
/// second restore in one process aborts in the allocator. Answering 0 is what
/// is answered before the first `SETLOCAL`, so that is the answer here, and no
/// differential case may run either program.
pub(crate) fn endlocal(
    interp: &mut Interp,
    _name: &'static [u8],
    _args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let restored = interp.pop_local_environment();
    Ok(interp.counted(usize::from(restored)))
}

/// `DIRECTORY([new])`: a native routine of the `REXX` package, not a builtin
/// which is why its arity errors are the
/// 88.9xx family. With an argument it moves the interpreter's own current
/// directory and answers the new one; a change it cannot make answers the null
/// string and moves nothing. With none it answers where it is.
pub(crate) fn directory(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    if args.len() > 1 {
        return Err(Raised::too_many_internal_arguments(1).into());
    }
    let Some(text) = rexxutil::optional_text(interp, args, 1)? else {
        let here = interp.cwd_bytes().to_vec();
        return Ok(interp.text_built(here));
    };
    if text.is_empty() {
        return Ok(interp.text(b""));
    }
    let qualified = qualify_name(interp, &text);
    let candidate = crate::paths::os_path(&qualified);
    if !std::fs::metadata(candidate).is_ok_and(|meta| meta.is_dir()) || !can_enter(candidate) {
        return Ok(interp.text(b""));
    }
    // `getcwd` answers the symlink-resolved path, so the answer is what the
    // filesystem calls the directory rather than the way it was named.
    let resolved = std::fs::canonicalize(candidate).unwrap_or_else(|_| candidate.to_path_buf());
    let answer = resolved.as_os_str().as_encoded_bytes().to_vec();
    interp.set_cwd(resolved);
    Ok(interp.text_built(answer))
}

/// Whether the process may make `directory` its current one, which is what
/// `chdir` asks: search permission, with the effective ids.
fn can_enter(directory: &std::path::Path) -> bool {
    rustix::fs::accessat(
        rustix::fs::CWD,
        directory,
        rustix::fs::Access::EXEC_OK,
        rustix::fs::AtFlags::EACCESS,
    )
    .is_ok()
}

/// The `FILESPEC` options, in the order the message lists them: `DELNP`.
const FILESPEC_OPTIONS: &[u8] = b"DELNP";

/// `FILESPEC(option, name)`: the piece of `name` that `option`'s first letter
/// selects. Purely textual -- the file need
/// not exist, and nothing here touches the file system. On unix the drive is
/// always the null string.
pub(crate) fn filespec(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    if args.len() > 2 {
        return Err(Raised::too_many_internal_arguments(2).into());
    }
    // Both are C strings, ending at a NUL, and the option is read for its
    // first character only.
    let option = rexxutil::required_text(interp, args, 1)?;
    let spec = rexxutil::required_c_string(interp, args, 2)?;
    let option = &option[..option.first().map_or(0, |first| usize::from(*first != 0))];
    let letter = option.first().map(u8::to_ascii_uppercase);
    let Some(letter) = letter.filter(|byte| FILESPEC_OPTIONS.contains(byte)) else {
        return Err(Raised::argument_not_in_list(
            b"FILESPEC",
            1,
            &String::from_utf8_lossy(FILESPEC_OPTIONS),
            option,
        )
        .into());
    };
    let cut = spec.iter().rposition(|byte| *byte == b'/');
    let answer: Vec<u8> = match letter {
        // Unix has no drive letter, and the answer is the null string.
        b'D' => Vec::new(),
        b'L' | b'P' => match cut {
            None => Vec::new(),
            Some(at) => spec[..=at].to_vec(),
        },
        b'N' => match cut {
            None => spec.clone(),
            Some(at) => spec[at + 1..].to_vec(),
        },
        // What follows the last dot of the *name*, wherever that dot sits:
        // measured, `filespec('E','/a/.hidden')` is `hidden`, not the null
        // string. `.File~extension` has the leading-dot rule this does not.
        b'E' => {
            let name = match cut {
                None => &spec[..],
                Some(at) => &spec[at + 1..],
            };
            match name.iter().rposition(|byte| *byte == b'.') {
                None => Vec::new(),
                Some(at) => name[at + 1..].to_vec(),
            }
        }
        other => unreachable!("{other} is not one of the options just checked"),
    };
    Ok(interp.text_built(answer))
}

/// `BEEP`'s bounds.
const MIN_FREQUENCY: i64 = 37;
const MAX_FREQUENCY: i64 = 32767;
const MIN_DURATION: i64 = 0;
const MAX_DURATION: i64 = 60000;

/// `BEEP(frequency, duration)`: writes one bell byte and answers the null
/// string. On this platform that reaches the terminal as a single `0x07` on
/// standard output: `beep(440,10)` writes it and answers `''`.
pub(crate) fn beep(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    if args.len() > 2 {
        return Err(Raised::too_many_internal_arguments(2).into());
    }
    // Both conversions, then frequency's range before duration's, and all
    // before the bell: `beep(1,1)` raises on the frequency and sounds nothing.
    let frequency = whole_number(interp, args, 1)?;
    let duration = whole_number(interp, args, 2)?;
    in_range(frequency, "frequency", MIN_FREQUENCY, MAX_FREQUENCY)?;
    in_range(duration, "duration", MIN_DURATION, MAX_DURATION)?;
    interp.write_out(&[0x07]);
    Ok(interp.text(b""))
}

/// A required `wholenumber_t` argument as the routine's marshalling converts
/// it: 88.901 where it is omitted,
/// 88.907 over the whole-number range where it is not a whole number.
fn whole_number(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    position: usize,
) -> Result<i64, Failure> {
    let value = rexxutil::required(args, position)?;
    if let Some(number) = rexxutil::stems::whole_value(interp, value, -MAX_WHOLENUMBER)? {
        return Ok(number);
    }
    let found = interp.string_value_text(value)?;
    Err(rexxutil::marshalled(Raised::native_argument_out_of_range(
        &position.to_string(),
        -MAX_WHOLENUMBER,
        MAX_WHOLENUMBER,
        &found,
    )))
}

/// `BEEP`'s own range check of a converted argument, which names it.
fn in_range(number: i64, argument: &str, minimum: i64, maximum: i64) -> Result<(), Failure> {
    if (minimum..=maximum).contains(&number) {
        return Ok(());
    }
    Err(Raised::native_argument_out_of_range(
        argument,
        minimum,
        maximum,
        number.to_string().as_bytes(),
    )
    .into())
}

/// `VALUE(name, [new], selector)`: the two external pools this crate answers.
/// The empty selector names `.environment` itself; `ENVIRONMENT`, caselessly,
/// names the process environment this interpreter holds. Any other selector is
/// 40.914, which is also what is answered on unix for every selector the value
/// function declines.
pub(crate) fn value_selector(
    interp: &mut Interp,
    args: Args<'_>,
    selector: &[u8],
) -> Result<ObjRef, Failure> {
    let name = required_string(interp, args, 1);
    let new = arg(args, 2);
    if selector.is_empty() {
        return environment_directory(interp, &name, new);
    }
    if selector.eq_ignore_ascii_case(b"ENVIRONMENT") {
        return process_environment(interp, &name, new);
    }
    // The value exit's pass at a selector nothing here knows (`BUILTIN(VALUE)`).
    if interp.exits.is_some()
        && let (Some(selector), Some(variable)) = (arg(args, 3), arg(args, 1))
        && let Some(value) = interp.value_exit(selector, variable, new)?
    {
        return Ok(value.unwrap_or(ObjRef::NIL));
    }
    Err(Raised::syntax(40, 914, vec![selector.to_vec()]).into())
}

/// The `''` selector: `.environment`'s own entry, under the upper-cased name
/// the directory stores. A name it does not hold reads as `.NAME` -- the same
/// answer `.NAME` itself gives, because that is the path it takes.
fn environment_directory(
    interp: &mut Interp,
    name: &[u8],
    new: Option<ObjRef>,
) -> Result<ObjRef, Failure> {
    let upper = name.to_ascii_uppercase();
    let mut dotted = Vec::with_capacity(upper.len() + 1);
    dotted.push(b'.');
    dotted.extend_from_slice(&upper);
    let old = interp.dot_variable(&dotted)?;
    // The write allocates, and `old` may be a fresh `.NAME` string nothing
    // else holds.
    interp.roots.activity_mut().push_temp(old);
    if let Some(value) = new {
        interp.set_directory_entry(EnvScope::Environment, &upper, value)?;
    }
    Ok(old)
}

/// The `ENVIRONMENT` selector: the interpreter's own environment. A name it
/// does not hold reads as the null string; `.nil` removes; any other value is
/// stored as its string. The name and the value are both cut at the first NUL,
/// because `setenv` takes C strings and the read-back stops there.
fn process_environment(
    interp: &mut Interp,
    name: &[u8],
    new: Option<ObjRef>,
) -> Result<ObjRef, Failure> {
    let name = name.split(|byte| *byte == 0).next().unwrap_or_default();
    let old = interp.env_get(name).unwrap_or_default().to_vec();
    if let Some(value) = new {
        if value == ObjRef::NIL {
            interp.env_set(name, None);
        } else {
            let text = interp.to_text(value).into_owned();
            let stored = match text.iter().position(|byte| *byte == 0) {
                Some(at) => text[..at].to_vec(),
                None => text,
            };
            interp.env_set(name, Some(stored));
        }
    }
    Ok(interp.text_built(old))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{home_through, name_through, passwd_home};

    /// **A home directory comes from the user database lookup, not from
    /// `/etc/passwd`**: an account only an NSS source holds, which no file of
    /// this host lists, is found through the lookup it is given, and an account
    /// the lookup does not know is not found. The adjacent success is the
    /// real lookup, which finds `root` where `/etc/passwd` lists it.
    #[test]
    fn a_home_directory_comes_from_the_user_database_lookup() {
        let ldap_only = |name: &str| (name == "ldapuser").then(|| PathBuf::from("/home/ldap/u"));
        assert_eq!(
            home_through(b"ldapuser", ldap_only),
            Some(b"/home/ldap/u".to_vec())
        );
        assert_eq!(home_through(b"nobodyhere", ldap_only), None);
        assert_eq!(home_through(b"ldap\xffuser", ldap_only), None);
        let passwd = std::fs::read_to_string("/etc/passwd").expect("the passwd file");
        let listed = passwd
            .lines()
            .find_map(|line| line.strip_prefix("root:"))
            .and_then(|rest| rest.split(':').nth(4))
            .map(|home| home.as_bytes().to_vec());
        assert_eq!(passwd_home(b"root"), listed);
        assert_eq!(passwd_home(b"no-such-account-zq"), None);
    }

    /// **A login name comes from the user database lookup, not from
    /// `/etc/passwd`**: an account only an NSS source holds, which no file of
    /// this host lists, is named through the lookup it is given, and a uid the
    /// lookup does not know has no name. The adjacent success is the running
    /// user, whom `USERID()` names as the real lookup does.
    #[test]
    fn a_login_name_comes_from_the_user_database_lookup() {
        let ldap_only = |uid: u32| (uid == 424_242).then(|| "ldapuser".to_string());
        assert_eq!(name_through(424_242, ldap_only), Some(b"ldapuser".to_vec()));
        assert_eq!(name_through(7, ldap_only), None);
        let running = nix::unistd::User::from_uid(nix::unistd::geteuid())
            .expect("the lookup runs")
            .map(|account| account.name);
        let source = b"say userid()\n";
        let outcome = crate::run_program("/t.rex", source.to_vec(), crate::Invocation::none());
        assert_eq!(outcome.exit_code, 0);
        let said = String::from_utf8_lossy(&outcome.stdout).into_owned();
        match running {
            Some(name) => assert_eq!(said, format!("{name}\n")),
            None => assert_eq!(said.matches('\n').count(), 1),
        }
    }

    /// **A `~` with `HOME` unset qualifies to the null string, and `~/x` to
    /// `/x`**, the empty home read as no directory at all. The reference run
    /// applies `strlen` to the null `getenv("HOME")` there (oracle defect,
    /// `corpus/oracle-crashes.txt` entry 48), so this is the crate's own
    /// answer. The adjacent success is the same names with `HOME` set.
    #[test]
    fn a_tilde_with_home_unset_qualifies_without_a_home() {
        let source = b"call value 'HOME', '/h', 'ENVIRONMENT'\n\
            say qualify('~') qualify('~/x')\n\
            'unset HOME'\n\
            say '['qualify('~')']' qualify('~/x')\n";
        let outcome = crate::run_program("/t.rex", source.to_vec(), crate::Invocation::none());
        assert_eq!(outcome.exit_code, 0);
        assert_eq!(String::from_utf8_lossy(&outcome.stdout), "/h /h/x\n[] /x\n");
    }
}
