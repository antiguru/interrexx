//! The Rexx packages shipped inside this crate, which a `::REQUIRES` finds
//! after every search route has missed (Phase 10 PR 4 plan, "Packages on
//! disk"): the rewritten packages of the extension libraries this crate runs
//! in Rust.

/// One shipped package.
pub(crate) struct Shipped {
    /// The file name a `::REQUIRES` names it by.
    pub(crate) name: &'static str,
    /// The package's source, from `packages/`.
    pub(crate) source: &'static [u8],
}

impl Shipped {
    /// The path the package is known by: its name in the interpreter's own
    /// directory, where the oracle installs its copy, or the bare name where
    /// that directory is unknown.
    pub(crate) fn path(&self) -> Vec<u8> {
        use std::os::unix::ffi::OsStrExt as _;
        match std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|dir| dir.join(self.name)))
        {
            Some(path) => path.as_os_str().as_bytes().to_vec(),
            None => self.name.as_bytes().to_vec(),
        }
    }
}

/// Every shipped package.
static SHIPPED: &[Shipped] = &[
    Shipped {
        name: "rxregexp.cls",
        source: include_bytes!("../packages/rxregexp.cls"),
    },
    Shipped {
        name: "socket.cls",
        source: include_bytes!("../packages/socket.cls"),
    },
    Shipped {
        name: "streamsocket.cls",
        source: include_bytes!("../packages/streamsocket.cls"),
    },
    Shipped {
        name: "mime.cls",
        source: include_bytes!("../packages/mime.cls"),
    },
    Shipped {
        name: "smtp.cls",
        source: include_bytes!("../packages/smtp.cls"),
    },
];

/// The shipped package a `::REQUIRES` of `name` finds: its file name, or that
/// name without the `.cls` the search appends, as written or in lower case,
/// as the file search tries them.
pub(crate) fn lookup(name: &[u8]) -> Option<&'static Shipped> {
    let lower = name.to_ascii_lowercase();
    SHIPPED.iter().find(|package| {
        let full = package.name.as_bytes();
        let bare = full.strip_suffix(b".cls").unwrap_or(full);
        [name, &lower[..]]
            .iter()
            .any(|candidate| *candidate == full || *candidate == bare)
    })
}
