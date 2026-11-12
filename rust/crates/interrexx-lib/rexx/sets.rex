/* SetMixin and BagMixin: the set operations that Set and Bag take from them.
   Neither class is public. See spec/src/classes/set.md and bag.md, rules
   class.set.operand, class.set.operand.list, class.bag.operand and
   class.bag.operand.list; Deviation 125 is the List case of the walk. */

::class SetMixin

/* Sends the message NAME to OBJ, with ARG as its argument when one is given.
   An object that does not understand it raises SYNTAX 93.948, the error of an
   argument that is not a collection (r:class.set.operand,
   r:class.set.operand.list). */
::method interrexx_ask private
  use arg obj, name
  signal on nomethod name missing
  if arg() > 2 then return obj~send(name, arg(3))
  return obj~send(name)
missing:
  raise syntax 93.948 array (1, 'Collection')

/* Visits the members of ANSWER, the answer of allItems or makeArray, in the
   order of r:class.set.operand.list, by sending STEP to the receiver with
   each member and the two objects A and B. A List answer gives all its
   items; any other answer is read by items and then [] for the positions 1
   up to the number items answered. */
::method interrexx_walk private
  use arg answer, step, a, b
  if answer~isA(.list) then do
    do member over answer
      self~send(step, member, a, b)
    end
    return
  end
  n = self~interrexx_ask(answer, 'items')
  do i = 1 to n
    self~send(step, self~interrexx_ask(answer, '[]', i), a, b)
  end

/* The answer of allItems or makeArray of OTHER, chosen by what isA with the
   class Collection answers (r:class.set.operand). */
::method interrexx_read private
  use arg other
  kind = other~isA(.collection)
  if kind == 1 then return self~interrexx_ask(other, 'allItems')
  if kind == 0 then return self~interrexx_ask(other, 'makeArray')
  raise syntax 34.1 array (kind)

/* r[class.set.putall] */
::method putAll
  if arg() = 0 then raise syntax 93.901 array (1)
  if arg() > 1 then raise syntax 93.902 array (1)
  self~interrexx_walk(self~interrexx_read(arg(1)), 'interrexx_put')
  return self

::method interrexx_put private
  use arg member
  self~put(member)

/* r[class.set.union] */
::method union
  if arg() = 0 then raise syntax 93.901 array (1)
  if arg() > 1 then raise syntax 93.902 array (1)
  copy = self~copy
  self~interrexx_walk(self~interrexx_read(arg(1)), 'interrexx_union', copy)
  return copy

::method interrexx_union private
  use arg member, copy
  if \copy~hasIndex(member) then copy~put(member)

/* r[class.set.intersection] */
::method intersection
  if arg() = 0 then raise syntax 93.901 array (1)
  if arg() > 1 then raise syntax 93.902 array (1)
  new = self~class~new
  self~interrexx_walk(self~interrexx_read(arg(1)), 'interrexx_intersection', new)
  return new

::method interrexx_intersection private
  use arg member, new
  if self~hasIndex(member) then new~put(member)

/* r[class.set.xor] */
::method xor
  if arg() = 0 then raise syntax 93.901 array (1)
  if arg() > 1 then raise syntax 93.902 array (1)
  copy = self~copy
  self~interrexx_walk(self~interrexx_read(arg(1)), 'interrexx_xor', copy)
  return copy

::method interrexx_xor private
  use arg member, copy
  if self~hasIndex(member) then copy~remove(member)
  else copy~put(member)

/* r[class.set.subset]: allItems is sent to the argument whatever it is. */
::method subset
  if arg() = 0 then raise syntax 93.901 array (1)
  if arg() > 1 then raise syntax 93.902 array (1)
  copy = self~copy
  self~interrexx_walk(self~interrexx_ask(arg(1), 'allItems'), 'interrexx_subset', copy)
  return copy~isEmpty

::method interrexx_subset private
  use arg member, copy
  copy~remove(member)

::class BagMixin

/* Sends the message NAME to OBJ, with ARG as its argument when one is given.
   An object that does not understand it raises SYNTAX 93.948, the error of an
   argument that is not a collection (r:class.bag.operand,
   r:class.bag.operand.list). */
::method interrexx_ask private
  use arg obj, name
  signal on nomethod name missing
  if arg() > 2 then return obj~send(name, arg(3))
  return obj~send(name)
missing:
  raise syntax 93.948 array (1, 'Collection')

/* Visits the members of ANSWER, the answer of allItems or makeArray, in the
   order of r:class.bag.operand.list, by sending STEP to the receiver with
   each member and the two objects A and B. A List answer gives all its
   items; any other answer is read by items and then [] for the positions 1
   up to the number items answered. */
::method interrexx_walk private
  use arg answer, step, a, b
  if answer~isA(.list) then do
    do member over answer
      self~send(step, member, a, b)
    end
    return
  end
  n = self~interrexx_ask(answer, 'items')
  do i = 1 to n
    self~send(step, self~interrexx_ask(answer, '[]', i), a, b)
  end

/* The answer of allItems or makeArray of OTHER, chosen by what isA with the
   class Collection answers (r:class.bag.operand). */
::method interrexx_read private
  use arg other
  kind = other~isA(.collection)
  if kind == 1 then return self~interrexx_ask(other, 'allItems')
  if kind == 0 then return self~interrexx_ask(other, 'makeArray')
  raise syntax 34.1 array (kind)

/* r[class.bag.putall] */
::method putAll
  if arg() = 0 then raise syntax 93.901 array (1)
  if arg() > 1 then raise syntax 93.902 array (1)
  self~interrexx_walk(self~interrexx_read(arg(1)), 'interrexx_put')
  return self

::method interrexx_put private
  use arg member
  self~put(member)

/* r[class.bag.union] */
::method union
  if arg() = 0 then raise syntax 93.901 array (1)
  if arg() > 1 then raise syntax 93.902 array (1)
  copy = self~copy
  self~interrexx_walk(self~interrexx_read(arg(1)), 'interrexx_union', copy)
  return copy

::method interrexx_union private
  use arg member, copy
  copy~put(member)

/* r[class.bag.difference] */
::method difference
  if arg() = 0 then raise syntax 93.901 array (1)
  if arg() > 1 then raise syntax 93.902 array (1)
  copy = self~copy
  self~interrexx_walk(self~interrexx_read(arg(1)), 'interrexx_difference', copy)
  return copy

::method interrexx_difference private
  use arg member, copy
  copy~remove(member)

/* r[class.bag.intersection] */
::method intersection
  if arg() = 0 then raise syntax 93.901 array (1)
  if arg() > 1 then raise syntax 93.902 array (1)
  new = self~class~new
  copy = self~copy
  self~interrexx_walk(self~interrexx_read(arg(1)), 'interrexx_intersection', copy, new)
  return new

::method interrexx_intersection private
  use arg member, copy, new
  if copy~hasIndex(member) then do
    new~put(member)
    copy~remove(member)
  end

/* r[class.bag.xor] */
::method xor
  if arg() = 0 then raise syntax 93.901 array (1)
  if arg() > 1 then raise syntax 93.902 array (1)
  copy = self~copy
  new = self~class~new
  self~interrexx_walk(self~interrexx_read(arg(1)), 'interrexx_xor', copy, new)
  copy~putAll(new)
  return copy

::method interrexx_xor private
  use arg member, copy, new
  if copy~hasIndex(member) then copy~remove(member)
  else new~put(member)

/* r[class.bag.subset] */
::method subset
  if arg() = 0 then raise syntax 93.901 array (1)
  if arg() > 1 then raise syntax 93.902 array (1)
  copy = self~copy
  self~interrexx_walk(self~interrexx_read(arg(1)), 'interrexx_subset', copy)
  return copy~isEmpty

::method interrexx_subset private
  use arg member, copy
  copy~remove(member)
