/*
 * OrderedCollection: the mixin of the collections whose entries have an order
 * of their own (Array, List, Queue, CircularQueue).
 *
 * The rules are those of the class page (orderedcollection.md). The messages
 * each method sends, and their order, are part of those rules, so every
 * operation reads its argument itself and the library has no shared helper
 * that a program could see. `understands` checks that an object understands a
 * message, which is what the page calls 93.948 for an argument or an answer
 * that lacks one (an UNKNOWN method counts as understanding). Its labels are
 * private to the method body they follow.
 */

::class "OrderedCollection" public mixinclass Collection

::method append abstract
::method delete abstract
::method first abstract
::method firstItem abstract
::method insert abstract
::method last abstract
::method lastItem abstract
::method next abstract
::method previous abstract
::method section abstract

-- r[class.orderedcollection.sort]
::method sort
  use strict arg
  return self~stableSort

-- r[class.orderedcollection.sort]
::method sortWith
  use strict arg comparator
  return self~stableSortWith(comparator)

-- r[class.orderedcollection.sort]
::method stableSort
  use strict arg
  sorted = self~makeArray
  sorted~sort
  index = self~first
  count = sorted~size
  do place = 1 to count
    self~'[]='(sorted~'[]'(place), index)
    index = self~next(index)
  end
  return self

-- r[class.orderedcollection.sort]
::method stableSortWith
  use strict arg comparator
  sorted = self~makeArray
  sorted~sortWith(comparator)
  index = self~first
  count = sorted~size
  do place = 1 to count
    self~'[]='(sorted~'[]'(place), index)
    index = self~next(index)
  end
  return self

-- r[class.orderedcollection.appendall], r[class.orderedcollection.appendall.sends]
::method appendAll
  use strict arg source
  if \ understands(source, 'ALLITEMS') then raise syntax 93.948 array(1, 'Collection')
  do item over source~allItems
    self~append(item)
  end
  return self

understands: procedure
  use arg object, name
  return object~hasMethod(name) | object~hasMethod('UNKNOWN')

-- r[class.orderedcollection.union], r[class.orderedcollection.union.sends]
-- r[class.orderedcollection.operand], r[class.orderedcollection.operand.isa]
::method union
  use strict arg other
  copy = self~copy
  isa = other~isA(.Collection)
  if \ (isa~class == .String | isa~class~isSubclassOf(.String)) then raise syntax 93.948 array(1, 'Collection')
  if isa == 1 then source = other
  else if isa == 0 then do
    if \ understands(other, 'MAKEARRAY') then raise syntax 93.948 array(1, 'Collection')
    source = other~makeArray
  end
  else raise syntax 34.901 array(isa)
  if \ understands(source, 'ALLITEMS') then raise syntax 93.948 array(1, 'Collection')
  do item over source~allItems
    copy~append(item)
  end
  if copy~isA(.Array) then return copy~makeArray
  return copy

understands: procedure
  use arg object, name
  return object~hasMethod(name) | object~hasMethod('UNKNOWN')

-- r[class.orderedcollection.difference], r[class.orderedcollection.difference.sends]
::method difference
  use strict arg other
  copy = self~copy
  isa = other~isA(.Collection)
  if \ (isa~class == .String | isa~class~isSubclassOf(.String)) then raise syntax 93.948 array(1, 'Collection')
  if isa == 1 then source = other
  else if isa == 0 then do
    if \ understands(other, 'MAKEARRAY') then raise syntax 93.948 array(1, 'Collection')
    source = other~makeArray
  end
  else raise syntax 34.901 array(isa)
  if \ understands(source, 'ALLITEMS') then raise syntax 93.948 array(1, 'Collection')
  checked = 0
  do item over source~allItems
    if \ checked then do
      if \ understands(copy, 'REMOVEITEM') then raise syntax 93.948 array(1, 'Collection')
      checked = 1
    end
    copy~removeItem(item)
  end
  if copy~isA(.Array) then return copy~makeArray
  return copy

understands: procedure
  use arg object, name
  return object~hasMethod(name) | object~hasMethod('UNKNOWN')

-- r[class.orderedcollection.intersection], r[class.orderedcollection.intersection.sends]
::method intersection
  use strict arg other
  made = self~class~new
  work = self~copy
  isa = other~isA(.Collection)
  if \ (isa~class == .String | isa~class~isSubclassOf(.String)) then raise syntax 93.948 array(1, 'Collection')
  if isa == 1 then source = other
  else if isa == 0 then do
    if \ understands(other, 'MAKEARRAY') then raise syntax 93.948 array(1, 'Collection')
    source = other~makeArray
  end
  else raise syntax 34.901 array(isa)
  if \ understands(source, 'ALLITEMS') then raise syntax 93.948 array(1, 'Collection')
  checked = 0
  do item over source~allItems
    if work~hasItem(item) then do
      made~append(item)
      if \ checked then do
        if \ understands(work, 'REMOVEITEM') then raise syntax 93.948 array(1, 'Collection')
        checked = 1
      end
      work~removeItem(item)
    end
  end
  return made

understands: procedure
  use arg object, name
  return object~hasMethod(name) | object~hasMethod('UNKNOWN')

-- r[class.orderedcollection.xor], r[class.orderedcollection.xor.sends]
::method xor
  use strict arg other
  made = self~copy
  work = self~copy
  isa = other~isA(.Collection)
  if \ (isa~class == .String | isa~class~isSubclassOf(.String)) then raise syntax 93.948 array(1, 'Collection')
  if isa == 1 then source = other
  else if isa == 0 then do
    if \ understands(other, 'MAKEARRAY') then raise syntax 93.948 array(1, 'Collection')
    source = other~makeArray
  end
  else raise syntax 34.901 array(isa)
  if \ understands(source, 'ALLITEMS') then raise syntax 93.948 array(1, 'Collection')
  checked = 0
  do item over source~allItems
    if work~hasItem(item) then do
      if \ checked then do
        if \ understands(work, 'REMOVEITEM') then raise syntax 93.948 array(1, 'Collection')
        checked = 1
      end
      made~removeItem(item)
      work~removeItem(item)
    end
    else made~append(item)
  end
  if made~isA(.Array) then return made~makeArray
  return made

understands: procedure
  use arg object, name
  return object~hasMethod(name) | object~hasMethod('UNKNOWN')

-- r[class.orderedcollection.subset], r[class.orderedcollection.subset.sends]
::method subset
  use strict arg other
  isa = other~isA(.Collection)
  if \ (isa~class == .String | isa~class~isSubclassOf(.String)) then raise syntax 93.948 array(1, 'Collection')
  if isa == 1 then source = other
  else if isa == 0 then do
    if \ understands(other, 'MAKEARRAY') then raise syntax 93.948 array(1, 'Collection')
    source = other~makeArray
  end
  else raise syntax 34.901 array(isa)
  copy = self~copy
  if \ understands(source, 'ALLITEMS') then raise syntax 93.948 array(1, 'Collection')
  checked = 0
  do item over source~allItems
    if \ checked then do
      if \ understands(copy, 'REMOVEITEM') then raise syntax 93.948 array(1, 'Collection')
      checked = 1
    end
    copy~removeItem(item)
  end
  return 0 = copy~items

understands: procedure
  use arg object, name
  return object~hasMethod(name) | object~hasMethod('UNKNOWN')
