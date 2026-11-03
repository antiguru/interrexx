//! The extension libraries this crate runs in Rust: `rxmath`, `rxregexp`,
//! `rxsock` and `rxunixsys` (`docs/superpowers/specs/extensions-behaviour.md`).
//!
//! A program sees each as the oracle's shared library: [`Interp::resolve_library`]
//! answers a library whose tables name the routines and methods with no
//! address ([`rexx_api::load::names_only`]) before any `dlopen`, so
//! `::REQUIRES ... LIBRARY`, `RxFuncAdd`, `Package~loadLibrary`, `EXTERNAL`
//! directives and `loadExternalRoutine`/`loadExternalMethod` bind, merge and
//! report it as any library. Where one of its rows is called, its body here
//! runs in place of a native call.
//!
//! [`Interp::resolve_library`]: crate::Interp::resolve_library

use rexx_core::ObjRef;

use crate::{Failure, Interp};

/// A routine body: the interpreter and the call's arguments.
pub(crate) type RoutineBody = fn(&mut Interp, &[Option<ObjRef>]) -> Result<ObjRef, Failure>;

/// A method body: the interpreter, the receiver, the method's scope and the
/// arguments.
pub(crate) type MethodBody =
    fn(&mut Interp, ObjRef, ObjRef, &[Option<ObjRef>]) -> Result<Option<ObjRef>, Failure>;

/// One library.
pub(crate) struct RustLibrary {
    /// The library name, matched byte for byte as a shared library's is.
    name: &'static str,
    routines: &'static [(&'static str, RoutineBody)],
    methods: &'static [(&'static str, MethodBody)],
}

impl RustLibrary {
    /// The library a load of this name answers.
    pub(crate) fn open(&self) -> rexx_api::load::Library {
        let routines: Vec<&str> = self.routines.iter().map(|(name, _)| *name).collect();
        let methods: Vec<&str> = self.methods.iter().map(|(name, _)| *name).collect();
        rexx_api::load::names_only(self.name, &routines, &methods)
    }
}

/// Every library this crate runs in Rust.
static LIBRARIES: &[RustLibrary] = &[
    RustLibrary {
        name: "rxmath",
        routines: &[
            ("MathLoadFuncs", crate::builtin::rxmath::load_funcs),
            ("MathDropFuncs", crate::builtin::rxmath::drop_funcs),
            ("RxCalcPi", crate::builtin::rxmath::pi),
            ("RxCalcSqrt", crate::builtin::rxmath::sqrt),
            ("RxCalcExp", crate::builtin::rxmath::exp),
            ("RxCalcLog", crate::builtin::rxmath::log),
            ("RxCalcLog10", crate::builtin::rxmath::log10),
            ("RxCalcSinH", crate::builtin::rxmath::sinh),
            ("RxCalcCosH", crate::builtin::rxmath::cosh),
            ("RxCalcTanH", crate::builtin::rxmath::tanh),
            ("RxCalcPower", crate::builtin::rxmath::power),
            ("RxCalcSin", crate::builtin::rxmath::sin),
            ("RxCalcCos", crate::builtin::rxmath::cos),
            ("RxCalcTan", crate::builtin::rxmath::tan),
            ("RxCalcCotan", crate::builtin::rxmath::cotan),
            ("RxCalcArcSin", crate::builtin::rxmath::arc_sin),
            ("RxCalcArcCos", crate::builtin::rxmath::arc_cos),
            ("RxCalcArcTan", crate::builtin::rxmath::arc_tan),
        ],
        methods: &[],
    },
    RustLibrary {
        name: "rxregexp",
        routines: &[],
        methods: &[
            ("RegExp_Init", crate::dispatch::regexp_methods::init),
            ("RegExp_Uninit", crate::dispatch::regexp_methods::uninit),
            ("RegExp_Parse", crate::dispatch::regexp_methods::parse),
            (
                "RegExp_Match",
                crate::dispatch::regexp_methods::match_string,
            ),
            ("RegExp_Pos", crate::dispatch::regexp_methods::pos),
        ],
    },
    RustLibrary {
        name: "rxsock",
        routines: &[
            ("SockDropFuncs", crate::builtin::rxsock::drop_funcs),
            (
                "SockGetHostByAddr",
                crate::builtin::rxsock::get_host_by_addr,
            ),
            (
                "SockGetHostByName",
                crate::builtin::rxsock::get_host_by_name,
            ),
            ("SockGetHostId", crate::builtin::rxsock::get_host_id),
            ("SockGetHostName", crate::builtin::rxsock::get_host_name),
            ("SockInit", crate::builtin::rxsock::init),
            ("SockSocket", crate::builtin::rxsock::socket_new),
            ("SockClose", crate::builtin::rxsock::close),
            ("SockSoClose", crate::builtin::rxsock::close),
            ("SockShutDown", crate::builtin::rxsock::shutdown),
            ("SockListen", crate::builtin::rxsock::listen),
            ("SockBind", crate::builtin::rxsock::bind),
            ("SockConnect", crate::builtin::rxsock::connect),
            ("SockAccept", crate::builtin::rxsock::accept),
            ("SockGetSockName", crate::builtin::rxsock::get_sock_name),
            ("SockGetPeerName", crate::builtin::rxsock::get_peer_name),
            ("SockSend", crate::builtin::rxsock::send),
            ("SockSendTo", crate::builtin::rxsock::send_to),
            ("SockRecv", crate::builtin::rxsock::recv),
            ("SockRecvFrom", crate::builtin::rxsock::recv_from),
            ("SockGetSockOpt", crate::builtin::rxsock::get_sock_opt),
            ("SockSetSockOpt", crate::builtin::rxsock::set_sock_opt),
            ("SockIoctl", crate::builtin::rxsock::ioctl),
            ("SockSelect", crate::builtin::rxsock::select),
            ("SockSock_Errno", crate::builtin::rxsock::sock_errno),
            ("SockPSock_Errno", crate::builtin::rxsock::psock_errno),
            ("SockVersion", crate::builtin::rxsock::version),
            ("SockLoadFuncs", crate::builtin::rxsock::load_funcs),
        ],
        methods: &[],
    },
    RustLibrary {
        name: "rxunixsys",
        // The oracle's table order. `SysCrypt` is not provided (Rulings).
        routines: &[
            ("SysSignal", crate::builtin::rxunixsys::signal),
            ("SysUname", crate::builtin::rxunixsys::uname),
            ("SysSetsid", crate::builtin::rxunixsys::setsid),
            ("SysGetsid", crate::builtin::rxunixsys::getsid),
            ("SysSetuid", crate::builtin::rxunixsys::setuid),
            ("SysGetuid", crate::builtin::rxunixsys::getuid),
            ("SysSeteuid", crate::builtin::rxunixsys::seteuid),
            ("SysGeteuid", crate::builtin::rxunixsys::geteuid),
            ("SysSetgid", crate::builtin::rxunixsys::setgid),
            ("SysGetgid", crate::builtin::rxunixsys::getgid),
            ("SysSetegid", crate::builtin::rxunixsys::setegid),
            ("SysGetegid", crate::builtin::rxunixsys::getegid),
            ("SysSetpgrp", crate::builtin::rxunixsys::setpgrp),
            ("SysGetpgrp", crate::builtin::rxunixsys::getpgrp),
            ("SysSetpgid", crate::builtin::rxunixsys::setpgid),
            ("SysGetpgid", crate::builtin::rxunixsys::getpgid),
            ("SysGetpid", crate::builtin::rxunixsys::getpid),
            ("SysGetppid", crate::builtin::rxunixsys::getppid),
            ("SysGettid", crate::builtin::rxunixsys::gettid),
            ("SysKill", crate::builtin::rxunixsys::kill),
            ("SysSymlink", crate::builtin::rxunixsys::symlink),
            ("SysLink", crate::builtin::rxunixsys::link),
            ("SysUnlink", crate::builtin::rxunixsys::unlink),
            ("SysChown", crate::builtin::rxunixsys::chown),
            ("SysLchown", crate::builtin::rxunixsys::lchown),
            ("SysChroot", crate::builtin::rxunixsys::chroot),
            ("SysUmask", crate::builtin::rxunixsys::umask),
            ("SysClose", crate::builtin::rxunixsys::close),
            ("SysGetpwnam", crate::builtin::rxunixsys::getpwnam),
            ("SysGetpwuid", crate::builtin::rxunixsys::getpwuid),
            ("SysGetgrnam", crate::builtin::rxunixsys::getgrnam),
            ("SysGetgrgid", crate::builtin::rxunixsys::getgrgid),
            ("SysStat", crate::builtin::rxunixsys::stat),
            ("SysAccess", crate::builtin::rxunixsys::access),
            ("SysEuidaccess", crate::builtin::rxunixsys::euidaccess),
            ("SysGetservbyname", crate::builtin::rxunixsys::getservbyname),
            ("SysGetservbyport", crate::builtin::rxunixsys::getservbyport),
            ("SysWordexp", crate::builtin::rxunixsys::wordexp),
            ("SysSetxattr", crate::builtin::rxunixsys::setxattr),
            ("SysGetxattr", crate::builtin::rxunixsys::getxattr),
            ("SysListxattr", crate::builtin::rxunixsys::listxattr),
            ("SysRemovexattr", crate::builtin::rxunixsys::removexattr),
            ("SysGetsizeofptr", crate::builtin::rxunixsys::getsizeofptr),
            ("SysGethostname", crate::builtin::rxunixsys::gethostname),
            ("SysChmod", crate::builtin::rxunixsys::chmod),
            ("SysGeterrno", crate::builtin::rxunixsys::geterrno),
            ("SysGeterrnomsg", crate::builtin::rxunixsys::geterrnomsg),
            ("SysMkdirUnix", crate::builtin::rxunixsys::mkdir),
            ("SysRmdirUnix", crate::builtin::rxunixsys::rmdir),
            ("SysGetdirlist", crate::builtin::rxunixsys::getdirlist),
            ("SysGettzname1", crate::builtin::rxunixsys::gettzname1),
            ("SysGettzname2", crate::builtin::rxunixsys::gettzname2),
        ],
        methods: &[],
    },
];

/// The library `name` names, or `None` for a name this crate leaves to the
/// shared-library search.
pub(crate) fn library(name: &[u8]) -> Option<&'static RustLibrary> {
    LIBRARIES
        .iter()
        .find(|library| library.name.as_bytes() == name)
}

/// The body of a row of `library`, a library [`RustLibrary::open`] answered,
/// named `row`; `None` for any other library or row.
fn of<B: Copy>(
    library: &rexx_api::load::Library,
    row: &[u8],
    table: fn(&RustLibrary) -> &'static [(&'static str, B)],
) -> Option<B> {
    let held = LIBRARIES
        .iter()
        .find(|candidate| library.name() == Some(candidate.name.as_bytes()))?;
    table(held)
        .iter()
        .find(|(name, _)| name.as_bytes() == row)
        .map(|(_, body)| *body)
}

/// The body of `library`'s routine row named `row`, where that row carries no
/// address and the library is one of these.
pub(crate) fn routine_body(library: &rexx_api::load::Library, row: &[u8]) -> Option<RoutineBody> {
    of(library, row, |held| held.routines)
}

/// The body of `library`'s method row named `row`, as [`routine_body`].
pub(crate) fn method_body(library: &rexx_api::load::Library, row: &[u8]) -> Option<MethodBody> {
    of(library, row, |held| held.methods)
}
