/*----------------------------------------------------------------------------*/
/* smtp.cls: an SMTP client over streamsocket.cls.                            */
/*                                                                            */
/* Written for this crate from the extension's documented behaviour;          */
/* its line numbers are its own (Deviation 89).                               */
/*----------------------------------------------------------------------------*/

::requires 'mime.cls'
::requires 'streamsocket.cls'


::class 'smtpmsg' public

::attribute from
::attribute subject

::method init
  expose from subject recipients content
  use strict arg
  from = ''
  subject = ''
  recipients = .array~new
  content = ''

::method content
  expose content
  return content~string

::method 'content='
  expose content
  use strict arg value
  if \(value~isA(.mimepart) | value~isA(.mimemultipart) | value~isA(.String)) then
    raise syntax 93.948 array('MIMEPART', 'MIMEMULTIPART')
  content = value

::method addRecipient
  expose recipients
  use strict arg recipient
  recipients~append(recipient)

::method recipients
  expose recipients
  return recipients


::class 'smtp' public

::attribute authid
::attribute cmdresponse
::attribute debug
::attribute password
::attribute response
::attribute smtperrno

/* The host's name a HostInfo holds, or the string set in its place.         */
::method localhost
  expose localhost
  if localhost~isA(.HostInfo) then return localhost~name
  return localhost

::method 'localhost='
  expose localhost
  use strict arg localhost

::method init
  use strict arg
  self~reset

/* The defaults init and logoff set. smtperrno is not one of them.           */
::method reset private
  expose authid password response cmdresponse debug host port localhost
  authid = ''
  password = ''
  response = ''
  cmdresponse = .array~new
  debug = .false
  host = ''
  port = 25
  /* The last character is the letter l: a name lookup that normally fails  */
  /* (an ooRexx defect).                                                     */
  localhost = .Socket~gethostbyaddr('127.0.0.l')

/* Reads a reply, every line of it, into cmdresponse; answers its lines.     */
::method reply private
  expose stream cmdresponse
  lines = .array~new
  line = stream~lineIn
  cmdresponse~append(line)
  lines~append(line)
  do while line~substr(4, 1) == '-'
    line = stream~lineIn
    cmdresponse~append(line)
    lines~append(line)
  end
  return lines

/* Sends a command and checks its reply's first digit against expected.     */
::method transact private
  expose stream cmdresponse smtperrno debug
  use arg command, expected
  cmdresponse~append(command)
  stream~lineOut(command)
  lines = self~reply
  last = lines~lastItem
  smtperrno = last~left(3)
  if last~left(1) == expected then return .true
  if debug \= .false then
    do line over lines
      say 'Error: unexpected response:' line
    end
  return .false

::method connect
  expose stream host port authid password localhost debug
  use strict arg target, newAuthid = .nil, newPassword = .nil
  if arg(2, 'E') then authid = newAuthid
  if arg(3, 'E') then password = newPassword
  parse var target host ':' port
  if port == '' then port = 25
  if \datatype(port, 'W') then return -1
  stream = .StreamSocket~new(host, port)
  stream~open
  self~reply
  if \self~transact('EHLO' localhost~name, 2) then return -1
  types = ''
  do line over self~cmdresponse
    if line~substr(5, 4) == 'AUTH' then do
      types = line~substr(5)
      leave
    end
  end
  if types~pos('LOGIN') = 0 then return 0
  if \self~transact('AUTH LOGIN', 3) then do
    if debug \= .false then say 'AUTH LOGIN failed on' host
    return -1
  end
  if \self~transact(authid~encodeBase64, 3) then do
    if debug \= .false then say 'AUTH LOGIN authid rejected on' host
    return -1
  end
  if \self~transact(password~encodeBase64, 2) then do
    if debug \= .false then say 'AUTH LOGIN password rejected on' host
    return -1
  end
  return 0

::method send
  expose stream response debug
  use strict arg msg
  if \msg~isA(.smtpmsg) then
    raise syntax 93.948 array(1, 'SMTPMsg')
  response = .array~new
  if \self~transact('MAIL FROM:' || msg~from, 2) then return -1
  recipients = msg~recipients
  do recipient over recipients
    if \self~transact('RCPT TO:' recipient, 2) then return -1
  end
  if \self~transact('DATA', 3) then return -1
  stream~lineOut('From:' msg~from)
  count = recipients~items
  do i = 1 to count
    line = recipients[i]
    if i < count then line = line || ','
    if i = 1 then stream~lineOut('To:' line)
    else stream~lineOut('    ' || line)
  end
  stream~lineOut('Subject:' msg~subject)
  stream~lineOut(msg~content)
  stream~lineOut('.')
  lines = self~reply
  if lines~lastItem~left(1) == '2' then return 0
  if debug \= .false then say 'Error: message not accepted:' lines~lastItem
  return -1

::method logoff
  expose stream
  use strict arg
  stream~lineOut('QUIT')
  stream~close
  self~reset
  return 0
