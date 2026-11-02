**CLOSED 2026-11-02 by Phase 10 PR 2.** Moritz granted one `unsafe` function (design doc, Decisions): `rexx-rxapi/src/descriptors.rs`, pinned in `unsafe_sites.rs`; the test is un-ignored.

Queued 2026-11-02 by Phase 10 PR 1.

# The started daemon keeps descriptors a client left inheritable

The behaviour description says the daemon starts with closed descriptors. Closing them needs `unsafe`
(`pre_exec`, or rustix `close`), or `nix` 0.29, and the workspace denies `unsafe_code`. Descriptors this
crate opens are `CLOEXEC`, so only one a caller deliberately left inheritable leaks, for example a
shell's `9>file`: a pipe held that way never sees EOF while the daemon lives.

`a_started_daemon_does_not_keep_inherited_descriptors` is ignored and red, ready to un-ignore.
Options: a one-site `unsafe` grant, `nix`, or dropping the requirement.
