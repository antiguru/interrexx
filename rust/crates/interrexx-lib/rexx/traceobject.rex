/* TraceObject: r[class.traceobject.*]. The class attributes live in the
   variable pool of the class that receives the message, so a subclass counts,
   collects and formats on its own. */

::class "TraceObject" subclass StringTable public

/* r[class.traceobject.collector], r[class.traceobject.notify],
   r[class.traceobject.counter], r[class.traceobject.option]: an unset
   attribute reads as its default (r[class.traceobject.arguments]). */
::attribute collector class get
  expose collector
  if arg() > 0 then raise syntax 93.902 array (0)
  if \var('collector') then collector = .nil
  return collector
::attribute collector class set

::attribute notify class get
  expose notify
  if arg() > 0 then raise syntax 93.902 array (0)
  if \var('notify') then notify = .nil
  return notify
::attribute notify class set

::attribute counter class get
  expose counter
  if arg() > 0 then raise syntax 93.902 array (0)
  if \var('counter') then counter = 0
  return counter

::attribute option class get
  expose option
  if arg() > 0 then raise syntax 93.902 array (0)
  if \var('option') then option = 'N'
  return option

/* r[class.traceobject.option]: the first character, if one of the five
   letters, in upper case. */
::attribute option class set
  expose option
  if arg(1, 'E') then do
    given = arg(1)
    if given~isInstanceOf(.string) then text = given~makeString:.string
    else text = given~request('STRING')
    if text~isInstanceOf(.string) then do
      letter = translate(left(text, 1))
      if pos(letter, 'NTSFP') > 0 then do
        option = letter
        return
      end
    end
  end
  raise syntax 93.914 array (1, '"N" (normal), "T" (thread), "S" (standard), "F" (full) or "P" (profiling)', arg(1))

/* r[class.traceobject.new]: the object, then its three entries, then the
   collector. */
::method new class
  expose counter option collector
  if \var('counter') then counter = 0
  if \var('option') then option = 'N'
  if \var('collector') then collector = .nil
  forward class (super) arguments (arg(1, 'A')) continue
  object = result
  counter = counter + 1
  object['TIMESTAMP'] = .DateTime~new
  object['OPTION'] = option
  object['NUMBER'] = counter
  if collector \== .nil then collector~append(object)
  return object

/* r[class.traceobject.counter]: back to the starting state. */
::method activate class
  expose counter option collector notify custom
  counter = 0
  option = 'N'
  collector = .nil
  notify = .nil
  drop custom

/* r[class.traceobject.setmakestring] */
::method setMakeString class
  expose custom
  n = arg()
  do while n > 0
    if arg(n, 'E') then leave
    n = n - 1
  end
  if n < 1 then raise syntax 93.901 array (1)
  if n > 1 then raise syntax 93.902 array (1)
  source = arg(1)
  if source~isInstanceOf(.method) then custom = source
  else if source~isInstanceOf(.string) | source~isInstanceOf(.array) then custom = .method~new('MAKESTRINGIMPL', source)
  else raise syntax 93.961 array (1)

::method unsetMakeString class
  expose custom
  if arg() > 0 then raise syntax 93.902 array (0)
  drop custom

/* r[class.traceobject.makestringimpl], r[class.traceobject.makestringimpl.method],
   r[class.traceobject.makestringimpl.sends] */
::method makeStringImpl class
  expose option custom
  if arg() < 1 then raise syntax 93.901 array (1)
  if arg() > 1 then raise syntax 93.902 array (1)
  object = arg(1)
  if object~isInstanceOf(.string) then raise syntax 93.924 array (object)
  if var('custom') then return self~run(custom, 'A', .array~of(object))
  if \var('option') then option = 'N'
  format = option
  if format == 'P' then do
    chosen = object['OPTION']
    if chosen == .nil then format = 'N'
    else if chosen = 'N' | chosen = 'P' then format = 'N'
    else if chosen = 'T' then format = 'T'
    else if chosen = 'S' then format = 'S'
    else format = 'F'
  end
  if format == 'N' then do
    line = object['TRACELINE']
    if line == .nil then return '?'
    return line
  end
  if format == 'T' then do
    line = object~traceline
    thread = object['THREAD']
    if line == .nil then line = '?'
    if thread == .nil then thread = '?'
    if length(line) < 10 then tail = ' '
    else tail = ''
    return left(line, min(9, length(line) + 3)) || thread || substr(line, 10) || tail
  end
  if format == 'F' then do
    interpreter = object['INTERPRETER']
    if interpreter == .nil then interpreter = '?'
  end
  thread = object['THREAD']
  invocation = object['INVOCATION']
  pool = object['ATTRIBUTEPOOL']
  if thread == .nil then thread = '?'
  if invocation == .nil then invocation = '?'
  if pool \== .nil then do
    guarded = object['ISGUARDED']
    locked = object['HASSCOPELOCK']
    count = object['SCOPELOCKCOUNT']
  end
  line = object['TRACELINE']
  if line == .nil then line = '?'
  if format == 'S' then do
    prefix = '[T' || thread || copies(' ', max(3 - length(thread), 1))
    prefix = prefix || 'I' || invocation || copies(' ', max(2 - length(invocation), 0))
    poolWidth = 2; countWidth = 2
  end
  else do
    prefix = '[R' || interpreter || copies(' ', max(4 - length(interpreter), 1))
    prefix = prefix || 'T' || thread || copies(' ', max(4 - length(thread), 1))
    prefix = prefix || 'I' || invocation || copies(' ', max(4 - length(invocation), 0))
    poolWidth = 4; countWidth = 3
  end
  if pool == .nil then do
    if format == 'S' then return prefix || ']' || copies(' ', 15) || line
    return prefix || ']' || copies(' ', 18) || line
  end
  if guarded \== .nil & guarded \== 0 & guarded \== 1 then raise syntax 34.901 array (guarded)
  if locked \== .nil & locked \== 0 & locked \== 1 then raise syntax 34.901 array (locked)
  if count == .nil then count = '?'
  if guarded == .nil then first = '?'
  else if guarded == 1 then first = 'G'
  else first = 'U'
  if guarded == 1 & locked == 0 then second = 'u'
  else if guarded == 0 & locked == 1 then second = 'g'
  else second = ' '
  if locked == .nil then lock = '?'
  else if locked == 1 then lock = '* '
  else lock = '  '
  pool = pool || copies(' ', max(poolWidth - length(pool), 0))
  count = count || copies(' ', max(countWidth - length(count), 0))
  return prefix || ' ' || first || second || ' ' || 'A' || pool || ' ' || 'L' || count || ' ' || lock || '] ' || line

/* r[class.traceobject.makestring] */
::method makeString
  return self~class~makeStringImpl(self)

/* r[class.traceobject.compareto] */
::method compareTo
  numeric digits 9
  mine = self['NUMBER']
  theirs = arg(1)['NUMBER']
  if mine == .nil & theirs == .nil then return 0
  if mine == .nil then return -1
  if theirs == .nil then return 1
  if \datatype(mine, 'N') then raise syntax 41.1 array (mine)
  if \datatype(theirs, 'N') then raise syntax 41.1 array (theirs)
  if mine < theirs then return -1
  if mine > theirs then return 1
  return 0
