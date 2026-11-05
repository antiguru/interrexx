# Extension libraries

`rxmath`, `rxregexp`, `rxunixsys` and `rxsock` are built into the interpreter as Rust internal
packages: naming them binds to Rust before any `dlopen`, and no `.so` is involved. The Rexx
packages that ship with them (`rxregexp.cls`, `socket.cls`, `streamsocket.cls`, `mime.cls`,
`smtp.cls`) are written for interrexx and found the way any `::REQUIRES` file is. `hostemu` and
`orxncurses` stay C extensions, built unmodified from `third_party/ooRexx` by
`scripts/build-extensions.sh` and loaded through the native API.

OS calls go through `rustix` or safe crates (`nix`, `dns-lookup`, `if-addrs`). The granted
`unsafe` is `rexx-exec/src/builtin/rxsock_sys.rs` (raw socket options on a borrowed descriptor) and
`set_disposition` and `send` in `rexx-exec/src/signal.rs` for `SysSignal` and `SysKill`.

## Shared conventions

* A package library loads whole, by `::requires 'name' LIBRARY`, `Package~loadLibrary` or
  `RxFuncAdd`. The name is case-sensitive. Once loaded, every routine is callable from every
  package and `RxFuncQuery` answers 0 for each. `RxFuncAdd`'s procedure argument is ignored, so
  aliases do not work; `RxFuncDrop` of a library routine answers 1 and the routine stays.
* Typed arguments convert in order before the routine runs; the first failure raises 88.901
  (missing), 88.922 (too many), 88.907 (32-bit integer), 88.905 (positive whole number) or 88.921
  (double).
* A routine that rejects its converted input raises 40.1, `External routine "NAME" failed.`, with
  the name as called, uppercased. This document calls that a routine failure.
* Where ooRexx crashes, interrexx raises a Rexx error instead (a Deviation): 93.900 for a
  RegularExpression with no compiled pattern or after `uninit`, 40.1 for an unknown group in
  `SysGetgrnam`/`SysGetgrgid`, a non-numeric `SockSelect` stem entry, or a `SockGetHostId` with no
  qualifying interface.
* Tracebacks through the rewritten `.cls` packages show their own line numbers (a Deviation).
* `SysGeterrno` and `SockSock_Errno` report the errno of their own library's last call on this
  thread, or 0; the interpreter's own I/O does not change it.

## rxmath

`RxCalcPi`, `RxCalcSqrt`, `RxCalcExp`, `RxCalcLog`, `RxCalcLog10`, `RxCalcSinH`, `RxCalcCosH`,
`RxCalcTanH`, `RxCalcPower`, the trigonometric `RxCalcSin`, `Cos`, `Tan`, `Cotan` and their arcs,
plus `MathLoadFuncs` (prints a banner only when given an argument) and `MathDropFuncs`.

* **Inputs** are doubles: a Rexx number, converted from all its digits whatever `NUMERIC DIGITS`
  is, or exactly `nan`, `+infinity`, `-infinity` (lower case). Overflow becomes infinity.
* **Precision** is a positive whole number capped at 16; omitted, the caller's `DIGITS`, capped.
* **Units** (trig only): first letter `D`, `R` or `G` (grades), default degrees; anything else is
  88.916, checked after the typed arguments.
* **Rendering.** `nan`, `+infinity`, `-infinity`; otherwise format with `%.*g` at precision + 2,
  then round that decimal text half-up to the precision and show it as Rexx would under that
  `DIGITS` in scientific form, whatever the caller's form. Rounding is on the text, twice:
  `RxCalcPower(10,9)` is `1.00000000E+9`, `RxCalcPower(1.05,1,2)` is `1.1`.
* **Values** are glibc `libm`'s for the named operation.
* **Degrees and grades** are reduced before the library call so multiples of a right angle are
  exact: `a = fmod(|angle|, 360)` picks the nearest multiple of 90 degrees and a reduced angle in
  [-45, 45] (with sine and cosine swapped or negated to match), and the radian conversion `r * pi / 180` is done in x87 extended precision before rounding to
  double. `RxCalcSin(30,16)` is `0.5` only because of that. Arc results convert back the same way.
  Cotangent is `1/tan`, giving signed infinities at multiples of 180.

## rxregexp

The library holds native methods used only by `rxregexp.cls`, whose public class is
`RegularExpression`: `new([pattern [, MINIMAL|MAXIMAL]])`, `parse`, `match`, `pos`, `position`,
`uninit`.

* **Patterns** are bytes, ending at the first NUL. Alternation `|` binds loosest; a term is a
  sequence of factors; a factor is an atom with at most one of `*`, `+` or `{n}` (n at least 2,
  read the way `atoi` reads it). Atoms: `?` any byte, `\b` the byte literally, `(...)` a group, a
  `[...]` set, or any other byte as itself. There are no anchors and no case folding.
* **Sets** may be negated with a leading `^`, and hold escaped bytes, `:NAME:` classes (ASCII C
  locale), ranges and literal bytes. A range `b0-b1` compares the bytes as signed values and adds
  those strictly above the lower end up to the upper, so `[c-a]` is `{c,b}`.
* **Parsing stops quietly** at the first byte that cannot continue a complete expression: `a)`
  compiles as `a`, and a second quantifier ends the pattern (`a**` is `a*`).
* **Compile errors** are codes 1 to 6 (unexpected symbol, missing `)`, bad set, unexpected end, bad
  count, bad class name). `new` raises 38 for a bad pattern and 93 for a bad match type; `parse`
  returns the code. After an error the object matches everything at position 0.
* **Repetition counts** whose automaton would pass 2,000,000 states are error 5 (a Deviation).
* **Meaning.** Alternation and repetition have their standard meaning, and every pattern compiles
  and matches in bounded time and memory. ooRexx differs here (a Deviation): it treats `X*R|Z` as
  `X*(R|Z)` and `X+Y*` as `XY*`, and loops forever on `(a*)*` or a set range ending at `'7F'x`.
* **match** in MAXIMAL mode needs the whole subject (a NUL in the subject also ends it); in MINIMAL
  mode any prefix, `position` being the shortest. On failure `position` is the longest prefix any
  partial match consumed.
* **pos** finds the leftmost start of the shortest match; MAXIMAL mode then extends from that start
  as far as a whole match reaches. So `b+` on `abbbc` is start 2 with end 4 (MAXIMAL) or 2
  (MINIMAL).
* `position` answers the string `!POS` until the first `parse`, `match` or `pos`.

## rxunixsys

Not loaded by default. Thin wrappers over POSIX calls: process and identity (`SysGetpid`,
`SysSetsid`, `SysKill`, `SysSignal`, `SysUmask`, the uid and gid getters and setters), files
(`SysSymlink`, `SysChown`, `SysChmod`, `SysStat`, `SysGetdirlist`, `SysAccess`, `SysMkdirUnix`, ...),
users and groups (`SysGetpwnam`, `SysGetgrgid`, ...), services, `SysUname`, extended attributes,
and `SysGeterrno`/`SysGeterrnomsg`.

* A routine returns the OS call's result as is (−1 on failure) and never raises because the call
  failed; the cause is read with `SysGeterrno`.
* Option arguments select by first letter, caselessly; an unknown letter or an empty argument is a
  routine failure. A missing user, file or service answers `""`.
* `SysChmod` takes exactly a 9-character `rwx` string with `s`/`S`/`t`/`T`; `SysStat`'s `P` option
  answers an `ls -l` style string; times are local `YYYY-MM-DD HH:MM:SS`.
* `SysGettid` answers the `pthread_self()` value, not the kernel thread id.
* `SysGeterrnomsg` uses a fixed table of messages for 0 to 132 and `Unknown` otherwise.
* `SysSignal` with SIGCHLD ignored makes a command's return code −10.
* `SysWordexp` is registered and refused: every call raises 40.1 (a Deviation).
* `SysCrypt` is absent, as on the ooRexx build it is compared with.

## rxsock

IPv4 only; a socket is its descriptor number.

* Routines set `errno` (a symbolic name for socket errors, otherwise decimal) and `h_errno` (0)
  in the caller's variable pool, so a method that exposes `errno` sees it.
* Stem arguments name a stem in the caller; a tail after the dot becomes a prefix on every element
  name (`'c.x'` uses `C.XFAMILY`). An unnameable stem makes the routine return 0 and do nothing.
* Address stems hold `FAMILY`, `PORT` (mod 65536) and `ADDR` (classic `inet_aton` text, so
  `127.1` works; `INADDR_ANY`); an unset element zeroes the whole address.
* Host lookups read `/etc/hosts`, then the resolver, whatever `nsswitch.conf` says; a resolver
  answer fills no aliases.
* Socket options map the `SO_*` names to their Linux numbers; timeouts are in milliseconds;
  `SO_LINGER` is `onoff seconds`.
* Blocking calls block only the calling activity. A blocking socket's accept and receives give up
  after `SO_RCVTIMEO` (−1, errno 11), connect after `SO_SNDTIMEO` (−1, errno 115). A halting signal
  ends a blocked send and raises HALT.
* `SockSelect` rewrites each participating stem to its ready sockets in order.
* `SockGetHostId` skips the last interface in the list, as ooRexx does.
* The interpreter's own descriptors sit at 1000 and above (`rexx_rxapi::reserved`); rxsock and
  rxunixsys answer errno 9 for them, as for a closed descriptor.

## Rexx packages

* **`socket.cls`**: `Socket`, `InetAddress`, `HostInfo` over rxsock. Argument checks raise 93.914
  with the allowed values listed in a fixed order. `accept` is unguarded, so `close` from another
  activity can end it. Failed lookups leave `HostInfo~address` as the literal `HOSTINFO.!ADDR`, as
  ooRexx does.
* **`streamsocket.cls`**: `StreamSocket`, an `InputOutputStream` over a socket; lines end at CRLF
  only; operations on a closed stream open it first. A failed send raises 97.1 for `ERRNO` and
  leaves the state unchanged, as ooRexx does.
* **`mime.cls`**: `mimepart` and `mimemultipart` render MIME headers and bodies byte for byte; a
  content line `.` is stored as `..`; the multipart boundary is derived from the date and time.
* **`smtp.cls`**: `smtpmsg` and `smtp`. `connect` sends `EHLO` with the local host name (which ooRexx
  looks up as `127.0.0.l`, and so does this package), logs in with `AUTH LOGIN` when offered, and
  `send` runs `MAIL FROM`, `RCPT TO`, `DATA` and the message, each step checking the reply's first
  digit.
