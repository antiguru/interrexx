//! `librexx.so.4`: the interpreter's C entry points and `librexxapi`'s beside them. This library
//! owns all of the API's state (Moritz, 2026-11-03, option A); `librexxapi.so.4` forwards to it.

mod exports;
mod rexxapi_exports;
