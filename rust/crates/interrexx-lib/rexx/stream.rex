/* The stream classes: InputStream, OutputStream, InputOutputStream and Stream.
   The three mixin classes give the methods that a class built on them has
   until it overrides them; Stream binds the native entry points for most of
   its methods and does the argument handling of the rest here.
   See spec/src/classes/inputstream.md, outputstream.md,
   inputoutputstream.md and stream.md. */

/* r[class.inputstream.creation] */
::class "InputStream" public mixinclass Object

/* r[class.inputstream.arrayin] and r[class.inputstream.arrayin.sends]: one
   lineIn per item and one more, the NOTREADY of which ends the reading. */
::method arrayIn
  items = .array~new
  signal on notready name done
  do forever
    items~append(self~lineIn)
  end
done:
  return items

/* r[class.inputstream.abstract] */
::method charIn abstract
::method chars abstract
::method lineIn abstract
::method lines abstract

/* r[class.inputstream.unsupported] */
::method charOut
  raise syntax 93.963
::method lineOut
  raise syntax 93.963
::method position
  raise syntax 93.963

/* r[class.inputstream.nop] */
::method open
::method close


/* r[class.outputstream.creation] */
::class "OutputStream" public mixinclass Object

/* r[class.outputstream.arrayout] and r[class.outputstream.arrayout.sends]:
   a NOTREADY that a lineOut raises is not looked at. */
::method arrayOut
  use strict arg collection
  items = collection~request('ARRAY')
  if .nil == items then raise syntax 98.913 array (collection)
  if \items~isA(.array) then raise syntax 98.913 array (collection)
  do item over items
    self~lineOut(item)
  end

/* r[class.outputstream.abstract] */
::method charOut abstract
::method lineOut abstract

/* r[class.outputstream.unsupported] */
::method charIn
  raise syntax 93.963
::method chars
  raise syntax 93.963
::method lineIn
  raise syntax 93.963
::method lines
  raise syntax 93.963
::method position
  raise syntax 93.963

/* r[class.outputstream.nop] */
::method open
::method close


/* r[class.inputoutputstream.creation] */
::class "InputOutputStream" public mixinclass Object inherit InputStream OutputStream


/* r[class.stream.creation] */
::class "Stream" public mixinclass InputOutputStream

/* r[class.stream.new], r[class.stream.new.sends] and r[class.stream.init]:
   the count of the argument is checked, then makeString is sent to a name
   that is not a string, once. The Rust half records the name and what it
   resolves to; the names of the standard streams and HANDLE: names mark the
   stream afterwards. */
::method init
  expose stream_name
  use strict arg name
  text = name~request('STRING')
  if .nil == text then raise syntax 93.938 array (1)
  stream_name = text
  self~!c_stream_init(text)
  upper = text~translate
  if upper == 'STDIN' | upper == 'STDIN:' | upper == 'STDOUT' | upper == 'STDOUT:' | upper == 'STDERR' | upper == 'STDERR:' then
    self~!std_set
  else if upper~left(7) == 'HANDLE:' then self~!handle_set(text~substr(8))

/* r[class.stream.string] */
::method string
  expose stream_name
  if arg() > 0 then raise syntax 93.902 array (0)
  return stream_name

::method !c_stream_init external 'LIBRARY REXX stream_init'
::method !std_set external 'LIBRARY REXX std_set'
::method !handle_set external 'LIBRARY REXX handle_set'
::method !query_exists external 'LIBRARY REXX query_exists'
::method !query_size external 'LIBRARY REXX query_size'
::method !query_time external 'LIBRARY REXX query_time'
::method !query_handle external 'LIBRARY REXX query_handle'
::method !query_streamtype external 'LIBRARY REXX query_streamtype'
::method !query_position external 'LIBRARY REXX stream_query_position'

/* r[class.stream.qualify], r[class.stream.state] and the methods that do
   what the stream functions do. */
::method qualify external 'LIBRARY REXX qualify'
::method state external 'LIBRARY REXX stream_state'
::method description external 'LIBRARY REXX stream_description'
::method open external 'LIBRARY REXX stream_open'
::method close external 'LIBRARY REXX stream_close'
::method flush external 'LIBRARY REXX stream_flush'
::method charIn external 'LIBRARY REXX stream_charin'
::method charOut external 'LIBRARY REXX stream_charout'
::method lineIn external 'LIBRARY REXX stream_linein'
::method lineOut external 'LIBRARY REXX stream_lineout'
::method chars external 'LIBRARY REXX stream_chars'
::method lines external 'LIBRARY REXX stream_lines'
::method position external 'LIBRARY REXX stream_position'
::method seek external 'LIBRARY REXX stream_position'
::method uninit external 'LIBRARY REXX stream_uninit'
::method line_arrayIn private external 'LIBRARY REXX stream_arrayin'

/* r[class.stream.query]: the word, shortened as far as its rule allows, and
   what follows it. No message goes to anything but the receiver's own
   entry points. */
::method query
  use strict arg request
  text = self~interrexx_value(request)
  parse upper var text word rest
  rest = rest~strip
  select
    when 'POSITION'~abbrev(word, 1) then return self~!query_position(rest)
    when rest \== '' then raise syntax 93.0
    when 'EXISTS'~abbrev(word, 1) then return self~!query_exists
    when 'SIZE'~abbrev(word, 2) then return self~!query_size
    when 'STREAMTYPE'~abbrev(word, 2) then return self~!query_streamtype
    when 'HANDLE'~abbrev(word, 1) then return self~!query_handle
    when word == '' | 'TIMESTAMP'~abbrev(word, 1) | 'DATETIME'~abbrev(word, 1) then do
      stamp = self~!query_time
      if stamp == '' then return ''
      parse var stamp . month day clock year .
      month = 'JanFebMarAprMayJunJulAugSepOctNovDec'~pos(month) % 3 + 1
      if word \== '' & 'TIMESTAMP'~abbrev(word, 1) then
        return year || '-' || month~right(2, '0') || '-' || day~right(2, '0') clock
      return month~right(2, '0') || '-' || day~right(2, '0') || '-' || year~right(2, '0') clock
    end
    otherwise raise syntax 93.0
  end

/* r[class.stream.command] and r[class.stream.command.sends] */
::method command
  use strict arg command
  text = self~interrexx_value(command)~strip('L')
  blank = text~pos(' ')
  if blank = 0 then do
    word = text
    rest = ''
  end
  else do
    word = text~left(blank - 1)
    rest = text~substr(blank + 1)
  end
  word = word~translate
  select
    when word == '' then forward message 'CLOSE' arguments (.array~new)
    when 'OPEN'~abbrev(word, 1) then forward message 'OPEN' arguments (.array~of(rest~translate))
    when 'CLOSE'~abbrev(word, 1) then do
      if rest~strip \== '' then raise syntax 93.0
      forward message 'CLOSE' arguments (.array~new)
    end
    when 'FLUSH'~abbrev(word, 1) then do
      if rest~strip \== '' then raise syntax 93.0
      forward message 'FLUSH' arguments (.array~new)
    end
    when 'SEEK'~abbrev(word, 1) then forward message 'POSITION' arguments (.array~of(rest~translate))
    when 'POSITION'~abbrev(word, 1) then forward message 'POSITION' arguments (.array~of(rest~translate))
    when 'QUERY'~abbrev(word, 1) then forward message 'QUERY' arguments (.array~of(rest~translate))
    otherwise raise syntax 93.914 array (1, 'CLOSE FLUSH OPEN POSITION QUERY SEEK', text)
  end

/* The string value of an argument that is read through makeString, and
   through string for an object that has only that. */
::method interrexx_value private
  use arg value
  text = value~request('STRING')
  if .nil == text then text = value~string
  return text

/* r[class.stream.say] and r[class.stream.say.sends] */
::method say unguarded
  use strict arg line = ''
  return self~lineOut(line)

/* r[class.stream.arrayin] and r[class.stream.arrayin.sends]: the lines are
   read by the entry point directly; for the characters one charIn is sent
   for each, and one more, the NOTREADY of which ends the reading. */
::method arrayIn
  if arg() > 1 then raise syntax 93.902 array (1)
  if arg() = 0 then kind = 'L'
  else do
    option = arg(1)
    if option~caselessStartsWith('L') then kind = 'L'
    else if option~caselessStartsWith('C') then kind = 'C'
    else raise syntax 93.915 array ('CL', option)
  end
  if kind == 'L' then count = self~lines
  else count = self~chars
  if \datatype(count, 'W') then raise syntax 93.906 array (1, count)
  if count < 0 then raise syntax 93.906 array (1, count)
  items = .array~new
  signal on notready name done
  if kind == 'L' then self~line_arrayIn(items)
  else do forever
    items~append(self~charIn)
  end
done:
  return items

/* r[class.stream.makearray] and r[class.stream.makearray.sends] */
::method makeArray
  if arg() > 1 then raise syntax 93.902 array (1)
  if arg() = 0 then return self~arrayIn
  return self~arrayIn(arg(1))

/* r[class.stream.arrayout] and r[class.stream.arrayout.sends]: the option is
   checked first; a NOTREADY ends the writing at the item that raised it. */
::method arrayOut
  expose stream_name
  use strict arg collection, option = 'L'
  letter = option~left(1)~translate
  if letter \== 'L' & letter \== 'C' then raise syntax 93.0
  items = collection~request('ARRAY')
  if .nil == items then raise syntax 98.913 array (collection)
  if \items~isA(.array) then raise syntax 98.913 array (collection)
  total = items~items
  written = 0
  signal on notready name stopped
  do item over items
    if letter == 'L' then self~lineOut(item)
    else self~charOut(item)
    written += 1
  end
  return 0
stopped:
  unwritten = total - written
  raise notready description (stream_name) return (unwritten)

/* r[class.stream.supplier] */
::method supplier
  if arg() > 0 then raise syntax 93.902 array (0)
  return .StreamSupplier~new(self)
