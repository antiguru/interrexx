**REFUSED by Moritz 2026-11-03** ("Can we refuse the function for the moment? It seems like a
function we need to be careful with"). Deviation 90. Owner: none yet; needs Moritz to schedule.

# SysWordexp: expansion without commands

`SysWordexp(text)` (rxunixsys) refuses every call since Phase 10 PR 4: it raises 40.1 and never
expands or starts a process. ooTest's `extensions/platform/unix/rxunixsys/SysUnix` `test_wordexp`
is held under Deviation 90.

Why: the first implementation handed `text` to `/bin/sh` after a pre-check whose quoting rules
differed from dash's. The PR 4 review found `"${x-'}"; touch PWNED; "'}"` (no `$(`, no backquote)
ran `touch PWNED`; the oracle's glibc `wordexp` returns one word and runs nothing.

The oracle calls `wordexp(3)` with no flags, so command substitution runs a shell. Moritz's
direction when this is picked up: no subshell, keep the safe expansion part, i.e. `wordexp` with
`WRDE_NOCMD` semantics: field splitting on `IFS`, quote removal, tilde, parameter expansion
(including the `${x-word}` family), arithmetic `$((...))`, pathname globbing, all in Rust with no
shell; command substitution raises 40.1; text `wordexp` rejects (`WRDE_BADCHAR`: unquoted newline,
`|`, `&`, `;`, `<`, `>`, `(`, `)`, `{`, `}`; bad syntax) raises 40.1 (Deviation 88). Implement from
POSIX and oracle measurements, not glibc's source.
