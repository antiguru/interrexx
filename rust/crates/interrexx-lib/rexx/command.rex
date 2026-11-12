/* Command classes. Spec rules: class.commandhandler.*,
   class.commandcontext.*, class.commandinput.*, class.commandoutput.*,
   class.nativecommandhandler.*. */

-- r[class.commandhandler.creation]
::class "CommandHandler" public mixinclass Object

-- r[class.commandhandler.command]
::method command abstract

-- r[class.commandhandler.environments]
::method environments class external 'LIBRARY REXX cmdhandler_environments'

-- r[class.commandhandler.handler]
::method handler class external 'LIBRARY REXX cmdhandler_handler'

-- r[class.commandhandler.handlers]
::method handlers class external 'LIBRARY REXX cmdhandler_handlers'

::method '!register' class private external 'LIBRARY REXX cmdhandler_register'

::method '!deregister' class private external 'LIBRARY REXX cmdhandler_deregister'

-- r[class.commandhandler.register], r[class.commandhandler.name],
-- r[class.commandhandler.register.sends]: the name is read first, then the
-- handler is asked isA, once.
::method register class
  use strict arg name, handler
  name = name~request('String')
  if name == .nil | \name~isA(.string) then raise syntax 93.900 array('name')
  if name == '' then raise syntax 93.900 array('name')
  accepted = handler~isA(.CommandHandler)
  if accepted \== 1 then raise syntax 93.900 array('handler')
  return self~!register(name, handler)

-- r[class.commandhandler.deregister]
::method deregister class
  use strict arg name
  name = name~request('String')
  if name == .nil | \name~isA(.string) then raise syntax 93.900 array('name')
  if name == '' then raise syntax 93.900 array('name')
  if self~handler(name) == .nil then raise syntax 93.900,
    array('deregister: there is no command environment named' name)
  return self~!deregister(name)


/* The context of one command. `cself` is the identity the interpreter keeps
   for the running command; `pending` is the one condition the handler asked
   for. */
-- r[class.commandcontext.creation], r[class.commandcontext.init]
::class "CommandContext" public

::method init
  expose cself environment pending inStream outStream errStream
  use strict arg cself, environment
  if \var('pending') then pending = .nil
  if \var('inStream') then inStream = .nil
  if \var('outStream') then outStream = .nil
  if \var('errStream') then errStream = .nil

-- r[class.commandcontext.environment]
::attribute environment get

-- The condition the handler asked for, as the interpreter reads it back.
::attribute pending get

::method '!check' private external 'LIBRARY REXX cmdcontext_check'
::method '!getvar' private external 'LIBRARY REXX cmdcontext_getVar'
::method '!hasvar' private external 'LIBRARY REXX cmdcontext_hasVar'
::method '!setvar' private external 'LIBRARY REXX cmdcontext_setVar'
::method '!dropvar' private external 'LIBRARY REXX cmdcontext_dropVar'
::method '!forward' external 'LIBRARY REXX cmdcontext_forward'

-- r[class.commandcontext.valid]
::method isValid external 'LIBRARY REXX cmdcontext_isValid'

-- r[class.commandcontext.isredirected]
::method isRedirected external 'LIBRARY REXX cmdcontext_isRedirected'

-- r[class.commandcontext.variables]
::method variables external 'LIBRARY REXX cmdcontext_variables'

-- r[class.commandcontext.name]: the name is read as the string that
-- `request` answers; what is no string is no name.
::method '!varname' private
  use arg name
  name = name~request('String')
  if name == .nil | \name~isA(.string) then raise syntax 93.900 array('name')
  return name

-- The text of a variable that has no value: its name in upper case, with
-- the tail of a compound name resolved.
::method '!derivedname' private
  use arg name
  name = name~upper
  p = name~pos('.')
  if p = 0 | p = name~length then return name
  derived = name~left(p)
  rest = name~substr(p + 1)
  do forever
    q = rest~pos('.')
    if q = 0 then do
      part = rest
      rest = ''
    end
    else do
      part = rest~left(q - 1)
      rest = rest~substr(q + 1)
    end
    if part \== '' & \part~left(1)~datatype('W') then part = self~getVar(part)~string
    derived = derived || part
    if q = 0 then leave
    derived = derived || '.'
  end
  return derived

-- r[class.commandcontext.getvar]
::method getVar
  use strict arg name
  name = self~!varname(name)
  if self~!hasvar(name) then return self~!getvar(name)
  return self~!derivedname(name)

::method '[]'
  use strict arg name
  return self~getVar(name)

-- r[class.commandcontext.hasvar]
::method hasVar
  use strict arg name
  return self~!hasvar(self~!varname(name))

-- r[class.commandcontext.setvar]
::method setVar
  use strict arg name, value
  self~!setvar(self~!varname(name), value)

::method '[]='
  use strict arg value, name
  self~!setvar(self~!varname(name), value)

-- r[class.commandcontext.dropvar]
::method dropVar
  use strict arg name
  self~!dropvar(self~!varname(name))

-- r[class.commandcontext.input]
::method input
  expose cself inStream
  if inStream \== .nil then return inStream
  if \self~isRedirected('Input') then raise syntax 98.900 array('input')
  inStream = .CommandInput~new(cself)
  return inStream

-- r[class.commandcontext.output]
::method output
  expose cself outStream
  if outStream \== .nil then return outStream
  if \self~isRedirected('Output') then raise syntax 98.900 array('output')
  outStream = .CommandOutput~new(cself, 'O')
  return outStream

-- r[class.commandcontext.error]
::method error
  expose cself errStream
  if errStream \== .nil then return errStream
  if \self~isRedirected('Error') then raise syntax 98.900 array('error')
  errStream = .CommandOutput~new(cself, 'E')
  return errStream

-- The interpreter sends this when the handler returns: text the output
-- streams still keep is written (r[class.commandoutput.pending]).
::method finish
  expose outStream errStream
  if outStream \== .nil then outStream~flush
  if errStream \== .nil then errStream~flush

-- r[class.commandcontext.raise], r[class.commandcontext.raise.syntax],
-- r[class.commandcontext.raise.once]
::method raise
  expose pending
  use strict arg condition, rc, description = .nil, additional = .nil
  self~!check
  kind = condition~request('String')
  if kind == .nil | \kind~isA(.string) then raise syntax 93.900 array('condition')
  kind = space(translate(kind, ' ', '09'x), 1)~upper
  wordcount = kind~words
  select
    when wordcount = 1 & 'ERROR FAILURE HALT LOSTDIGITS NOMETHOD NOSTRING NOTREADY NOVALUE'~wordpos(kind) > 0 then nop
    when wordcount = 2 & kind~word(1) == 'USER' then nop
    when kind == 'SYNTAX' & \arg(4, 'E') then do
      -- The second form: `description` holds `additional`.
      -- The number is major * 1000 + minor, as the interpreter reads it.
      text = rc~request('String')
      if text == .nil | \text~isA(.string) then raise syntax 93.900 array('code')
      parse value text with major '.' minor
      if text~countStr('.') \= 1 then raise syntax 93.900 array('code')
      if major == '' | verify(major, '0123456789') > 0 then raise syntax 93.900 array('code')
      if minor == '' | verify(minor, '0123456789') > 0 then raise syntax 93.900 array('code')
      major = major~strip('L', '0')
      minor = minor~strip('L', '0')
      if major == '' | major~length > 2 then raise syntax 93.900 array('code')
      if minor == '' | minor~length > 3 then raise syntax 93.900 array('code')
      number = major * 1000 + minor
      if pending \== .nil then raise syntax 93.900 array('condition')
      if description == .nil then description = .array~new
      else if \description~isA(.array) then description = .array~of(description)
      pending = .array~of('SYNTAX', number, .nil, description)
      return rc
    end
    otherwise raise syntax 93.900 array('condition')
  end
  if pending \== .nil then raise syntax 93.900 array('condition')
  pending = .array~of(kind, rc, description, additional)
  return rc

-- r[class.commandcontext.raiseerror]
::method raiseError
  use strict arg rc
  return self~raise('ERROR', rc)

-- r[class.commandcontext.raisefailure]
::method raiseFailure
  use strict arg rc
  return self~raise('FAILURE', rc)


-- r[class.commandinput.creation]: `q` holds the lines still to come and is
-- shared with a copy, so that the two read one text; `buf` is the line that
-- has been partly read, which a copy has its own record of.
::class "CommandInput" public inherit InputStream

::method init
  expose cself q buf
  use strict arg cself
  q = .queue~new
  buf = ''

::method '!readline' private external 'LIBRARY REXX cmdcontext_lineIn'

-- Takes the next line of the command's input into the queue. Asking the
-- command is also how a stream finds out that its command is over.
::method '!nextline' private
  expose q
  line = self~!readline
  if line \== .nil then q~queue(line)

-- r[class.commandinput.linein]
::method lineIn
  expose q buf
  use arg line, count
  if arg() > 2 then raise syntax 93.902 array('LINEIN', 2)
  if arg(1, 'E') then raise syntax 93.963 array('LINEIN', 1)
  if \arg(2, 'E') then count = 1
  if count \== '1' & count \== '0' then raise syntax 93.963 array('LINEIN', 2)
  self~!nextline
  if count == '0' then return ''
  if buf \== '' then do
    rest = buf~left(buf~length - 1)
    buf = ''
    return rest
  end
  if q~items = 0 then raise notready return ''
  return q~pull

-- r[class.commandinput.charin]
::method charIn
  expose q buf
  use arg start, count
  if arg() > 2 then raise syntax 93.902 array('CHARIN', 2)
  if arg(1, 'E') then raise syntax 93.963 array('CHARIN', 1)
  if \arg(2, 'E') then count = 1
  else do
    if \datatype(count, 'N') then raise syntax 93.943 array('CHARIN', 2, count)
    if \datatype(count, 'W') | count < 0 then raise syntax 93.923 array('CHARIN', 2, count)
  end
  self~!nextline
  if count = 0 then return ''
  out = ''
  do while count > 0
    if buf == '' then do
      if q~items = 0 then leave
      buf = q~pull || '0a'x
    end
    take = min(count, buf~length)
    out = out || buf~left(take)
    buf = buf~substr(take + 1)
    count = count - take
  end
  if out == '' then raise notready return ''
  return out

-- r[class.commandinput.lines]
::method lines
  expose q buf
  self~!nextline
  if buf \== '' | q~items > 0 then return 1
  return 0

-- r[class.commandinput.chars]
::method chars
  expose q buf
  self~!nextline
  if buf \== '' then return buf~length
  if q~items > 0 then return q~peek~length + 1
  return 0


-- r[class.commandoutput.creation]: `which` is `O` for the output and `E` for
-- the error output; `kept` is the text of a line that is not finished.
::class "CommandOutput" public inherit OutputStream

::method init
  expose cself which kept
  use strict arg cself, which
  kept = ''

::method '!write' private external 'LIBRARY REXX cmdcontext_write'

-- r[class.commandoutput.lineout]
::method lineOut
  expose which kept
  use arg string
  if arg() > 1 then raise syntax 93.902 array('LINEOUT', 1)
  if arg(1, 'E') then text = kept || string~string
  else do
    if kept == '' then return 0
    text = kept
  end
  self~!write(which, text)
  kept = ''
  return 0

-- r[class.commandoutput.charout]
::method charOut
  expose which kept
  use arg string, position
  if arg() > 2 then raise syntax 93.902 array('CHAROUT', 2)
  if arg(2, 'E') then raise syntax 93.963 array('CHAROUT', 2)
  if \arg(1, 'E') then return 0
  text = kept || string~string
  p = text~lastPos('0a'x)
  if p = 0 then do
    kept = text
    return 0
  end
  head = text~left(p - 1)
  kept = text~substr(p + 1)
  do forever
    q = head~pos('0a'x)
    if q = 0 then do
      self~!write(which, head)
      leave
    end
    self~!write(which, head~left(q - 1))
    head = head~substr(q + 1)
  end
  return 0

-- r[class.commandoutput.flush]
::method flush
  expose which kept
  if kept == '' then return
  self~!write(which, kept)
  kept = ''

-- r[class.commandoutput.close]
::method close
  expose which kept
  if kept == '' then return
  self~!write(which, kept)
  kept = ''


-- r[class.nativecommandhandler.creation]
::class "NativeCommandHandler" public inherit CommandHandler

-- r[class.nativecommandhandler.creation]: a program makes none.
::method new class private
  forward class (super)

-- r[class.nativecommandhandler.name]
::method name
  expose name
  return name

-- r[class.nativecommandhandler.kind]
::method kind external 'LIBRARY REXX cmdnative_kind'

-- r[class.nativecommandhandler.command]
::method command unguarded
  use strict arg command, caller
  return caller~!forward(self, command)
