//! `librexx.so.4`: the interpreter's C entry points, with `librexxapi`'s compiled in beside them,
//! so a process loading both keeps one API state when this library's symbols bind first.

mod exports;

#[path = "../../librexxapi/src/exports.rs"]
mod rexxapi_exports;
