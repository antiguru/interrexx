/*
 * Collection, MapCollection and the multi-item set operations of Relation.
 *
 * The rules are those of the class pages (collection.md, mapcollection.md,
 * relation.md). The messages each method sends, and their order, are part of
 * those rules, so every operation reads its argument itself: the library has
 * no shared helper that a program could not see.
 *
 * Internal routines below a method body are private to that method. `require`
 * checks that an object understands a message, which is what the pages call
 * 93.948 for an argument or an answer that lacks one (an UNKNOWN method counts
 * as understanding). `arrays` and `asarray` read an answer as the two
 * position-wise arrays of r[class.collection.operand.list].
 */

::class "Collection" public mixinclass Object

::method '[]' abstract
::method '[]=' abstract
::method allIndexes abstract
::method allItems abstract
::method at abstract
::method index abstract
::method put abstract

-- r[class.collection.hasindex], r[class.collection.hasindex.sends]
::method hasIndex
  use strict arg index
  return self~allIndexes~hasItem(index)

-- r[class.collection.hasitem], r[class.collection.hasitem.sends]
::method hasItem
  use strict arg item
  return self~allItems~hasItem(item)

-- r[class.collection.items], r[class.collection.items.sends]
::method items
  return self~allIndexes~items

-- r[class.collection.makearray], r[class.collection.makearray.sends]
::method makeArray
  return self~allItems

-- r[class.collection.supplier], r[class.collection.supplier.sends]
::method supplier
  return .Supplier~new(self~allItems, self~allIndexes)

-- r[class.collection.union], r[class.collection.union.sends]
::method union
  use strict arg other
  copy = self~copy
  checked = ''
  if other~isA(.OrderedCollection) then source = other~allItems
  else if other~isA(.MapCollection) then do
    supplier = other~supplier
    checked = require(supplier, 'AVAILABLE', checked)
    do while supplier~available
      checked = require(supplier, 'INDEX', checked)
      index = supplier~index
      has = copy~hasIndex(index)
      -- a logical value: a non-string is refused, and a string that is not 0 or 1 by `\`
      kind = has~class
      if \ (kind == .String | kind~isSubclassOf(.String)) then raise syntax 93.948 array(1, 'Collection')
      if \ has then do
        checked = require(supplier, 'ITEM', checked)
        item = supplier~item
        copy~put(item, index)
      end
      checked = require(supplier, 'NEXT', checked)
      supplier~next
    end
    return copy
  end
  else do
    if \ other~hasMethod('MAKEARRAY') then raise syntax 93.948 array(1, 'Collection')
    source = other~makeArray
  end
  pair = arrays(source)
  items = pair~at(1)
  indexes = pair~at(2)
  do i = 1 to items~size
    item = items~at(i)
    index = indexes~at(i)
    has = copy~hasIndex(index)
    kind = has~class
    if \ (kind == .String | kind~isSubclassOf(.String)) then raise syntax 93.948 array(1, 'Collection')
    if \ has then copy~put(item, index)
  end
  return copy

require: procedure
  use arg object, name, checked
  if wordpos(name, checked) = 0 then do
    if \ (object~hasMethod(name) | object~hasMethod('UNKNOWN')) then raise syntax 93.948 array(1, 'Collection')
    checked = checked name
  end
  return checked

arrays: procedure
  use arg source
  if source~class == .Array then return .Array~of(source, source)
  first = asarray(source)
  second = asarray(source)
  return .Array~of(first, second)

asarray: procedure
  use arg source
  if \ source~hasMethod('MAKEARRAY') then raise syntax 98.913 array('object')
  return source~makeArray

-- r[class.collection.difference], r[class.collection.difference.sends]
::method difference
  use strict arg other
  if other~isA(.OrderedCollection) then answer = other~allItems
  else if other~isA(.MapCollection) then answer = other~allIndexes
  else do
    if \ other~hasMethod('MAKEARRAY') then raise syntax 93.948 array(1, 'Collection')
    answer = other~makeArray
  end
  copy = self~copy
  checked = require(other, 'ITEMS', '')
  count = other~items
  if count == .nil then raise syntax 93.948 array(1, 'Collection')
  checked = ''
  do position = 1 to count
    checked = require(answer, '[]', checked)
    entry = answer[position]
    checked = require(copy, 'REMOVE', checked)
    copy~remove(entry)
  end
  return copy

require: procedure
  use arg object, name, checked
  if wordpos(name, checked) = 0 then do
    if \ (object~hasMethod(name) | object~hasMethod('UNKNOWN')) then raise syntax 93.948 array(1, 'Collection')
    checked = checked name
  end
  return checked

-- r[class.collection.intersection], r[class.collection.intersection.sends]
::method intersection
  use strict arg other
  made = self~class~new
  checked = ''
  if other~isA(.OrderedCollection) then source = other~allItems
  else if other~isA(.MapCollection) then do
    supplier = other~supplier
    checked = require(supplier, 'AVAILABLE', checked)
    do while supplier~available
      checked = require(supplier, 'INDEX', checked)
      index = supplier~index
      if self~hasIndex(index) then do
        checked = require(supplier, 'ITEM', checked)
        item = supplier~item
        made~put(item, index)
      end
      checked = require(supplier, 'NEXT', checked)
      supplier~next
    end
    return made
  end
  else do
    if \ other~hasMethod('MAKEARRAY') then raise syntax 93.948 array(1, 'Collection')
    source = other~makeArray
  end
  pair = arrays(source)
  items = pair~at(1)
  indexes = pair~at(2)
  do i = 1 to items~size
    item = items~at(i)
    index = indexes~at(i)
    if self~hasIndex(index) then made~put(item, index)
  end
  return made

require: procedure
  use arg object, name, checked
  if wordpos(name, checked) = 0 then do
    if \ (object~hasMethod(name) | object~hasMethod('UNKNOWN')) then raise syntax 93.948 array(1, 'Collection')
    checked = checked name
  end
  return checked

arrays: procedure
  use arg source
  if source~class == .Array then return .Array~of(source, source)
  first = asarray(source)
  second = asarray(source)
  return .Array~of(first, second)

asarray: procedure
  use arg source
  if \ source~hasMethod('MAKEARRAY') then raise syntax 98.913 array('object')
  return source~makeArray

-- r[class.collection.xor], r[class.collection.xor.sends]
::method xor
  use strict arg other
  copy = self~copy
  checked = ''
  if other~isA(.OrderedCollection) then source = other~allItems
  else if other~isA(.MapCollection) then do
    supplier = other~supplier
    checked = require(supplier, 'AVAILABLE', checked)
    do while supplier~available
      checked = require(supplier, 'INDEX', checked)
      index = supplier~index
      if copy~hasIndex(index) then do
        checked = require(copy, 'REMOVE', checked)
        copy~remove(index)
      end
      else do
        checked = require(supplier, 'ITEM', checked)
        item = supplier~item
        copy~put(item, index)
      end
      checked = require(supplier, 'NEXT', checked)
      supplier~next
    end
    return copy
  end
  else do
    if \ other~hasMethod('MAKEARRAY') then raise syntax 93.948 array(1, 'Collection')
    source = other~makeArray
  end
  pair = arrays(source)
  items = pair~at(1)
  indexes = pair~at(2)
  do i = 1 to items~size
    item = items~at(i)
    index = indexes~at(i)
    if copy~hasIndex(index) then do
      checked = require(copy, 'REMOVE', checked)
      copy~remove(index)
    end
    else copy~put(item, index)
  end
  return copy

require: procedure
  use arg object, name, checked
  if wordpos(name, checked) = 0 then do
    if \ (object~hasMethod(name) | object~hasMethod('UNKNOWN')) then raise syntax 93.948 array(1, 'Collection')
    checked = checked name
  end
  return checked

arrays: procedure
  use arg source
  if source~class == .Array then return .Array~of(source, source)
  first = asarray(source)
  second = asarray(source)
  return .Array~of(first, second)

asarray: procedure
  use arg source
  if \ source~hasMethod('MAKEARRAY') then raise syntax 98.913 array('object')
  return source~makeArray

-- r[class.collection.subset], r[class.collection.subset.sends],
-- r[class.collection.subset.receiver]
::method subset
  use strict arg other
  map = 0
  setlike = 0
  if other~isA(.OrderedCollection) then answer = other~allItems
  else if other~isA(.MapCollection) then do
    map = 1
    answer = other~allIndexes
    isSet = other~isA(.Set)
    kind = isSet~class
    if \ (kind == .String | kind~isSubclassOf(.String)) then raise syntax 93.948 array(1, 'Collection')
    if \ \ isSet then setlike = 1
  end
  else do
    if \ other~hasMethod('MAKEARRAY') then raise syntax 93.948 array(1, 'Collection')
    answer = other~makeArray
  end
  copy = self~copy
  checked = require(other, 'ITEMS', '')
  count = other~items
  receiver = self~makeArray
  if receiver~class \== .Array then raise syntax 98.913 array('object')
  checked = ''
  last = receiver~last
  if last == .nil then return 1
  do i = 1 to last
    item = receiver~at(i)
    checked = require(answer, 'HASITEM', checked)
    if \ answer~hasItem(item) then return 0
    if map & \ setlike then do
      mine = self[item]
      theirs = other[item]
      if mine \= theirs, 1 then return 0
    end
  end
  return 1

require: procedure
  use arg object, name, checked
  if wordpos(name, checked) = 0 then do
    if \ (object~hasMethod(name) | object~hasMethod('UNKNOWN')) then raise syntax 93.948 array(1, 'Collection')
    checked = checked name
  end
  return checked

-- r[class.collection.disjoint], r[class.collection.disjoint.sends]
::method disjoint
  use strict arg other
  return self~intersection(other)~isEmpty

-- r[class.collection.equivalent], r[class.collection.equivalent.sends]
::method equivalent
  use strict arg other
  mine = self~items
  theirs = other~items
  if mine = theirs, 1 then nop
  else return 0
  answer = self~subset(other)
  if answer, 1 then nop
  return answer


::class "MapCollection" public mixinclass Collection

-- r[class.mapcollection.of], r[class.mapcollection.of.sends],
-- r[class.mapcollection.of.error]
::method of class
  numeric digits 9
  made = self~new
  last = arg()
  do while last > 0
    if arg(last, 'E') then leave
    last = last - 1
  end
  do i = 1 to last
    if \ arg(i, 'E') then raise syntax 93.903 array(i)
    pair = arg(i)
    if \ pair~isA(.Array) then raise syntax 88.923 array(i)
    if \ pair~dimension then raise syntax 88.923 array(i)
    count = pair~items
    top = pair~last
    if count = 2 & top = 2 then nop
    else raise syntax 88.924 array(i)
    item = pair[2]
    index = pair[1]
    made~put(item, index)
  end
  return made

-- r[class.mapcollection.makearray], r[class.mapcollection.makearray.sends]
::method makeArray
  return self~allIndexes

-- r[class.mapcollection.putall], r[class.mapcollection.putall.sends],
-- r[class.mapcollection.putall.error]
::method putAll
  use strict arg source
  checked = require(source, 'SUPPLIER', '')
  supplier = source~supplier
  checked = require(supplier, 'AVAILABLE', checked)
  do while supplier~available
    checked = require(supplier, 'ITEM', checked)
    item = supplier~item
    checked = require(supplier, 'INDEX', checked)
    index = supplier~index
    self~put(item, index)
    checked = require(supplier, 'NEXT', checked)
    supplier~next
  end
  return self

require: procedure
  use arg object, name, checked
  if wordpos(name, checked) = 0 then do
    if \ (object~hasMethod(name) | object~hasMethod('UNKNOWN')) then raise syntax 93.948 array(1, 'Collection')
    checked = checked name
  end
  return checked


::class "ManyItemMixin"

-- r[class.relation.operand], r[class.relation.setops.sends]
::method union
  use strict arg other
  copy = self~copy
  checked = ''
  if other~isA(.OrderedCollection) then source = other~allItems
  else if other~isA(.MapCollection) then do
    supplier = other~supplier
    checked = require(supplier, 'AVAILABLE', checked)
    do while supplier~available
      checked = require(supplier, 'ITEM', checked)
      item = supplier~item
      checked = require(supplier, 'INDEX', checked)
      index = supplier~index
      checked = require(copy, 'PUT', checked)
      copy~put(item, index)
      checked = require(supplier, 'NEXT', checked)
      supplier~next
    end
    return copy
  end
  else do
    if \ other~hasMethod('MAKEARRAY') then raise syntax 93.948 array(1, 'Collection')
    source = other~makeArray
  end
  pair = arrays(source)
  items = pair~at(1)
  indexes = pair~at(2)
  do i = 1 to items~size
    item = items~at(i)
    index = indexes~at(i)
    checked = require(copy, 'PUT', checked)
    copy~put(item, index)
  end
  return copy
require: procedure
  use arg object, name, checked
  if wordpos(name, checked) = 0 then do
    if \ (object~hasMethod(name) | object~hasMethod('UNKNOWN')) then raise syntax 93.948 array(1, 'Collection')
    checked = checked name
  end
  return checked

arrays: procedure
  use arg source
  if source~class == .Array then return .Array~of(source, source)
  first = asarray(source)
  second = asarray(source)
  return .Array~of(first, second)

asarray: procedure
  use arg source
  if \ source~hasMethod('MAKEARRAY') then raise syntax 98.913 array('object')
  return source~makeArray


-- r[class.relation.operand], r[class.relation.setops.sends]
::method difference
  use strict arg other
  copy = self~copy
  checked = ''
  if other~isA(.OrderedCollection) then source = other~allItems
  else if other~isA(.MapCollection) then do
    supplier = other~supplier
    checked = require(supplier, 'AVAILABLE', checked)
    do while supplier~available
      checked = require(supplier, 'ITEM', checked)
      item = supplier~item
      checked = require(supplier, 'INDEX', checked)
      index = supplier~index
      checked = require(copy, 'REMOVEITEM', checked)
      copy~removeItem(item, index)
      checked = require(supplier, 'NEXT', checked)
      supplier~next
    end
    return copy
  end
  else do
    if \ other~hasMethod('MAKEARRAY') then raise syntax 93.948 array(1, 'Collection')
    source = other~makeArray
  end
  pair = arrays(source)
  items = pair~at(1)
  indexes = pair~at(2)
  do i = 1 to items~size
    item = items~at(i)
    index = indexes~at(i)
    checked = require(copy, 'REMOVEITEM', checked)
    copy~removeItem(item, index)
  end
  return copy
require: procedure
  use arg object, name, checked
  if wordpos(name, checked) = 0 then do
    if \ (object~hasMethod(name) | object~hasMethod('UNKNOWN')) then raise syntax 93.948 array(1, 'Collection')
    checked = checked name
  end
  return checked

arrays: procedure
  use arg source
  if source~class == .Array then return .Array~of(source, source)
  first = asarray(source)
  second = asarray(source)
  return .Array~of(first, second)

asarray: procedure
  use arg source
  if \ source~hasMethod('MAKEARRAY') then raise syntax 98.913 array('object')
  return source~makeArray


-- r[class.relation.operand], r[class.relation.setops.sends]
::method intersection
  use strict arg other
  made = self~class~new
  copy = self~copy
  checked = ''
  if other~isA(.OrderedCollection) then source = other~allItems
  else if other~isA(.MapCollection) then do
    supplier = other~supplier
    checked = require(supplier, 'AVAILABLE', checked)
    do while supplier~available
      checked = require(supplier, 'INDEX', checked)
      index = supplier~index
      checked = require(supplier, 'ITEM', checked)
      item = supplier~item
      checked = require(copy, 'HASITEM', checked)
      if copy~hasItem(item, index) then do
        checked = require(made, 'PUT', checked)
        made~put(item, index)
        checked = require(copy, 'REMOVEITEM', checked)
        copy~removeItem(item, index)
      end
      checked = require(supplier, 'NEXT', checked)
      supplier~next
    end
    return made
  end
  else do
    if \ other~hasMethod('MAKEARRAY') then raise syntax 93.948 array(1, 'Collection')
    source = other~makeArray
  end
  pair = arrays(source)
  items = pair~at(1)
  indexes = pair~at(2)
  do i = 1 to items~size
    item = items~at(i)
    index = indexes~at(i)
    checked = require(copy, 'HASITEM', checked)
    if copy~hasItem(item, index) then do
      checked = require(made, 'PUT', checked)
      made~put(item, index)
      checked = require(copy, 'REMOVEITEM', checked)
      copy~removeItem(item, index)
    end
  end
  return made
require: procedure
  use arg object, name, checked
  if wordpos(name, checked) = 0 then do
    if \ (object~hasMethod(name) | object~hasMethod('UNKNOWN')) then raise syntax 93.948 array(1, 'Collection')
    checked = checked name
  end
  return checked

arrays: procedure
  use arg source
  if source~class == .Array then return .Array~of(source, source)
  first = asarray(source)
  second = asarray(source)
  return .Array~of(first, second)

asarray: procedure
  use arg source
  if \ source~hasMethod('MAKEARRAY') then raise syntax 98.913 array('object')
  return source~makeArray


-- r[class.relation.operand], r[class.relation.setops.sends]
::method xor
  use strict arg other
  copy = self~copy
  made = self~class~new
  checked = ''
  if other~isA(.OrderedCollection) then source = other~allItems
  else if other~isA(.MapCollection) then do
    supplier = other~supplier
    checked = require(supplier, 'AVAILABLE', checked)
    do while supplier~available
      checked = require(supplier, 'INDEX', checked)
      index = supplier~index
      checked = require(supplier, 'ITEM', checked)
      item = supplier~item
      checked = require(copy, 'HASITEM', checked)
      if copy~hasItem(item, index) then do
        checked = require(copy, 'REMOVEITEM', checked)
        copy~removeItem(item, index)
      end
      else do
        checked = require(made, 'PUT', checked)
        made~put(item, index)
      end
      checked = require(supplier, 'NEXT', checked)
      supplier~next
    end
    checked = require(copy, 'PUTALL', checked)
    copy~putAll(made)
    return copy
  end
  else do
    if \ other~hasMethod('MAKEARRAY') then raise syntax 93.948 array(1, 'Collection')
    source = other~makeArray
  end
  pair = arrays(source)
  items = pair~at(1)
  indexes = pair~at(2)
  do i = 1 to items~size
    item = items~at(i)
    index = indexes~at(i)
    checked = require(copy, 'HASITEM', checked)
    if copy~hasItem(item, index) then do
      checked = require(copy, 'REMOVEITEM', checked)
      copy~removeItem(item, index)
    end
    else do
      checked = require(made, 'PUT', checked)
      made~put(item, index)
    end
  end
  checked = require(copy, 'PUTALL', checked)
  copy~putAll(made)
  return copy
require: procedure
  use arg object, name, checked
  if wordpos(name, checked) = 0 then do
    if \ (object~hasMethod(name) | object~hasMethod('UNKNOWN')) then raise syntax 93.948 array(1, 'Collection')
    checked = checked name
  end
  return checked

arrays: procedure
  use arg source
  if source~class == .Array then return .Array~of(source, source)
  first = asarray(source)
  second = asarray(source)
  return .Array~of(first, second)

asarray: procedure
  use arg source
  if \ source~hasMethod('MAKEARRAY') then raise syntax 98.913 array('object')
  return source~makeArray


-- r[class.relation.operand], r[class.relation.setops.sends]
::method subset
  use strict arg other
  copy = self~copy
  checked = ''
  if other~isA(.OrderedCollection) then source = other~allItems
  else if other~isA(.MapCollection) then do
    supplier = other~supplier
    checked = require(supplier, 'AVAILABLE', checked)
    do while supplier~available
      checked = require(supplier, 'ITEM', checked)
      item = supplier~item
      checked = require(supplier, 'INDEX', checked)
      index = supplier~index
      checked = require(copy, 'REMOVEITEM', checked)
      copy~removeItem(item, index)
      checked = require(supplier, 'NEXT', checked)
      supplier~next
    end
    checked = require(copy, 'ISEMPTY', checked)
    return copy~isEmpty
  end
  else do
    if \ other~hasMethod('MAKEARRAY') then raise syntax 93.948 array(1, 'Collection')
    source = other~makeArray
  end
  pair = arrays(source)
  items = pair~at(1)
  indexes = pair~at(2)
  do i = 1 to items~size
    item = items~at(i)
    index = indexes~at(i)
    checked = require(copy, 'REMOVEITEM', checked)
    copy~removeItem(item, index)
  end
  checked = require(copy, 'ISEMPTY', checked)
  return copy~isEmpty
require: procedure
  use arg object, name, checked
  if wordpos(name, checked) = 0 then do
    if \ (object~hasMethod(name) | object~hasMethod('UNKNOWN')) then raise syntax 93.948 array(1, 'Collection')
    checked = checked name
  end
  return checked

arrays: procedure
  use arg source
  if source~class == .Array then return .Array~of(source, source)
  first = asarray(source)
  second = asarray(source)
  return .Array~of(first, second)

asarray: procedure
  use arg source
  if \ source~hasMethod('MAKEARRAY') then raise syntax 98.913 array('object')
  return source~makeArray

