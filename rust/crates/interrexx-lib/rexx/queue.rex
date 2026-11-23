/* RexxQueue and CircularQueue. Spec rules: class.rexxqueue.*,
   class.circularqueue.*. */

::class "RexxQueue" public

/* r[class.rexxqueue.init] and r[class.rexxqueue.init.sends]: the name is
   kept in NAMED_QUEUE, which the bound methods below read. */
::method init
  expose named_queue
  if arg() > 1 then raise syntax 93.902 array (1)
  if arg(1, 'O') then name = 'SESSION'
  else if .nil == arg(1) then do
    name = self~class~create~upper
    self~class~open(name)
  end
  else do
    name = arg(1)~upper
    self~class~open(name)
  end
  named_queue = name
  self~objectName = name

/* r[class.rexxqueue.get] */
::method get unguarded
  expose named_queue
  if arg() > 0 then raise syntax 93.902 array (0)
  if \var('NAMED_QUEUE') then raise syntax 98.901 array (self~string)
  return named_queue

/* r[class.rexxqueue.set] and r[class.rexxqueue.set.sends] */
::method set
  expose named_queue
  if arg() > 1 then raise syntax 93.902 array (1)
  if arg(1, 'O') then raise syntax 93.901 array (1)
  if \var('NAMED_QUEUE') then raise syntax 98.901 array (self~string)
  name = arg(1)~upper
  old = named_queue
  named_queue = name
  self~objectName = name
  return old

/* r[class.rexxqueue.delete] and r[class.rexxqueue.delete.sends] */
::method delete
  expose named_queue
  if arg() > 0 then raise syntax 93.902 array (0)
  if \var('NAMED_QUEUE') then raise syntax 98.901 array (self~string)
  return self~class~delete(named_queue)

/* r[class.rexxqueue.lineout] and r[class.rexxqueue.lineout.sends] */
::method lineOut
  forward message 'QUEUE'

/* r[class.rexxqueue.say] and r[class.rexxqueue.say.sends] */
::method say
  forward message 'QUEUE'

/* r[class.rexxqueue.makearray] and r[class.rexxqueue.makearray.sends] */
::method makeArray
  if arg() > 0 then raise syntax 93.902 array (0)
  lines = .array~new
  count = self~queued
  do count
    line = self~pull
    if .nil == line then leave
    lines~append(line)
  end
  return lines

::method empty external 'LIBRARY REXX rexx_clear_queue'
::method lineIn external 'LIBRARY REXX rexx_linein_queue'
::method pull external 'LIBRARY REXX rexx_pull_queue'
::method push external 'LIBRARY REXX rexx_push_queue'
::method queue external 'LIBRARY REXX rexx_queue_queue'
::method queued external 'LIBRARY REXX rexx_query_queue'

::method create class external 'LIBRARY REXX rexx_create_queue'
::method delete class external 'LIBRARY REXX rexx_delete_queue'
::method exists class external 'LIBRARY REXX rexx_queue_exists'
::method open class external 'LIBRARY REXX rexx_open_queue'


::class "CircularQueue" public subclass Queue

/* r[class.circularqueue.of] */
::method of class
  count = arg()
  queue = self~new(count)
  do i = 1 to count
    if arg(i, 'E') then queue~queue(arg(i))
    else queue~queue('')
  end
  return queue

/* r[class.circularqueue.new] and r[class.circularqueue.new.sends] */
::method init
  expose limit givenLimit
  if arg() > 1 then raise syntax 93.902 array (1)
  if arg(1, 'O') then raise syntax 93.901 array (1)
  limit = self~interrexx_size(arg(1), 1)
  givenLimit = arg(1)
  self~init:super

/* r[class.circularqueue.size] */
::method size
  expose givenLimit
  if arg() > 0 then raise syntax 93.902 array (0)
  return givenLimit

/* r[class.circularqueue.deviation.183]: numeric sizes also use the STRING
   fallback. Return the checked text so callers can retain its spelling. */
::method interrexx_size private
  use arg value, position
  numeric digits 40
  text = self~interrexx_text(value, 1)
  if .nil \== text then do
    if datatype(text, 'W') & text >= 0 & text <= 999999999 then return text
  end
  raise syntax 93.906 array (position, text)

/* r[class.index.string]: operate on a String subclass's own bytes, even if
   it overrides the type inquiry or makeString. */
::method interrexx_text private
  use arg value, numericArgument = 0
  if value~isInstanceOf:.Object(.string) then return value~makeString:.string
  text = value~request('STRING')
  if numericArgument & \text~isInstanceOf:.Object(.string) then text = value~string
  if text~isInstanceOf:.Object(.string) then return text~makeString:.string
  return .nil

/* r[class.circularqueue.order]: keep the order spelling for string's
   makeArray send. */
::method interrexx_order private
  use arg value, position
  order = self~interrexx_text(value)
  if .nil == order then raise syntax 88.909 array (position)
  if order~caselessStartsWith('F') | order~caselessStartsWith('L') then return order
  raise syntax 93.914 array (2, 'FL', order)

/* r[class.circularqueue.queue]: only a leading U makes the option Unique. */
::method interrexx_unique private
  text = self~interrexx_text(arg(1))
  if .nil == text then raise syntax 88.909 array (2)
  return text~caselessMatchChar(1, 'u')

/* r[class.circularqueue.queue] and r[class.circularqueue.queue.sends] */
::method queue
  expose limit
  if arg() > 2 then raise syntax 93.902 array (2)
  if arg() == 0 then raise syntax 93.901 array (1)
  if arg(1, 'O') then raise syntax 93.903 array (1)
  item = arg(1)
  unique = 0
  if arg(2, 'E') then unique = self~interrexx_unique(arg(2))
  if 1 > limit then return .nil
  if unique then do
    do while self~hasItem(item)
      self~removeItem(item)
    end
  end
  taken = .nil
  if self~items >= limit then taken = self~pull
  self~queue:super(item)
  return taken

::method push
  expose limit
  if arg() > 2 then raise syntax 93.902 array (2)
  if arg() == 0 then raise syntax 93.901 array (1)
  if arg(1, 'O') then raise syntax 93.903 array (1)
  item = arg(1)
  unique = 0
  if arg(2, 'E') then unique = self~interrexx_unique(arg(2))
  if 1 > limit then return .nil
  if unique then do
    do while self~hasItem(item)
      self~removeItem(item)
    end
  end
  taken = .nil
  if self~items >= limit then taken = self~remove(self~items)
  self~push:super(item)
  return taken

/* r[class.circularqueue.append] */
::method append
  expose limit
  if arg() > 1 then raise syntax 93.902 array (1)
  if arg() == 0 then raise syntax 93.901 array (1)
  if 1 > limit then return .nil
  if self~items >= limit then self~pull
  return self~append:super(arg(1))

/* r[class.circularqueue.insert] */
::method insert
  expose limit
  if arg() > 2 then raise syntax 93.902 array (2)
  forward class (super) continue
  index = result
  if self~items > limit then self~remove(self~last)
  return index

/* r[class.circularqueue.resize] and r[class.circularqueue.resize.sends] */
::method resize
  expose limit givenLimit
  if arg() > 2 then raise syntax 93.902 array (2)
  if arg() == 0 then raise syntax 93.901 array (1)
  if arg(1, 'O') then raise syntax 93.903 array (1)
  new = self~interrexx_size(arg(1), 1)
  order = 'F'
  if arg(2, 'E') then order = self~interrexx_order(arg(2), 2)
  fifo = order~caselessStartsWith('F')
  count = self~items
  if count > new then do
    count = self~items
    do count - new
      if fifo then self~remove(1)
      else self~remove(new + 1)
    end
  end
  limit = new
  givenLimit = arg(1)
  return givenLimit

/* r[class.circularqueue.order] and r[class.circularqueue.order.sends] */
::method makeArray
  if arg() > 1 then raise syntax 93.902 array (1)
  order = 'F'
  if arg(1, 'E') then order = self~interrexx_order(arg(1), 1)
  if order~caselessStartsWith('F') then return self~makeArray:super
  copy = self~copy
  count = copy~items
  lines = .array~new
  do i = count to 1 by -1
    lines~append(copy~at(i))
  end
  return lines

::method supplier
  if arg() > 1 then raise syntax 93.902 array (1)
  order = 'F'
  if arg(1, 'E') then order = self~interrexx_order(arg(1), 1)
  if order~caselessStartsWith('F') then return self~supplier:super
  copy = self~copy
  count = copy~items
  lines = .array~new
  indexes = .array~new
  do i = count to 1 by -1
    lines~append(copy~at(i))
    indexes~append(i)
  end
  return .supplier~new(lines, indexes)

/* r[class.circularqueue.string] and r[class.circularqueue.string.sends] */
::method string
  if arg() > 2 then raise syntax 93.902 array (2)
  order = 'F'
  if arg(2, 'E') then order = self~interrexx_order(arg(2), 2)
  lines = self~makeArray(order)
  if arg(1, 'E') then delimiter = arg(1)
  else delimiter = ','
  return lines~makeString('L', delimiter)

::method makeString
  forward message 'STRING'

/* r[class.circularqueue.section] and r[class.circularqueue.section.sends] */
::method section
  expose limit
  if arg() > 2 then raise syntax 93.902 array (2)
  if arg() == 0 then raise syntax 93.901 array (1)
  if arg(1, 'O') then raise syntax 93.903 array (1)
  numeric digits 40
  size = limit + 0
  count = size
  if arg(2, 'E') then do
    text = self~interrexx_size(arg(2), 2)
    count = text + 0
    if count > size then count = size
  end
  text = '' || arg(1)
  if \datatype(text, 'W') then raise syntax 93.907 array (1, text)
  if text < 1 then raise syntax 93.907 array (1, text)
  position = text + 0
  made = self~class~new(count)
  do count
    made~queue(self~'[]'(position))
    position = position + 1
    if position > size then position = 1
  end
  return made

/* The list a set operation reads its argument as: the argument itself when
   it answers 1 to isA with Collection, and else what its makeArray answers
   (r[class.orderedcollection.operand]). */
::method interrexx_operand private
  other = arg(1)
  answer = other~isA(.collection)
  if \answer~isInstanceOf(.string) then raise syntax 93.948 array (1)
  if answer == 1 then return other
  if answer \== 0 then raise syntax 34.901 array (answer)
  signal on syntax name notlist
  list = other~makeArray
  signal off syntax
  return list
notlist:
  if condition('O')~code == 97.1 then raise syntax 93.948 array (1)
  raise propagate

/* Sends a message with no argument to a list, an object that does not
   understand it being a 93.948 (the same rule). */
::method interrexx_ask private
  signal on syntax name notlist
  return arg(1)~send(arg(2))
notlist:
  if condition('O')~code == 97.1 then raise syntax 93.948 array (1)
  raise propagate

/* The items of a list as an array, as DO OVER reads them. */
::method interrexx_items private
  items = .array~new
  do item over self~interrexx_ask(arg(1), 'ALLITEMS')
    items~append(item)
  end
  return items

/* r[class.circularqueue.set] and r[class.circularqueue.set.sends] */
::method union
  expose limit
  if arg() > 1 then raise syntax 93.902 array (1)
  if arg() == 0 then raise syntax 93.901 array (1)
  answer = self~copy
  list = self~interrexx_operand(arg(1))
  sum = self~items + self~interrexx_ask(list, 'ITEMS')
  if sum > limit then do
    sum = self~items + self~interrexx_ask(list, 'ITEMS')
    answer~resize(sum)
  end
  do item over self~interrexx_items(list)
    answer~append(item)
  end
  return answer

::method intersection
  expose limit
  if arg() > 1 then raise syntax 93.902 array (1)
  if arg() == 0 then raise syntax 93.901 array (1)
  work = self~copy
  list = self~interrexx_operand(arg(1))
  answer = self~class~new(limit)
  do item over self~interrexx_items(list)
    if work~hasItem(item) then do
      answer~append(item)
      work~removeItem(item)
    end
  end
  return answer

::method xor
  expose limit
  if arg() > 1 then raise syntax 93.902 array (1)
  if arg() == 0 then raise syntax 93.901 array (1)
  left = self~allItems
  work = self~copy
  list = self~interrexx_operand(arg(1))
  found = .array~new
  do item over self~interrexx_items(list)
    if work~hasItem(item) then do
      work~removeItem(item)
      left~removeItem(item)
    end
    else found~append(item)
  end
  size = limit
  if left~items + found~items > size then size = left~items + found~items
  answer = self~class~new(size)
  do item over left
    answer~append(item)
  end
  do item over found
    answer~append(item)
  end
  return answer
