/*----------------------------------------------------------------------------*/
/* streamsocket.cls: a stream over a connected Socket.                        */
/*                                                                            */
/* Written for this crate from docs/superpowers/specs/extensions-behaviour.md */
/* ("streamsocket.cls"); its line numbers are its own (Deviation 89).         */
/*----------------------------------------------------------------------------*/

::requires 'socket.cls'

::class 'StreamSocket' public subclass InputOutputStream

/* socket: the Socket; host and port: where it connects; bufsize: the most   */
/* one read takes; buffer: what has arrived and not been read; state and     */
/* descr: the stream's state and description; isOpen: whether it is open.    */
::method init
  expose socket host port bufsize buffer state descr isOpen
  buffer = ''
  state = 'READY'
  descr = ''
  if arg() = 1 then do
    use strict arg socket
    if \socket~isA(.Socket) then
      raise syntax 93.914 array(1, 'a Socket class instance', socket)
    bufsize = 4096
    isOpen = .true
    peer = socket~getPeerName
    if peer \== .nil then do
      host = peer~address
      port = peer~port
    end
    return
  end
  use strict arg host, port, bufsize = 4096
  isOpen = .false
  socket = .Socket~new
  if socket~errno \= 0 then do
    state = 'ERROR'
    raise syntax 5.900 array('System resources exhausted: Socket' socket~errno)
  end

::method open
  expose socket host port state descr isOpen
  if isOpen then do
    state = 'READY'
    return 'READY:'
  end
  if socket~connect(.InetAddress~new(host, port)) \= 0 then do
    state = 'NOTREADY'
    descr = socket~errno
    return self~description
  end
  isOpen = .true
  state = 'READY'
  return 'READY:'

::method description
  expose state descr
  return (state':' descr)~strip

::method state
  expose state
  return state

::method string
  expose host port
  return host':'port

::method chars
  expose isOpen
  return isOpen

::method lines
  expose isOpen
  return isOpen

/* The stream stays open: a later read or write does not reconnect.          */
::method close
  expose socket state descr isOpen
  if isOpen then do
    if socket~close = -1 then do
      state = 'ERROR'
      descr = socket~errno
      return self~description
    end
  end
  state = 'READY'
  return self~description

/* Whether the stream is open, opening it where it is not.                   */
::method ready private
  expose isOpen
  if isOpen then return .true
  return self~open == 'READY:'

/* Sends data, answering the count. A failed send reproduces the oracle's    */
/* defect: it asks a string of its own for its errno, which raises 97.1      */
/* and leaves the state as it was (D10).                                      */
::method transmit private
  expose socket
  use arg data
  sent = socket~send(data)
  if sent = -1 then do
    s = 'S'
    descr = s~errno
  end
  return sent

::method lineOut
  use arg str = '', line
  if \self~ready then return 0
  return self~transmit(str || '0d0a'x)

::method say
  use arg str = ''
  if \self~ready then return
  self~transmit(str || '0d0a'x)

::method charOut
  use arg str, start
  if \self~ready then return 0
  return self~transmit(str)

/* Reads one block into the buffer; .false at end of file or on a failure.   */
::method fill private
  expose socket bufsize buffer
  data = socket~recv(bufsize)
  if data == .nil then return .false
  if data == '' then return .false
  buffer = buffer || data
  return .true

/* The end of the data: the stream becomes NOTREADY, which is raised.        */
::method ended private
  expose socket state descr
  state = 'NOTREADY'
  descr = socket~errno

::method lineIn
  expose buffer state
  use arg line, count = 1
  if count = 0 then return ''
  if count > 1 then raise syntax 93.908 array(2, 1)
  if \self~ready then return ''
  if state == 'NOTREADY' then raise notready return ''
  do while buffer~pos('0d0a'x) = 0
    if \self~fill then do
      if buffer \== '' then do
        line = buffer
        buffer = ''
        return line
      end
      self~ended
      raise notready return ''
    end
  end
  parse var buffer line '0d0a'x buffer
  return line

::method charIn
  expose buffer state
  use arg start, length = 1
  if datatype(length, 'W') then
    if length < 0 then raise syntax 93.906 array(2, length)
  if length = 0 then return ''
  if \self~ready then return ''
  if state == 'NOTREADY' then raise notready return ''
  do while buffer~length < length
    if \self~fill then do
      if buffer \== '' then do
        chars = buffer
        buffer = ''
        return chars
      end
      self~ended
      raise notready return ''
    end
  end
  chars = buffer~left(length)
  buffer = buffer~substr(length + 1)
  return chars

::method arrayIn
  raise syntax 93.963

::method arrayOut
  raise syntax 93.963

::method position
  raise syntax 93.963
