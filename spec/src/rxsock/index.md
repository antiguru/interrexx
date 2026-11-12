---
element: "rxsock"
kind: chapter
rexxref: "Rexx Socket Functions Library (rxsock) > Functions"
---

# The rxsock library

The library `rxsock` lets a program use the sockets of the system: it makes a
socket, binds it to an address on the host, connects it, sends and receives, and
asks and sets the options of a socket. This page holds what every function of
the library shares: how the functions are made available, how their arguments are
read, and how a call that fails says why. [The sockets page](sockets.md) holds the
functions themselves. Every example on these pages works on the loopback
interface of the host, with both ends of a connection in the one program, so that
it needs no network and no second program, and asks the system for a port
instead of naming one.

## Availability

r[rxsock.index.available]
The functions of the library are not provided from the start of a program. They
are made available by [RXFUNCADD](r:bif.rxfuncadd.library) of the name of any
one of them with the library name `rxsock`, or by a
[`::requires`](r:dir.requires.library) directive with the `LIBRARY` keyword. From
then on every function of the library can be called, as `name(arguments)` or by
CALL, with its name in any case. A name is not a method of `String`: sending it
to a string raises `SYNTAX 97.1`, and a call before the library is available raises
`SYNTAX 43.1`. RXFUNCDROP removes nothing. The functions are `SockLoadFuncs`,
`SockDropFuncs`, `SockInit`, `SockVersion`, `SockSock_Errno`, `SockPSock_Errno`,
`SockSocket`, `SockClose`, `SockSoClose`, `SockShutDown`, `SockBind`,
`SockListen`, `SockConnect`, `SockAccept`, `SockGetSockName`,
`SockGetPeerName`, `SockSend`, `SockSendTo`, `SockRecv`, `SockRecvFrom`,
`SockGetSockOpt`, `SockSetSockOpt`, `SockIoctl`, `SockSelect`,
`SockGetHostByName`, `SockGetHostByAddr`, `SockGetHostName` and `SockGetHostId`.

```rexx
say rxfuncquery('SockInit') rxfuncquery('SockSocket')
say rxfuncadd('SockInit', 'rxsock')
say rxfuncquery('SockInit') rxfuncquery('SockSocket') rxfuncquery('SockSelect')
say SockInit() SOCKINIT() 'sockinit'()
say rxfuncdrop('SockInit') rxfuncquery('SockInit') SockInit()
signal on syntax name a
say 'x'~SockInit
a: say condition('o')~code
```

```output
1 1
0
0 0 0
0 0 0
1 0 0
97.1
```

```rexx
say SockInit()
::requires 'rxsock' library
```

```output
0
```

```rexx
signal on syntax name a
say SockInit()
a: say condition('o')~code
```

```output
43.1
```

r[rxsock.index.load]
`SockLoadFuncs` and `SockDropFuncs` exist for programs that were written for a
system where the library had to be loaded and dropped: each answers the null
string and changes nothing, and `SockLoadFuncs` may be given one argument, of any
value, which it ignores. `SockInit()` answers `0`. `SockVersion()` answers the
version of the library as three whole numbers with a point between each. More
arguments than these raise `SYNTAX 88.922`.

```rexx
say '['SockLoadFuncs()']' '['SockLoadFuncs('x')']' '['SockDropFuncs()']' SockInit()
parse value SockVersion() with major '.' minor '.' patch
say datatype(major, 'W') datatype(minor, 'W') datatype(patch, 'W') (words(translate(SockVersion(), ' ', '.')) == 3)
call try "SockLoadFuncs(1, 2)"
call try "SockDropFuncs(1)"
call try "SockInit(1)"
call try "SockVersion(1)"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
::requires 'rxsock' library
```

```output
[] [] [] 0
1 1 1 1
88.922
88.922
88.922
88.922
```

## Arguments

r[rxsock.index.args]
The functions follow [the native call rule](r:bif.index.call.native), as the
[RexxUtil](../rexxutil/index.md) functions do. A required argument that is not
given, or is omitted, raises `SYNTAX 88.901`, and an argument past the last one
of the function raises `SYNTAX 88.922`; the count is checked before the arguments
are read. A socket, a flag and a count are given as [whole
numbers](r:rxunixsys.index.whole), and one that is not raises `SYNTAX 88.907`.
A name of a domain, a type, a protocol, a level or an option is read in either
case. One that the function does not know raises `SYNTAX 40.1` where the page of
the function says so, and is otherwise a failed call that [the error
rule](r:rxsock.index.errors) describes.

```rexx
call try "SockSocket('AF_INET', 'SOCK_STREAM')"
call try "SockSocket('AF_INET', 'SOCK_STREAM', 0, 1)"
call try "SockClose()"
call try "SockClose(1, 2)"
call try "SockClose('x')"
call try "SockClose(1.5)"
call try "SockBind(1)"
call try "SockListen(1)"
call try "SockSend(1)"
call try "SockSend(1, 'x', 0, 1)"
call try "SockRecv(1, 'v')"
call try "SockGetSockOpt(1, 'SOL_SOCKET', 'SO_TYPE')"
call try "SockSetSockOpt(1, 'SOL_SOCKET', 'SO_TYPE')"
call try "SockIoctl(1, 'FIONBIO')"
call try "SockGetSockName(1)"
call try "SockGetHostName(1)"
call try "SockGetHostId(1)"
call try "SockPSock_Errno('x', 'y')"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
::requires 'rxsock' library
```

```output
88.901
88.922
88.901
88.922
88.907
88.907
88.901
88.901
88.901
88.922
88.901
88.901
88.901
88.901
88.901
88.922
88.922
88.922
```

## Errors

r[rxsock.index.errors]
A function that makes or uses a socket answers `-1` where the system refused
it, and the functions that look up a host answer `0`. The call does not raise a
condition. It sets two variables of the program that made the call, as `PARSE`
sets one: `errno` to the name of the error, and `h_errno` to `0`. The name is
`EWOULDBLOCK`, `EINPROGRESS`, `EALREADY`, `ENOTSOCK`, `EDESTADDRREQ`, `EMSGSIZE`,
`EPROTOTYPE`, `ENOPROTOOPT`, `EPROTONOSUPPORT`, `ESOCKTNOSUPPORT`, `EOPNOTSUPP`,
`EPFNOSUPPORT`, `EAFNOSUPPORT`, `EADDRINUSE`, `EADDRNOTAVAIL`, `ENETDOWN`,
`ENETUNREACH`, `ENETRESET`, `ECONNABORTED`, `ECONNRESET`, `ENOBUFS`, `EISCONN`,
`ENOTCONN`, `ESHUTDOWN`, `ETOOMANYREFS`, `ETIMEDOUT`, `ECONNREFUSED`, `ELOOP`,
`ENAMETOOLONG`, `EHOSTDOWN`, `EHOSTUNREACH` or `ENOTEMPTY` for the errors of
those names, and for any other error the number of the error as a Linux system
gives it, as `9` for a socket that is not open and `22` for an argument the
system found invalid. A call that succeeds sets `errno` to `0`. `SockSock_Errno()`
answers the number of the error of the last call that failed, which the
variable shows as a name; what it answers after a call that succeeded, or before
any failed, is not specified.

```rexx
say SockClose(999) errno h_errno SockSock_Errno()
say SockClose(-1) errno
say SockListen(999, 5) errno
say SockGetPeerName(999, 'p.') errno
say SockInit() errno h_errno
s = SockSocket('AF_INET', 'SOCK_STREAM', 0)
say (s > 2) errno
say SockGetPeerName(s, 'p.') errno SockSock_Errno()
say SockClose(s) errno
say SockClose(s) errno SockSock_Errno()
::requires 'rxsock' library
```

```output
-1 9 0 9
-1 9
-1 9
-1 9
0 0 0
1 0
-1 ENOTCONN 107
0 0
-1 9 9
```

r[rxsock.index.psock]
`SockPSock_Errno([text])` writes the line `RxSOCK Error: ` followed by `text`,
the null string where it is omitted, on the standard error stream, and answers the null
string. It does not look at the error of the last call.

```rexx
call lineout 'c1zqs1.rex', "say '['SockPSock_Errno('first')']'"
call lineout 'c1zqs1.rex', "say '['SockPSock_Errno()']'"
call lineout 'c1zqs1.rex', "::requires 'rxsock' library"
call lineout 'c1zqs1.rex'
address system 'rexx c1zqs1.rex' with output stem out. error stem err.
say out.0 err.0
do i = 1 to out.0
  say out.i
end
do i = 1 to err.0
  say '['err.i']'
end
call sysfiledelete 'c1zqs1.rex'
```

```output
2 2
[]
[]
[RxSOCK Error: first]
[RxSOCK Error: ]
```
