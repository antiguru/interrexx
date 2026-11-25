/* SPDX-License-Identifier: MIT OR Apache-2.0 */
numeric digits 20
say 'init' SockInit() SockSock_Errno()
say 'host' SockGetHostByName('localhost', 'host.') host.addr
listen = SockSocket('AF_INET', 'SOCK_STREAM', 0)
client = SockSocket('AF_INET', 'SOCK_STREAM', 0)
say 'sockets' (listen >= 0 & client >= 0)
local.family = 'AF_INET'
local.addr = '127.0.0.1'
local.port = 0
say 'bind' SockBind(listen, 'local.')
say 'listen' SockListen(listen, 1)
say 'name' SockGetSockName(listen, 'local.') local.addr (local.port > 0)
say 'connect' SockConnect(client, 'local.')
accepted = SockAccept(listen, 'peer.')
say 'accept' (accepted >= 0) peer.addr
say 'type' SockGetSockOpt(accepted, 'SOL_SOCKET', 'SO_TYPE', 'kind') kind
say 'set-timeout' SockSetSockOpt(accepted, 'SOL_SOCKET', 'SO_RCVTIMEO', 2000)
say 'get-timeout' SockGetSockOpt(accepted, 'SOL_SOCKET', 'SO_RCVTIMEO', 'timeout') timeout
say 'nonblocking' SockIoctl(accepted, 'FIONBIO', 1)
say 'empty-receive' SockRecv(accepted, 'data', 1) errno SockSock_Errno()
say 'blocking' SockIoctl(accepted, 'FIONBIO', 0)
read.0 = 1
read.1 = accepted
say 'select-empty' SockSelect('read.', , , 0.02)
say 'send' SockSend(client, 'x')
say 'receive' SockRecv(accepted, 'data', 1) data
exception.0 = 1
exception.1 = accepted
say 'urgent-send' SockSend(client, '!', 'MSG_OOB')
say 'select-exception' SockSelect(, , 'exception.', 1) exception.0
say 'urgent-receive' SockRecv(accepted, 'urgent', 1, 'MSG_OOB') urgent
say 'close' SockClose(client) SockClose(accepted) SockClose(listen)
say 'closed' SockClose(client) errno SockSock_Errno()
::requires 'rxsock' library
