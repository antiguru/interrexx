/* Properties, Validate, ArgUtil and Singleton.
   Properties: r[class.properties.*], Validate: r[class.validate.*],
   Singleton: r[class.singleton.*]. */

::class "ArgUtil" public
/* No members: r[class.index.hierarchy] keeps `.ArgUtil` in `.environment`. */

::class "Singleton" public mixinclass Class

/* r[class.singleton.new]: the first `new` makes the object, `copy` answers it
   (r[class.singleton.copy]), every later `new` answers it. */
::method new
  expose single
  if var('single') then return single
  self~define('COPY', 'return self')
  forward class (super) arguments (arg(1, 'A')) continue
  single = result
  return single

::class "Properties" public subclass Directory

/* r[class.properties.put]: `put(value, name)`. Two arguments exactly, the
   value read as a string, then the name as a directory reads it. */
::method put
  n = arg()
  do while n > 0
    if arg(n, 'E') then leave
    n = n - 1
  end
  if n < 2 then raise syntax 93.901 array (2)
  if n > 2 then raise syntax 93.902 array (2)
  if \arg(1, 'E') then raise syntax 93.903 array (1)
  value = arg(1)
  if value~isInstanceOf(.string) then text = value~makeString:.string
  else do
    text = value~request('STRING')
    if \text~isInstanceOf(.string) then raise syntax 93.938 array (1)
    text = text~makeString:.string
  end
  self~put:super(text, arg(2))

/* r[class.properties.put]: `[name] = value` is `put(value, name)`. */
::method '[]='
  forward message 'PUT' class (.properties)

/* r[class.properties.put]: `setProperty(name, value)` is `put(value, name)`
   with the arguments counted as `put` counts them. */
::method setProperty
  n = arg()
  do while n > 0
    if arg(n, 'E') then leave
    n = n - 1
  end
  if n < 2 then raise syntax 93.901 array (2)
  if n > 2 then raise syntax 93.902 array (2)
  if \arg(1, 'E') then raise syntax 93.903 array (1)
  forward message 'PUT' class (.properties) arguments (.array~of(arg(2), arg(1)))

/* r[class.properties.getproperty] */
::method getProperty
  n = arg()
  do while n > 0
    if arg(n, 'E') then leave
    n = n - 1
  end
  if n < 1 then raise syntax 93.901 array (1)
  if n > 2 then raise syntax 93.902 array (2)
  if \arg(1, 'E') then raise syntax 93.903 array (1)
  item = self~at(arg(1))
  if item \== .nil then return item
  if arg(2, 'E') then return arg(2)
  return .nil

/* r[class.properties.getlogical] */
::method getLogical
  forward message 'GETPROPERTY' continue
  value = result
  if value~isInstanceOf(.string) then do
    if value == '1' | value == 'true' then return 1
    if value == '0' | value == 'false' then return 0
  end
  raise syntax 34.904 array (value)

/* r[class.properties.getwhole] */
::method getWhole
  numeric digits 9
  forward message 'GETPROPERTY' continue
  value = result
  text = value~request('STRING')
  if text~isInstanceOf(.string) then do
    if text~datatype('W') then return text
  end
  raise syntax 26.901 array (value)

/* r[class.properties.setlogical] */
::method setLogical
  n = arg()
  do while n > 0
    if arg(n, 'E') then leave
    n = n - 1
  end
  if n < 2 then raise syntax 93.901 array (2)
  if n > 2 then raise syntax 93.902 array (2)
  if \arg(1, 'E') then raise syntax 93.903 array (1)
  value = arg(2)
  if \value~datatype('O') then raise syntax 34.901 array (value)
  if value~isInstanceOf(.string) then flag = value~makeString:.string
  else flag = value
  if flag then text = 'true'
  else text = 'false'
  self~setProperty(arg(1), text)

/* r[class.properties.setwhole] */
::method setWhole
  numeric digits 9
  n = arg()
  do while n > 0
    if arg(n, 'E') then leave
    n = n - 1
  end
  if n < 2 then raise syntax 93.901 array (2)
  if n > 2 then raise syntax 93.902 array (2)
  if \arg(1, 'E') then raise syntax 93.903 array (1)
  value = arg(2)
  if \value~datatype('W') then raise syntax 93.905 array (2, value)
  if value~isInstanceOf(.string) then text = value~makeString:.string
  else do
    text = value~request('STRING')
    if \text~isInstanceOf(.string) then raise syntax 93.938 array (2)
    text = text~makeString:.string
  end
  self~setProperty(arg(1), text)

/* r[class.properties.load.class] */
::method load class
  n = arg()
  do while n > 0
    if arg(n, 'E') then leave
    n = n - 1
  end
  if n < 1 then raise syntax 93.901 array (1)
  if n > 1 then raise syntax 93.902 array (1)
  properties = .properties~new
  properties~load(arg(1))
  return properties

/* r[class.properties.load.format], r[class.properties.load] */
::method load
  n = arg()
  do while n > 0
    if arg(n, 'E') then leave
    n = n - 1
  end
  if n < 1 then raise syntax 93.901 array (1)
  if n > 1 then raise syntax 93.902 array (1)
  source = arg(1)
  if source~isInstanceOf(.string) | source~isInstanceOf(.file) then do
    stream = .stream~new(source~string)
    self~load(stream)
    stream~close
    return
  end
  blanks = ' ' || '09'x
  signal on notready name finished
  do forever
    line = source~linein
    first = verify(line, blanks)
    if first == 0 then iterate
    if substr(line, first, 2) == '--' then iterate
    equals = pos('=', line)
    if equals == 0 then do
      before = line
      value = ''
    end
    else do
      before = left(line, equals - 1)
      value = substr(line, equals + 1)
    end
    start = verify(before, blanks)
    if start == 0 then name = ''
    else do
      finish = length(before) - verify(reverse(before), blanks) + 1
      name = substr(before, start, finish - start + 1)
    end
    self~put(value, name)
  end
finished:
  return

/* r[class.properties.save] */
::method save
  n = arg()
  do while n > 0
    if arg(n, 'E') then leave
    n = n - 1
  end
  if n < 1 then raise syntax 93.901 array (1)
  if n > 1 then raise syntax 93.902 array (1)
  target = arg(1)
  if target~isInstanceOf(.string) | target~isInstanceOf(.file) then do
    stream = .stream~new(target~string)
    stream~open('write replace')
    self~save(stream)
    stream~close
    return
  end
  entries = self~supplier
  do while entries~available
    item = entries~item
    if \item~isInstanceOf(.string) then item = item~request('STRING')
    target~lineout(entries~index || '=' || item)
    entries~next
  end

::class "Validate" public

/* Each check hands its arguments to `!run`, which counts them first
   (r[class.validate.arguments]), reads `digits`
   (r[class.validate.digits]) and then judges the value. */
::method classType class
  self~!run('classType', arg(1, 'A'))
::method length class
  self~!run('length', arg(1, 'A'))
::method logical class
  self~!run('logical', arg(1, 'A'))
::method nonNegativeNumber class
  self~!run('nonNegativeNumber', arg(1, 'A'))
::method nonNegativeWholeNumber class
  self~!run('nonNegativeWholeNumber', arg(1, 'A'))
::method number class
  self~!run('number', arg(1, 'A'))
::method numberRange class
  self~!run('numberRange', arg(1, 'A'))
::method position class
  self~!run('position', arg(1, 'A'))
::method positiveNumber class
  self~!run('positiveNumber', arg(1, 'A'))
::method positiveWholeNumber class
  self~!run('positiveWholeNumber', arg(1, 'A'))
::method requestClassType class
  return self~!run('requestClassType', arg(1, 'A'))
::method wholeNumber class
  self~!run('wholeNumber', arg(1, 'A'))
::method wholeNumberRange class
  self~!run('wholeNumberRange', arg(1, 'A'))

::method '!run' class private
  use arg kind, args
  select
    when kind == 'number' | kind == 'logical' then do
      least = 2; most = 2
    end
    when kind == 'classType' | kind == 'requestClassType' then do
      least = 3; most = 3
    end
    when kind == 'numberRange' | kind == 'wholeNumberRange' then do
      least = 4; most = 5
    end
    otherwise do
      least = 2; most = 3
    end
  end
  n = args~size
  do while n > 0
    if args~hasIndex(n) then leave
    n = n - 1
  end
  if n < least then raise syntax 93.901 array (least)
  if n > most then raise syntax 93.902 array (most)
  do i = 1 to n
    if \args~hasIndex(i) then raise syntax 93.903 array (i)
  end
  name = args[1]
  value = args[2]
  if kind == 'logical' then do
    if value == '1' | value == '0' then return
    raise syntax 88.927 array (name, value)
  end
  if kind == 'classType' then do
    class = args[3]
    if value~isA(class) == 1 then return
    raise syntax 88.914 array (name, class~id)
  end
  if kind == 'requestClassType' then do
    class = args[3]
    id = class~id
    answer = value~request(id)
    if answer~isA(class) == 1 then return answer
    raise syntax 93.978 array (name, id, answer)
  end
  /* The number checks. */
  if kind == 'numberRange' | kind == 'wholeNumberRange' then digitsAt = 5
  else digitsAt = 3
  if kind == 'position' | kind == 'length' then digits = .rexxinfo~internalDigits
  else digits = 9
  numeric digits 9
  if kind \== 'number' & args~hasIndex(digitsAt) then do
    given = args[digitsAt]
    text = self~!text(given)
    if text == .nil then raise syntax 26.5 additional (.array~of(given))
    if \text~datatype('W') then raise syntax 26.5 additional (.array~of(text))
    if text < 1 | text > 999999999 then raise syntax 26.5 additional (.array~of(text))
    digits = text
  end
  numeric digits digits
  op = ''
  whole = 0
  select
    when kind == 'number' then code = 88.902
    when kind == 'positiveNumber' then do
      code = 88.926; op = '>'
    end
    when kind == 'nonNegativeNumber' then do
      code = 88.925; op = '>='
    end
    when kind == 'wholeNumber' then do
      code = 88.903; whole = 1
    end
    when kind == 'positiveWholeNumber' then do
      code = 88.905; whole = 1; op = '>'
    end
    when kind == 'nonNegativeWholeNumber' then do
      code = 88.904; whole = 1; op = '>='
    end
    when kind == 'position' then do
      code = 88.912; whole = 1; op = '>'
    end
    when kind == 'length' then do
      code = 88.911; whole = 1; op = '>='
    end
    when kind == 'numberRange' then do
      code = 88.907; op = 'range'
    end
    otherwise do
      code = 88.907; whole = 1; op = 'range'
    end
  end
  if op == 'range' then extra = .array~of(name, args[3], args[4], value)
  else extra = .array~of(name, value)
  text = self~!text(value)
  ok = text \== .nil
  if ok then do
    if whole then ok = datatype(text, 'W')
    else ok = datatype(text, 'N')
  end
  if \ok then raise syntax (code) additional (extra)
  if op == '' then return
  count = 1
  if op == 'range' then do
    count = 2
    lowText = self~!text(args[3])
    highText = self~!text(args[4])
    if lowText == .nil | highText == .nil then raise syntax (code) additional (extra)
  end
  do i = 1 to count
    if op == '>' then answer = (value > 0)
    else if op == '>=' then answer = (value >= 0)
    else if i == 1 then answer = (value >= lowText)
    else answer = (value <= highText)
    if answer == 0 then raise syntax (code) additional (extra)
    if answer \== 1 then raise syntax 34.6 array (answer)
  end
  return

/* A value read as a string, r[conv.string.object]: an instance of String
   by its own characters, any other object by `request` with STRING and, when
   that does not answer a string, by `string`; `.nil` when neither does. */
::method '!text' class private
  use arg object
  if object == .nil then return .nil
  if object~isInstanceOf(.string) then return object~makeString:.string
  text = object~request('STRING')
  if text~isInstanceOf(.string) then return text~makeString:.string
  text = object~string
  if text~isInstanceOf(.string) then return text~makeString:.string
  return .nil
