//! `librexxapi.so.4`: the classic API's queues, registrations, macrospace and memory, as the
//! public declarations in `api/rexx.h` name them, forwarded to `librexx.so.4`, which owns their
//! state.

// On 64-bit Windows, REXXENTRY (APIENTRY) uses the C ABI. On 32-bit
// Windows the classic exports would need stdcall signatures.
#[cfg(all(windows, target_arch = "x86"))]
compile_error!("the classic API forwarding DLL requires 64-bit Windows");

mod exports;
