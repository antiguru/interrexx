/* SPDX-License-Identifier: MIT OR Apache-2.0 */
numeric digits 20
/* spec: rxsock.sockets.windows-loopback */
if SockInit() <> 0 then exit 1
if SockGetHostName() == '' then exit 2
if SockGetHostByName('localhost', 'host.') <> 1 | host.addr \== '127.0.0.1' then exit 3
listen = SockSocket('AF_INET', 'SOCK_STREAM', 0)
client = SockSocket('AF_INET', 'SOCK_STREAM', 'IPPROTO_TCP')
if listen < 0 | client < 0 then exit 4
bind.family = 'AF_INET'
bind.addr = '127.0.0.1'
bind.port = 0
if SockBind(listen, 'bind.') <> 0 then exit 5
if SockListen(listen, 1) <> 0 then exit 6
if SockGetSockName(listen, 'endpoint.') <> 0 | endpoint.port <= 0 then exit 7
if SockGetSockOpt(client, 'SOL_SOCKET', 'SO_TYPE', 'kind') <> 0 | kind \== 'STREAM' then exit 8
if SockSetSockOpt(listen, 'SOL_SOCKET', 'SO_RCVTIMEO', 2000) <> 0 then exit 9
connecting = .sender~new~start('connect', client, endpoint.)
server = SockAccept(listen, 'peer.')
if server < 0 | peer.addr \== '127.0.0.1' then exit 10
if connecting~result <> 0 then exit 11
if SockSetSockOpt(server, 'SOL_SOCKET', 'SO_RCVTIMEO', 2000) <> 0 then exit 12
read.0 = 1
read.1 = server
if SockSelect('read.', , , 0.02) <> 0 then exit 13
if SockIoctl(server, 'FIONBIO', 1) <> 0 then exit 14
if SockRecv(server, 'data', 5) <> -1 | errno \== 'EWOULDBLOCK' then exit 15
if SockIoctl(server, 'FIONBIO', 0) <> 0 then exit 16
sending = .sender~new~start('send', client)
if SockRecv(server, 'data', 5) <> 5 | data \== 'hello' then exit 17
if sending~result <> 5 then exit 18
if SockClose(client) <> 0 | SockClose(server) <> 0 | SockClose(listen) <> 0 then exit 19

receiver = SockSocket('AF_INET', 'SOCK_DGRAM', 'IPPROTO_UDP')
sender = SockSocket('AF_INET', 'SOCK_DGRAM', 0)
if receiver < 0 | sender < 0 then exit 20
if SockBind(receiver, 'bind.') <> 0 then exit 21
if SockGetSockName(receiver, 'endpoint.') <> 0 | endpoint.port <= 0 then exit 22
if SockSetSockOpt(receiver, 'SOL_SOCKET', 'SO_RCVTIMEO', 2000) <> 0 then exit 23
if SockSendTo(sender, 'packet', 'endpoint.') <> 6 then exit 24
read.0 = 1
read.1 = receiver
if SockSelect('read.', , , 1) <> 1 | read.0 <> 1 | read.1 <> receiver then exit 25
if SockRecvFrom(receiver, 'packet', 20, 'peer.') <> 6 | packet \== 'packet' then exit 26
if peer.addr \== '127.0.0.1' | peer.port <= 0 then exit 27
if SockClose(sender) <> 0 | SockClose(receiver) <> 0 then exit 28
exit 0

::requires 'rxsock' library

::class sender
::method connect
  use strict arg socket, endpoint
  call SysSleep 0.05
  return SockConnect(socket, endpoint)

::method send
  use strict arg socket
  call SysSleep 0.05
  return SockSend(socket, 'hello')
