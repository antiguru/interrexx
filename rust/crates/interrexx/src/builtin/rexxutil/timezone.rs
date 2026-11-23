use crate::Interp;

/// The zone the program's own `TZ` names, resolved as glibc's `tzset` does:
/// unset is `/etc/localtime`, a name is a TZif file under the zoneinfo
/// directories or a path after `:`, anything else a POSIX rule, and what none
/// of these reads, an empty `TZ` among them, is UTC. Read on every call, as
/// `localtime` and `mktime` read it.
pub(crate) fn program_zone(interp: &Interp) -> tz::TimeZone {
    let named = match interp.env_get(b"TZ") {
        None => tz::TimeZone::local(),
        Some(text) => match std::str::from_utf8(text) {
            Ok(text) => tz::TimeZone::from_posix_tz(text),
            Err(_) => return tz::TimeZone::utc(),
        },
    };
    named.unwrap_or_else(|_| tz::TimeZone::utc())
}
