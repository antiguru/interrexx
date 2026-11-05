//! What every client process shares: its session queue's handle and its connection.

use crate::transport::unix::{Unix, service_dir};

/// The environment variable a session queue's handle travels in.
pub const SESSION_VARIABLE: &str = "RXQUEUESESSION";

/// `value` as a session handle: hex, with or without `0x`, or `(nil)` for 0.
pub fn parse_session(value: &[u8]) -> Option<u64> {
    if value == b"(nil)" {
        return Some(0);
    }
    let digits = value
        .strip_prefix(b"0x")
        .or_else(|| value.strip_prefix(b"0X"))
        .unwrap_or(value);
    if digits.is_empty() {
        return None;
    }
    u64::from_str_radix(std::str::from_utf8(digits).ok()?, 16).ok()
}

/// `handle` as a child receives it.
pub fn session_value(handle: u64) -> Vec<u8> {
    format!("0x{handle:x}").into_bytes()
}

/// A process's session queue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Session {
    handle: u64,
    inherited: bool,
}

impl Session {
    /// The handle this process inherited in [`SESSION_VARIABLE`], else its own process id.
    pub fn of_process() -> Session {
        use std::os::unix::ffi::OsStrExt;
        match std::env::var_os(SESSION_VARIABLE).and_then(|value| parse_session(value.as_bytes())) {
            Some(handle) => Session {
                handle,
                inherited: true,
            },
            None => Session {
                handle: u64::from(std::process::id()),
                inherited: false,
            },
        }
    }

    pub fn handle(self) -> u64 {
        self.handle
    }

    /// Whether another process handed this one the handle.
    pub fn inherited(self) -> bool {
        self.inherited
    }
}

/// Connects to this user's service, starting it if nothing answers, or the API code for the
/// failure.
pub fn connect() -> Result<Unix, u64> {
    Unix::connect_or_start(&service_dir()).map_err(|error| error.status())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_values_parse_as_hex_with_or_without_the_prefix() {
        assert_eq!(parse_session(b"1234"), Some(0x1234));
        assert_eq!(parse_session(b"0x1234"), Some(0x1234));
        assert_eq!(parse_session(b"0"), Some(0));
        assert_eq!(parse_session(b"0x0"), Some(0));
        assert_eq!(parse_session(b"(nil)"), Some(0));
        assert_eq!(parse_session(b"garbage"), None);
        assert_eq!(parse_session(b""), None);
        assert_eq!(parse_session(&session_value(0x1f4a5f)), Some(0x1f4a5f));
    }
}
