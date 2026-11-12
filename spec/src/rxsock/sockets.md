---
element: "SockSocket"
kind: chapter
syntax: "SockSocket(domain, type, protocol); SockClose(socket); SockSoClose(socket); SockShutDown(socket, how); SockBind(socket, address); SockListen(socket, backlog); SockConnect(socket, address); SockAccept(socket [, address]); SockGetSockName(socket, address); SockGetPeerName(socket, address); SockSend(socket, data [, flags]); SockSendTo(socket, data [, flags], address); SockRecv(socket, variable, length [, flags]); SockRecvFrom(socket, variable, length [, flags], address); SockGetSockOpt(socket, level, option, variable); SockSetSockOpt(socket, level, option, value); SockIoctl(socket, command, value); SockSelect(read, write, except [, timeout]); SockGetHostByName(name, host); SockGetHostByAddr(address, host); SockGetHostName(); SockGetHostId()"
rexxref: "Rexx Socket Functions Library (rxsock) > Functions"
---

# Sockets

The functions on this page make a socket, give it an address, connect it,
send and receive, and ask and set its options. What every function of [the
rxsock library](index.md) shares holds here too: [the availability
rule](r:rxsock.index.available), [the argument rule](r:rxsock.index.args) and
[the error rule](r:rxsock.index.errors). A socket is named by its number, a
whole number that the system gives it and that depends on the run, so that an
example shows only that it is above `2`, which is the highest number of the
standard streams.

## Making and closing a socket

r[rxsock.sockets.socket]
`SockSocket(domain, type, protocol)` makes a socket and answers its number, or `-1`.
`domain` is `AF_INET`. `type` is `SOCK_STREAM` for a connection to one peer that
carries a stream of bytes, `SOCK_DGRAM` for single messages, or `SOCK_RAW`, which asks
for a privilege. `protocol` is `0` for the usual protocol of the type,
`IPPROTO_TCP` or `IPPROTO_UDP`. Each is read in either case. A name that is none
of these raises `SYNTAX 40.1`. A socket is made open, in blocking mode, and
bound to no address, so that [its address](r:rxsock.sockets.address) is `0.0.0.0` with
the port `0`.

```rexx
a = SockSocket('AF_INET', 'SOCK_STREAM', 0)
b = SockSocket('af_inet', 'sock_dgram', 'ipproto_udp')
c = SockSocket('AF_INET', 'SOCK_STREAM', 'IPPROTO_TCP')
d = SockSocket('AF_INET', 'SOCK_DGRAM', 0)
say (a > 2) (b > 2) (c > 2) (d > 2) (a <> b) (b <> c) (c <> d) (a <> d)
say SockGetSockName(a, 'n.') n.family n.addr n.port
say SockClose(a) SockClose(b) SockClose(c) SockClose(d)
call try "SockSocket('AF_UNIX', 'SOCK_STREAM', 0)"
call try "SockSocket('AF_INET6', 'SOCK_STREAM', 0)"
call try "SockSocket('AF_INET', 'SOCK_X', 0)"
call try "SockSocket('AF_INET', 'SOCK_STREAM', 'x')"
call try "SockSocket('AF_INET', 'SOCK_STREAM', 6)"
call try "SockSocket('', 'SOCK_STREAM', 0)"
exit
try:
  signal on syntax name e
  interpret 'say arg(1) ''=>'' ' arg(1)
  return
e: say arg(1) '->' condition('o')~code
  return
::requires 'rxsock' library
```

```output
1 1 1 1 1 1 1 1
0 2 0.0.0.0 0
0 0 0 0
SockSocket('AF_UNIX', 'SOCK_STREAM', 0) -> 40.1
SockSocket('AF_INET6', 'SOCK_STREAM', 0) -> 40.1
SockSocket('AF_INET', 'SOCK_X', 0) -> 40.1
SockSocket('AF_INET', 'SOCK_STREAM', 'x') -> 40.1
SockSocket('AF_INET', 'SOCK_STREAM', 6) -> 40.1
SockSocket('', 'SOCK_STREAM', 0) -> 40.1
```

r[rxsock.sockets.close]
`SockClose(socket)` and `SockSoClose(socket)` close the socket and answer `0`, or
`-1` with the error `9` for a number that is not an open descriptor, one that
was already closed included.

```rexx
a = SockSocket('AF_INET', 'SOCK_STREAM', 0)
b = SockSocket('AF_INET', 'SOCK_DGRAM', 0)
say SockClose(a) SockSoClose(b)
say SockClose(a) errno SockSoClose(b) errno
say SockClose(999) errno SockClose(-1) errno SockClose(1000000) errno
::requires 'rxsock' library
```

```output
0 0
-1 9 -1 9
-1 9 -1 9 -1 9
```

## Addresses

r[rxsock.sockets.address]
An address is given as a stem, named as a string that ends with a point: its field
`family` is `AF_INET` or its number `2`, its field `port` the port number, and its
field `addr` an Internet address in dotted form, where `0.0.0.0` and an `addr` that
is not set are the address of every interface of the host, and `127.0.0.1` is the
loopback interface. A `port` of `0` asks the system for a free port when the
socket is bound. A function that answers an address sets the three fields of the
stem it is given: `family` is the number `2`, `port` the port number and `addr`
the address in dotted form. `SockGetSockName(socket, address)` answers the address of the socket
itself with `0`, which is `0.0.0.0` and the port `0` for a socket that is not bound,
and `SockGetPeerName(socket, address)` that of the socket at the other end of a
connection, or `-1` with the error `ENOTCONN` where there is none.

```rexx
s = SockSocket('AF_INET', 'SOCK_STREAM', 0)
say SockGetSockName(s, 'n.') n.family n.addr n.port
a.family = 'AF_INET'; a.port = 0; a.addr = '127.0.0.1'
say SockBind(s, 'a.') errno
say SockGetSockName(s, 'n.') n.family n.addr (n.port > 0) (n.port < 65536)
say SockGetPeerName(s, 'p.') errno
say SockClose(s)
t = SockSocket('AF_INET', 'SOCK_STREAM', 0)
b.family = 2; b.port = 0
say SockBind(t, 'b.') SockGetSockName(t, 'n.') n.family n.addr (n.port > 0)
say SockClose(t)
u = SockSocket('AF_INET', 'SOCK_STREAM', 0)
drop a.
say SockBind(u, 'a.') SockGetSockName(u, 'n.') n.addr (n.port > 0)
a.family = 'AF_UNIX'; a.port = 0; a.addr = '127.0.0.1'
v = SockSocket('AF_INET', 'SOCK_STREAM', 0)
say SockBind(v, 'a.') errno
say SockClose(u) SockClose(v)
call try "SockBind(1)"
call try "SockGetSockName(1)"
exit
try:
  signal on syntax name e
  interpret 'say arg(1) ''=>'' ' arg(1)
  return
e: say arg(1) '->' condition('o')~code
  return
::requires 'rxsock' library
```

```output
0 2 0.0.0.0 0
0 0
0 2 127.0.0.1 1 1
-1 ENOTCONN
0
0 0 2 0.0.0.0 1
0
0 0 0.0.0.0 1
-1 EAFNOSUPPORT
0 0
SockBind(1) -> 88.901
SockGetSockName(1) -> 88.901
```

## A connection

r[rxsock.sockets.connection]
A server makes a stream socket, binds it with `SockBind(socket, address)`, and
calls `SockListen(socket, backlog)`, which lets the socket queue that many
connections that are not yet accepted. A client makes a stream socket and calls
`SockConnect(socket, address)` with the address of the server. All three answer `0`
or `-1`. `SockAccept(socket [, address])` takes the first queued connection
and answers the number of a new socket that is the server's end of it, with the
address of the client in the stem `address` where one is given; it waits for a
connection where none is queued, unless the socket is in non-blocking mode,
where it answers `-1` with the error `EWOULDBLOCK`. A connection to a listening socket of the same program is complete as soon
as `SockConnect` answers, before `SockAccept` is called, as long as the queue is not
full.
A stream socket connects once: a second `SockConnect` answers `-1` with the error
`EISCONN`. A connection to a port where no socket listens answers `-1` with the error
`ECONNREFUSED`, and a second bind of an address that another socket holds `-1`
with `EADDRINUSE`; a socket that is bound already answers `-1` with the error
`22` when it is bound again.

```rexx
srv = SockSocket('AF_INET', 'SOCK_STREAM', 0)
a.family = 'AF_INET'; a.port = 0; a.addr = '127.0.0.1'
say SockBind(srv, 'a.') SockListen(srv, 5)
say SockBind(srv, 'a.') errno
call SockGetSockName srv, 'name.'
say SockIoctl(srv, 'FIONBIO', 1) SockAccept(srv, 'p.') errno
say SockIoctl(srv, 'FIONBIO', 0)
cli = SockSocket('AF_INET', 'SOCK_STREAM', 0)
say SockConnect(cli, 'name.') errno
say SockConnect(cli, 'name.') errno
conn = SockAccept(srv, 'peer.')
say (conn > 2) (conn <> srv) (conn <> cli) peer.family peer.addr (peer.port > 0)
say SockGetPeerName(cli, 'q.') q.addr (q.port == name.port)
say SockGetSockName(conn, 'r.') r.addr (r.port == name.port)
say SockGetPeerName(conn, 's.') s.addr (s.port == peer.port)
say SockGetPeerName(srv, 't.') errno
other = SockSocket('AF_INET', 'SOCK_STREAM', 0)
say SockBind(other, 'name.') errno
say SockClose(other)
say SockClose(conn) SockClose(cli) SockClose(srv)
late = SockSocket('AF_INET', 'SOCK_STREAM', 0)
say SockConnect(late, 'name.') errno
say SockClose(late)
::requires 'rxsock' library
```

```output
0 0
-1 22
0 -1 EWOULDBLOCK
0
0 0
-1 EISCONN
1 1 1 2 127.0.0.1 1
0 127.0.0.1 1
0 127.0.0.1 1
0 127.0.0.1 1
-1 ENOTCONN
-1 EADDRINUSE
0
0 0 0
-1 ECONNREFUSED
0
```

## Sending and receiving

r[rxsock.sockets.stream]
`SockSend(socket, data [, flags])` sends the bytes of `data` on a connected
socket and answers the number of bytes it sent, or `-1`. `SockRecv(socket,
variable, length [, flags])` receives at most `length` bytes, sets the variable
that `variable` names, a string as a name of a variable, to them, and answers the
number received, or `-1`; the variable is set to the null string where nothing
was received. It waits for data unless the socket is in non-blocking mode, and
answers `0` with the null string where the other end has finished sending, which
`SockShutDown(socket, how)` does with a `how` of `1`, for sending, `0` for
receiving or `2` for both, and answers `0`, or `-1` with the error `22` for
another `how`. Stream sockets keep the order of the bytes and not the bounds of
the calls, so that bytes that were sent by one call can be received by several.
`SockRecv` on a stream socket that is not connected answers `-1` with the error
`ENOTCONN`, and `SockSend` on one answers `-1` with the error `32`. The flags are the words `MSG_OOB`,
`MSG_PEEK` and `MSG_DONTROUTE`, in either case, with blanks between them.

```rexx
srv = SockSocket('AF_INET', 'SOCK_STREAM', 0)
a.family = 'AF_INET'; a.port = 0; a.addr = '127.0.0.1'
call SockBind srv, 'a.'
call SockListen srv, 5
call SockGetSockName srv, 'name.'
cli = SockSocket('AF_INET', 'SOCK_STREAM', 0)
call SockConnect cli, 'name.'
conn = SockAccept(srv)
say SockSend(cli, 'hello') SockRecv(conn, 'data', 100) data
say SockSend(conn, 'back!') SockRecv(cli, 'part', 3) part SockRecv(cli, 'rest', 100) rest
say SockSend(cli, 'abcdef') SockRecv(conn, 'peek', 4, 'MSG_PEEK') peek SockRecv(conn, 'got', 100) got
say SockSend(cli, '') SockSend(cli, 'x', 'MSG_DONTROUTE')
say SockRecv(conn, 'one', 1) one
say SockShutDown(cli, 1) SockRecv(conn, 'eof', 100) '['eof']'
say SockSend(conn, 'tail') SockRecv(cli, 'tail', 100) tail
say SockShutDown(cli, 5) errno SockShutDown(conn, 2)
say SockClose(conn) SockClose(cli) SockClose(srv)
fresh = SockSocket('AF_INET', 'SOCK_STREAM', 0)
say SockRecv(fresh, 'v', 10) errno '['v']'
say SockSend(fresh, 'x') errno
say SockClose(fresh)
::requires 'rxsock' library
```

```output
5 5 hello
5 3 bac 2 k!
6 4 abcd 6 abcdef
0 1
1 x
0 0 []
4 4 tail
-1 22 0
0 0 0
-1 ENOTCONN []
-1 32
0
```

r[rxsock.sockets.datagram]
Datagram sockets send messages that keep their bounds. `SockSendTo(socket, data
[, flags], address)` sends `data` to the address in the stem `address` from a
socket that need not be connected, and answers the number of bytes, or `-1`.
`SockRecvFrom(socket, variable, length [, flags], address)` receives one message
of at most `length` bytes, sets the variable to it, sets the stem `address` to the
address it came from, and answers the number of bytes. A message longer than
`length` is cut to it.

```rexx
a.family = 'AF_INET'; a.port = 0; a.addr = '127.0.0.1'
u1 = SockSocket('AF_INET', 'SOCK_DGRAM', 0)
u2 = SockSocket('AF_INET', 'SOCK_DGRAM', 'IPPROTO_UDP')
say SockBind(u1, 'a.') SockBind(u2, 'a.')
call SockGetSockName u1, 'n1.'
call SockGetSockName u2, 'n2.'
say SockSendTo(u2, 'datagram', 0, 'n1.')
say SockRecvFrom(u1, 'd', 100, 0, 'f.') d f.family f.addr (f.port == n2.port)
say SockSendTo(u2, 'x', 'n1.') SockSendTo(u2, 'again', 'n1.')
say SockRecvFrom(u1, 'd', 100, 'f.') d SockRecvFrom(u1, 'd', 100, 'f.') d
say SockSendTo(u2, 'truncated', 'n1.') SockRecvFrom(u1, 'd', 5, 'f.') d
say SockClose(u1) SockClose(u2)
::requires 'rxsock' library
```

```output
0 0
8
8 datagram 2 127.0.0.1 1
1 5
1 x 5 again
9 5 trunc
0 0
```

## Options and modes

r[rxsock.sockets.options]
`SockGetSockOpt(socket, level, option, variable)` sets the variable that `variable`
names to the value of a socket option and answers `0`, or `-1`;
`SockSetSockOpt(socket, level, option, value)` sets it and answers `0`, or `-1`.
`level` is `SOL_SOCKET`, and any other level raises `SYNTAX 40.1`. The options are
`SO_TYPE`, which is `STREAM`, `DGRAM` or `RAW` and cannot be set, `SO_ERROR`, a
number, that cannot be set, `SO_REUSEADDR`, `SO_KEEPALIVE`, `SO_DONTROUTE`,
`SO_BROADCAST` and `SO_OOBINLINE`, which are `0` or `1`, `SO_SNDBUF` and
`SO_RCVBUF`, sizes in bytes, `SO_LINGER`, two numbers `on` and `seconds` with a
blank between them, `SO_RCVTIMEO` and `SO_SNDTIMEO`, a time in milliseconds, and
`SO_DEBUG`, `SO_RCVLOWAT`, `SO_SNDLOWAT` and `SO_USELOOPBACK`, numbers. An
option that is not one of them answers `-1` with the error `ENOPROTOOPT`, and the
variable is `0` after a failed `SockGetSockOpt`. A value that is set for a
`SO_TYPE` or `SO_ERROR` answers `-1` without an error.

```rexx
s = SockSocket('AF_INET', 'SOCK_STREAM', 0)
d = SockSocket('AF_INET', 'SOCK_DGRAM', 0)
say SockGetSockOpt(s, 'SOL_SOCKET', 'SO_TYPE', 'v') v SockGetSockOpt(d, 'sol_socket', 'so_type', 'v') v
say SockGetSockOpt(s, 'SOL_SOCKET', 'SO_ERROR', 'v') v
say SockGetSockOpt(s, 'SOL_SOCKET', 'SO_REUSEADDR', 'v') v SockSetSockOpt(s, 'SOL_SOCKET', 'SO_REUSEADDR', 1) SockGetSockOpt(s, 'SOL_SOCKET', 'SO_REUSEADDR', 'v') v
say SockSetSockOpt(s, 'SOL_SOCKET', 'SO_KEEPALIVE', 1) SockGetSockOpt(s, 'SOL_SOCKET', 'SO_KEEPALIVE', 'v') v
say SockSetSockOpt(s, 'SOL_SOCKET', 'SO_KEEPALIVE', 0) SockGetSockOpt(s, 'SOL_SOCKET', 'SO_KEEPALIVE', 'v') v
say SockSetSockOpt(s, 'SOL_SOCKET', 'SO_LINGER', '1 5') SockGetSockOpt(s, 'SOL_SOCKET', 'SO_LINGER', 'v') v
say SockSetSockOpt(s, 'SOL_SOCKET', 'SO_RCVTIMEO', 1500) SockGetSockOpt(s, 'SOL_SOCKET', 'SO_RCVTIMEO', 'v') v
say SockSetSockOpt(d, 'SOL_SOCKET', 'SO_BROADCAST', 1) SockGetSockOpt(d, 'SOL_SOCKET', 'SO_BROADCAST', 'v') v
say SockSetSockOpt(s, 'SOL_SOCKET', 'SO_TYPE', 1) SockSetSockOpt(s, 'SOL_SOCKET', 'SO_ERROR', 1)
say SockSetSockOpt(s, 'SOL_SOCKET', 'SO_NOSUCH', 1) errno
say SockGetSockOpt(s, 'SOL_SOCKET', 'SO_NOSUCH', 'v') errno '['v']'
say SockGetSockOpt(999, 'SOL_SOCKET', 'SO_TYPE', 'v') errno
say SockSetSockOpt(999, 'SOL_SOCKET', 'SO_REUSEADDR', 1) errno
call try "SockGetSockOpt(s, 'SOL_TCP', 'SO_TYPE', 'v')"
call try "SockSetSockOpt(s, 'IPPROTO_TCP', 'SO_REUSEADDR', 1)"
call try "SockGetSockOpt(s, '', 'SO_TYPE', 'v')"
say SockClose(s) SockClose(d)
exit
try:
  signal on syntax name e
  interpret 'say arg(1) ''=>'' ' arg(1)
  return
e: say arg(1) '->' condition('o')~code
  return
::requires 'rxsock' library
```

```output
0 STREAM 0 DGRAM
0 0
0 0 0 0 1
0 0 1
0 0 0
0 0 1 5
0 0 1500
0 0 1
-1 -1
-1 ENOPROTOOPT
-1 ENOPROTOOPT [0]
-1 9
-1 9
SockGetSockOpt(s, 'SOL_TCP', 'SO_TYPE', 'v') -> 40.1
SockSetSockOpt(s, 'IPPROTO_TCP', 'SO_REUSEADDR', 1) -> 40.1
SockGetSockOpt(s, '', 'SO_TYPE', 'v') -> 40.1
0 0
```

r[rxsock.sockets.ioctl]
`SockIoctl(socket, command, value)` controls a socket and answers `0`, or `-1`. The
command `FIONBIO` puts the socket in non-blocking mode for a `value` that is not `0`
and in blocking mode for `0`. The command `FIONREAD` sets the variable that `value`
names to the number of bytes that can be received at once. Any other command
answers `-1`, and leaves `errno` as it was. In non-blocking mode a call that would
wait answers `-1` with the error `EWOULDBLOCK`.

```rexx
srv = SockSocket('AF_INET', 'SOCK_STREAM', 0)
a.family = 'AF_INET'; a.port = 0; a.addr = '127.0.0.1'
call SockBind srv, 'a.'
call SockListen srv, 5
call SockGetSockName srv, 'name.'
cli = SockSocket('AF_INET', 'SOCK_STREAM', 0)
call SockConnect cli, 'name.'
conn = SockAccept(srv)
say SockIoctl(cli, 'FIONREAD', 'n') n
say SockSend(conn, 'abc') SockIoctl(cli, 'fionread', 'n') n
say SockRecv(cli, 'd', 2) d SockIoctl(cli, 'FIONREAD', 'n') n
say SockIoctl(cli, 'FIONBIO', 1) SockRecv(cli, 'd', 100) d SockRecv(cli, 'd', 100) '['d']' errno
say SockIoctl(cli, 'FIONBIO', 0)
say SockIoctl(cli, 'FIOASYNC', 1)
say SockIoctl(999, 'FIONBIO', 1) errno
say SockClose(conn) SockClose(cli) SockClose(srv)
call try "SockIoctl(cli, 'FIONBIO', 'x')"
call try "SockIoctl(cli, 'FIONBIO')"
exit
try:
  signal on syntax name e
  interpret 'say arg(1) ''=>'' ' arg(1)
  return
e: say arg(1) '->' condition('o')~code
  return
::requires 'rxsock' library
```

```output
0 0
3 0 3
2 ab 0 1
0 1 c -1 [] EWOULDBLOCK
0
-1
-1 9
0 0 0
SockIoctl(cli, 'FIONBIO', 'x') -> 40.1
SockIoctl(cli, 'FIONBIO') -> 88.901
```

## Waiting for sockets

r[rxsock.sockets.select]
`SockSelect(read, write, except [, timeout])` asks which sockets are ready. Each
of the first three arguments is the name of a stem as a string that ends with a
point, whose tail `0` holds the number of sockets and whose tails `1` to that
number hold their numbers: the sockets to look at for reading, for writing and for
urgent data. An argument that is the null string, or that names no such stem,
looks at no sockets. `timeout` is a number of seconds, which may have a
fraction, and `0` asks without waiting; without it the call waits until a socket
is ready. The call answers the number of sockets that are ready. Where that is not `0`, it
sets each stem to the sockets of it that are ready, its tail `0` first, which is `0` for a
stem none of whose sockets is ready; a call that answers `0` leaves the stems as they were. A socket is
ready for reading where a read would not wait, which is also true of a listening
socket with a connection to accept and of a socket whose peer has closed it, and
ready for writing where a write would not wait. A call that finds no socket ready
before the time is up answers `0`. The sockets of the stems are those of this program;
a stem entry that is not a whole number raises `SYNTAX 40.1`.

```rexx
srv = SockSocket('AF_INET', 'SOCK_STREAM', 0)
a.family = 'AF_INET'; a.port = 0; a.addr = '127.0.0.1'
call SockBind srv, 'a.'
call SockListen srv, 5
call SockGetSockName srv, 'name.'
r.0 = 1; r.1 = srv
say SockSelect('r.', '', '', 0) r.0
cli = SockSocket('AF_INET', 'SOCK_STREAM', 0)
call SockConnect cli, 'name.'
r.0 = 1; r.1 = srv
say SockSelect('r.', '', '', 0.2) r.0 (r.1 == srv)
conn = SockAccept(srv)
r.0 = 3; r.1 = srv; r.2 = cli; r.3 = conn
w.0 = 2; w.1 = cli; w.2 = conn
say SockSelect('r.', 'w.', '', 0) r.0 w.0 (w.1 == cli) (w.2 == conn)
say SockSend(conn, 'abc')
r.0 = 3; r.1 = srv; r.2 = cli; r.3 = conn
say SockSelect('r.', '', '', 1) r.0 (r.1 == cli)
say SockSelect('', '', '', 0) SockSelect('nostem.', 'none', '', 0)
say SockClose(conn) SockClose(cli) SockClose(srv)
::requires 'rxsock' library
```

```output
0 1
1 1 1
2 0 2 1 1
3
1 1 1
0 0
0 0 0
```

## Hosts

r[rxsock.sockets.host]
`SockGetHostByName(name, host)` looks up the host `name` and answers `1` where it
found it and `0` where it did not. `SockGetHostByAddr(address, host [, family])`
looks it up by its address, with the family as the number `2` where it is given.
They set the stem `host`: its field `name` to the official name of the host,
`addr` to its address in dotted form, `addrtype` to `AF_INET`, and `alias.0` to the number of
its other names, followed by the names in `alias.1` and on. A host that is given by
an address in dotted form is its own name. `SockGetHostName()` answers the name
of this host and `SockGetHostId()` its address in dotted form.

```rexx
say SockGetHostByName('127.0.0.1', 'h.') h.name h.addr h.addrtype h.alias.0
say SockGetHostByAddr('127.0.0.1', 'k.') k.addr k.addrtype
say SockGetHostByAddr('127.0.0.1', 'k.', 2)
say SockGetHostByAddr('x', 'k.')
say (length(SockGetHostName()) > 0)
parse value SockGetHostId() with p1 '.' p2 '.' p3 '.' p4
say datatype(p1, 'W') datatype(p2, 'W') datatype(p3, 'W') datatype(p4, 'W')
::requires 'rxsock' library
```

```output
1 127.0.0.1 127.0.0.1 AF_INET 0
1 127.0.0.1 AF_INET
1
0
1
1 1 1 1
```
