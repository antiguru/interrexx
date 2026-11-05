/*----------------------------------------------------------------------------*/
/* socket.cls: the Socket, InetAddress and HostInfo classes over rxsock.      */
/*                                                                            */
/* Written for this crate from the extension's documented behaviour;          */
/* its line numbers are its own (Deviation 89).                               */
/*----------------------------------------------------------------------------*/

::requires 'rxsock' LIBRARY

/* The option names getOption and setOption accept, in the description's     */
/* order, which is the order a 93.914 lists them in.                          */
::routine optionNames
  return 'SO_DEBUG SO_REUSEADDR SO_TYPE SO_ERROR SO_DONTROUTE SO_BROADCAST' -
    'SO_SNDBUF SO_RCVBUF SO_KEEPALIVE SO_OOBINLINE SO_LINGER SO_RCVLOWAT' -
    'SO_SNDLOWAT SO_RCVTIMEO SO_SNDTIMEO SO_USELOOPBACK'

/* The allowed values of a 93.914, one per line.                              */
::routine allowed
  use arg words
  return words~space(1)~changeStr(' ', '0a'x)

/* The name of an address family number; other numbers stay numbers.         */
::routine familyName
  use arg number
  names = .directory~new
  names[0] = 'AF_UNSPEC';    names[1] = 'AF_LOCAL';     names[2] = 'AF_INET'
  names[3] = 'AF_AX25';      names[4] = 'AF_IPX';       names[5] = 'AF_APPLETALK'
  names[6] = 'AF_NETROM';    names[7] = 'AF_BRIDGE';    names[8] = 'AF_ATMPVC'
  names[9] = 'AF_X25';       names[10] = 'AF_INET6';    names[11] = 'AF_ROSE'
  names[12] = 'AF_DECnet';   names[13] = 'AF_NETBEUI';  names[14] = 'AF_SECURITY'
  names[15] = 'AF_KEY';      names[16] = 'AF_NETLINK';  names[17] = 'AF_PACKET'
  names[18] = 'AF_ASH';      names[19] = 'AF_ECONET';   names[20] = 'AF_ATMSVC'
  names[22] = 'AF_SNA';      names[23] = 'AF_IRDA';     names[24] = 'AF_PPPOX'
  names[25] = 'AF_WANPIPE';  names[31] = 'AF_BLUETOOTH'
  if names~hasIndex(number) then return names[number]
  return number

/* A new InetAddress from an address stem rxsock filled.                      */
::routine addressOf
  use arg stem.
  return .InetAddress~new(stem.!addr, stem.!port, familyName(stem.!family))


::class 'Socket' public

::method gethostbyname class
  use strict arg name
  return .HostInfo~new(name)

::method gethostbyaddr class
  use strict arg addr
  return .HostInfo~new(addr)

::method gethostid class
  use strict arg
  return SockGetHostId()

::method gethostname class
  use strict arg
  return SockGetHostName()

/* The object's errno: what the last rxsock call it made set.                 */
::attribute errno get

::method init
  expose socket errno
  errno = 0
  if arg() = 1 & arg(1, 'E') then do
    use arg socket
    if \datatype(socket, 'W') then
      raise syntax 88.905 array('socket', socket)
    if socket <= 0 then
      raise syntax 88.905 array('socket', socket)
    return
  end
  use strict arg domain = 'AF_INET', type = 'SOCK_STREAM', protocol = 0
  domain = domain~upper
  type = type~upper
  protocol = protocol~upper
  if domain \== 'AF_INET' then
    raise syntax 93.914 array('domain', 'AF_INET', domain)
  if wordpos(type, 'SOCK_STREAM SOCK_DGRAM SOCK_RAW') = 0 then
    raise syntax 93.914 array('type', allowed('SOCK_STREAM SOCK_DGRAM SOCK_RAW'), type)
  if wordpos(protocol, '0 IPPROTO_UDP IPPROTO_TCP') = 0 then
    raise syntax 93.914 array('protocol', allowed('0 IPPROTO_UDP IPPROTO_TCP'), protocol)
  socket = SockSocket(domain, type, protocol)

::method string
  expose socket
  return socket

/* Unguarded, so that a close from another activity runs while it waits.     */
::method accept unguarded
  expose socket errno
  use strict arg
  accepted = SockAccept(socket)
  if accepted = -1 then return .nil
  return .Socket~new(accepted)

::method bind
  expose socket errno
  use strict arg hostaddr
  if \hostaddr~isA(.InetAddress) then
    raise syntax 88.914 array('hostaddr', 'INETADDRESS')
  address. = hostaddr~makeStem
  return SockBind(socket, 'address.!')

::method connect
  expose socket errno
  use strict arg hostaddr
  if \hostaddr~isA(.InetAddress) then
    raise syntax 88.914 array('hostaddr', 'INETADDRESS')
  address. = hostaddr~makeStem
  return SockConnect(socket, 'address.!')

::method close
  expose socket errno
  use strict arg
  call SockShutDown socket, 2
  closed = SockClose(socket)
  socket = -1
  return closed

::method getOption
  expose socket errno
  use strict arg name
  name = name~upper
  if wordpos(name, optionNames()) = 0 then
    raise syntax 93.914 array(1, allowed(optionNames()), name)
  if SockGetSockOpt(socket, 'SOL_SOCKET', name, 'value') = -1 then return .nil
  return value

::method setOption
  expose socket errno
  use strict arg name, value
  name = name~upper
  if wordpos(name, optionNames()) = 0 then
    raise syntax 93.914 array(1, allowed(optionNames()), name)
  if name \== 'SO_LINGER' & \datatype(value, 'W') then
    raise syntax 93.907 array(2, value)
  if SockSetSockOpt(socket, 'SOL_SOCKET', name, value) = -1 then return .nil
  return 0

::method getPeerName
  expose socket errno
  use strict arg
  if SockGetPeerName(socket, 'address.!') = -1 then return .nil
  return addressOf(address.)

::method getSockName
  expose socket errno
  use strict arg
  if SockGetSockName(socket, 'address.!') = -1 then return .nil
  return addressOf(address.)

/* FIONREAD's count goes to a variable of this method's, not to the caller.   */
::method ioctl
  expose socket errno
  use strict arg cmd, data
  answer = SockIoctl(socket, cmd, data)
  if answer = -1 then return .nil
  return answer

::method listen
  expose socket errno
  use strict arg backlog
  if \datatype(backlog, 'W') then
    raise syntax 88.904 array('backlog', backlog)
  if backlog < 0 then
    raise syntax 88.904 array('backlog', backlog)
  return SockListen(socket, backlog)

::method recv
  expose socket errno
  use strict arg length
  if \datatype(length, 'W') then
    raise syntax 88.911 array('length', length)
  if length < 0 then
    raise syntax 88.911 array('length', length)
  if SockRecv(socket, 'data', length) = -1 then return .nil
  return data

::method recvFrom
  expose socket errno
  use strict arg length, address
  if \datatype(length, 'W') then
    raise syntax 88.905 array('length', length)
  if length <= 0 then
    raise syntax 88.905 array('length', length)
  if \address~isA(.InetAddress) then
    raise syntax 88.914 array('address', 'INETADDRESS')
  if SockRecvFrom(socket, 'data', length, 'from.!') = -1 then return .nil
  address~family = familyName(from.!family)
  address~address = from.!addr
  address~port = from.!port
  return data

::method select
  expose errno
  use strict arg reads, writes, excepts, timeout
  sets = .array~of(reads, writes, excepts)
  names = .array~of('reads', 'writes', 'excepts')
  do i = 1 to 3
    if \sets[i]~isA(.Array) then
      raise syntax 88.914 array(names[i], 'Array')
  end
  r.0 = reads~items
  do i = 1 to r.0
    r.i = reads[i]~string
  end
  w.0 = writes~items
  do i = 1 to w.0
    w.i = writes[i]~string
  end
  e.0 = excepts~items
  do i = 1 to e.0
    e.i = excepts[i]~string
  end
  ready = SockSelect('r.', 'w.', 'e.', timeout)
  if ready = -1 then return .nil
  reads~empty
  do i = 1 to r.0
    reads~append(r.i)
  end
  writes~empty
  do i = 1 to w.0
    writes~append(w.i)
  end
  excepts~empty
  do i = 1 to e.0
    excepts~append(e.i)
  end
  return ready

::method send
  expose socket errno
  use strict arg data
  return SockSend(socket, data)

::method uninit
  expose socket
  if socket \= -1 then do
    call SockShutDown socket, 2
    call SockClose socket
    socket = -1
  end


::class 'InetAddress' public

::attribute address get
::attribute port get
::attribute family get

::method init
  use strict arg address, port, family = 'AF_INET'
  self~address = address
  self~port = port
  self~family = family

::method 'address='
  expose address
  use strict arg value
  if value~upper \== 'INADDR_ANY' then do
    if SockGetHostByName(value, 'host.!') then value = host.!addr
    if value~verify('0123456789.') \= 0 then
      raise syntax 93.953 array(1, 'dotted decimal internet address')
  end
  address = value

::method 'port='
  expose port
  use strict arg value
  if \datatype(value, 'W') then
    raise syntax 88.907 array('port', 0, 65535, value)
  if value < 0 | value > 65535 then
    raise syntax 88.907 array('port', 0, 65535, value)
  port = value

::method 'family='
  expose family
  use strict arg value
  value = value~upper
  if value \== 'AF_INET' then
    raise syntax 93.914 array('family', 'AF_INET', value)
  family = value

::method makeStem
  expose address port family
  stem. = .stem~new
  stem.!addr = address
  stem.!family = family
  stem.!port = port
  return stem.


::class 'HostInfo' public

::attribute name get
::attribute address get
::attribute addr get
::attribute alias get

::method init
  expose name address addr alias
  use strict arg name
  if name = '' then
    raise syntax 93.901 array(1)
  /* The counts start at zero, so that a failed lookup leaves the Arrays     */
  /* empty; its address is then the unset element's name.                    */
  hostinfo.!addr.0 = 0
  hostinfo.!alias.0 = 0
  if name~verify('0123456789.') = 0 then
    call SockGetHostByAddr name, 'hostinfo.!'
  else
    call SockGetHostByName name, 'hostinfo.!'
  address = hostinfo.!addr
  addr = .array~new
  do i = 1 to hostinfo.!addr.0
    addr~append(hostinfo.!addr.i)
  end
  alias = .array~new
  do i = 1 to hostinfo.!alias.0
    alias~append(hostinfo.!alias.i)
  end

::method makeStem
  expose name address addr alias
  stem. = .stem~new
  stem.!name = name
  stem.!addr = address
  stem.!addrtype = 'AF_INET'
  stem.!addr.0 = addr~items
  do i = 1 to addr~items
    stem.!addr.i = addr[i]
  end
  stem.!alias.0 = alias~items
  do i = 1 to alias~items
    stem.!alias.i = alias[i]
  end
  return stem.
