//! Requests and replies, the service's whole vocabulary.
//!
//! Names and items are bytes. A name of `None` is a NULL name at the C API. Flags stay raw numbers
//! because a bad flag is an answer, not a malformed request.

pub mod wire;

/// `RXQUEUE_FIFO`, the add flag.
pub const FIFO: u64 = 0;
/// `RXQUEUE_LIFO`, the add flag.
pub const LIFO: u64 = 1;
/// `RXQUEUE_NOWAIT`, the pull flag.
pub const NO_WAIT: u64 = 0;
/// `RXQUEUE_WAIT`, the pull flag.
pub const WAIT: u64 = 1;

/// Longest valid queue name, `MAX_QUEUE_NAME_LENGTH`.
pub const MAX_QUEUE_NAME: usize = 250;
/// Registration, library, procedure and macro names are truncated to this many bytes.
pub const MAX_NAME: usize = 255;

/// Return codes, by the values in `rexxapidefs.h`.
pub mod code {
    /// Success, in every family.
    pub const OK: u64 = 0;
    /// `RXQUEUE_STORAGE`.
    pub const QUEUE_STORAGE: u64 = 1;
    /// `RXQUEUE_BADQNAME`.
    pub const QUEUE_BAD_NAME: u64 = 5;
    /// `RXQUEUE_PRIORITY`.
    pub const QUEUE_PRIORITY: u64 = 6;
    /// `RXQUEUE_BADWAITFLAG`.
    pub const QUEUE_BAD_WAIT_FLAG: u64 = 7;
    /// `RXQUEUE_EMPTY`.
    pub const QUEUE_EMPTY: u64 = 8;
    /// `RXQUEUE_NOTREG`.
    pub const QUEUE_NOT_REGISTERED: u64 = 9;
    /// `RXQUEUE_ACCESS`.
    pub const QUEUE_ACCESS: u64 = 10;
    /// `RXSUBCOM_DUP`, also `RXFUNC_DEFINED`.
    pub const DUPLICATE: u64 = 10;
    /// `RXSUBCOM_NOTREG`, also `RXFUNC_NOTREG`.
    pub const NOT_REGISTERED: u64 = 30;
    /// `RXSUBCOM_NOCANDROP`.
    pub const NO_CAN_DROP: u64 = 40;
    /// `RXMACRO_NO_STORAGE`.
    pub const MACRO_NO_STORAGE: u64 = 1;
    /// `RXMACRO_NOT_FOUND`.
    pub const MACRO_NOT_FOUND: u64 = 2;
    /// `RXMACRO_FILE_ERROR`.
    pub const MACRO_FILE_ERROR: u64 = 5;
    /// `RXAPI_NORXAPI`.
    pub const NO_SERVICE: u64 = 1004;
}

/// The service's local time, as `struct tm` counts it. `hundredths` and `microseconds` are always
/// 0 at the API, so they are not carried.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Timestamp {
    /// Years since 1900.
    pub year: u16,
    /// 0 to 11.
    pub month: u16,
    /// 1 to 31.
    pub day: u16,
    /// 0 is Sunday.
    pub weekday: u16,
    /// 0-based.
    pub yearday: u16,
    pub hours: u16,
    pub minutes: u16,
    pub seconds: u16,
}

/// A queue item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub data: Vec<u8>,
    pub time: Timestamp,
}

/// Which of the independent registries a registration operation addresses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Registry {
    Function,
    Subcom,
    Exit,
}

/// A macro as held by the service.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Macro {
    pub name: Vec<u8>,
    pub image: Vec<u8>,
    pub position: u64,
}

/// One operation. Library and `...Exe` loading, macro translation and macrospace files stay in the
/// client, so `Registered` answers `RexxLoadSubcom` and the `RexxResolve` family, and
/// `SaveMacros`/`LoadMacros` carry a file's contents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    /// `RexxCreateSessionQueue`: attaches the caller to its session queue.
    CreateSessionQueue { session: u64 },
    /// `RexxDeleteSessionQueue`: repeatable, and leaves the queue alone.
    DeleteSessionQueue { session: u64 },
    /// `RexxCreateQueue`. `size` is the caller's buffer size.
    CreateQueue {
        session: u64,
        requested: Option<Vec<u8>>,
        size: u64,
    },
    /// `RexxOpenQueue`.
    OpenQueue { name: Option<Vec<u8>> },
    /// `RexxQueueExists`.
    QueueExists { name: Option<Vec<u8>> },
    /// `RexxDeleteQueue`.
    DeleteQueue { name: Option<Vec<u8>> },
    /// `RexxClearQueue`.
    ClearQueue { session: u64, name: Option<Vec<u8>> },
    /// `RexxQueryQueue`.
    QueryQueue { session: u64, name: Option<Vec<u8>> },
    /// `RexxAddQueue`.
    AddQueue {
        session: u64,
        name: Option<Vec<u8>>,
        data: Vec<u8>,
        flag: u64,
        time: Timestamp,
    },
    /// `RexxPullFromQueue` and `RexxPullQueue`.
    PullQueue {
        session: u64,
        name: Option<Vec<u8>>,
        wait: u64,
    },
    /// Ends the caller's pull `call`, which answers `Cancelled`. Any other call is left alone.
    CancelPull { call: u64 },
    /// A library registration, `RexxRegister*Dll`. `owner_only` is `RXSUBCOM_NONDROP`; the
    /// function registry always treats it as set.
    Register {
        registry: Registry,
        name: Vec<u8>,
        library: Vec<u8>,
        procedure: Vec<u8>,
        user_area: Option<[u8; 16]>,
        owner_only: bool,
    },
    /// `RexxQuerySubcom`, `RexxQueryExit` and `RexxQueryFunction`. An empty `library` is name-only.
    QueryRegistration {
        registry: Registry,
        name: Vec<u8>,
        library: Vec<u8>,
    },
    /// `RexxDeregisterSubcom`, `RexxDeregisterExit` and `RexxDeregisterFunction`. An empty
    /// `library` is name-only.
    Deregister {
        registry: Registry,
        name: Vec<u8>,
        library: Vec<u8>,
    },
    /// `RexxAddMacro`, after the client translated the file.
    AddMacro {
        name: Vec<u8>,
        image: Vec<u8>,
        position: u64,
    },
    /// `RexxDropMacro`.
    DropMacro { name: Vec<u8> },
    /// `RexxClearMacroSpace`.
    ClearMacroSpace,
    /// `RexxQueryMacro`.
    QueryMacro { name: Vec<u8> },
    /// `RexxReorderMacro`.
    ReorderMacro { name: Vec<u8>, position: u64 },
    /// `RexxResolveMacroFunction`.
    ResolveMacro { name: Vec<u8> },
    /// `RexxSaveMacroSpace`: the macros the client writes. `None` is everything.
    SaveMacros { names: Option<Vec<Vec<u8>>> },
    /// `RexxLoadMacroSpace`: the macros the client read. `None` loads them all.
    LoadMacros {
        names: Option<Vec<Vec<u8>>>,
        macros: Vec<Macro>,
    },
}

/// A registration as a query sees it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Registered {
    pub library: Vec<u8>,
    pub procedure: Vec<u8>,
    /// Zeros when none was supplied.
    pub user_area: [u8; 16],
}

/// The answer to a request. A `None` field is an output the API leaves untouched.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reply {
    Status(u64),
    QueueCreated {
        status: u64,
        name: Option<Vec<u8>>,
        duplicate: Option<bool>,
    },
    QueueOpened {
        status: u64,
        duplicate: Option<bool>,
    },
    QueueCount {
        status: u64,
        count: Option<u64>,
    },
    Pulled {
        status: u64,
        item: Option<Item>,
    },
    Registration {
        status: u64,
        registered: Option<Registered>,
    },
    MacroPosition {
        status: u64,
        position: u64,
    },
    MacroImage {
        status: u64,
        image: Option<Vec<u8>>,
    },
    Macros {
        status: u64,
        macros: Vec<Macro>,
    },
    /// A pull ended by `CancelPull`.
    Cancelled,
}
