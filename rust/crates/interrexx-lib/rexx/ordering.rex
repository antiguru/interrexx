/* Ordering classes. Spec rules: class.orderable.*, class.comparable.*,
   class.comparators.*. */

::class "Orderable" public mixinclass Object

-- r[class.orderable.compareto]: the default order is that of identityHash.
::method compareTo
  use strict arg other
  numeric digits 18
  mine = self~identityHash
  theirs = other~identityHash
  mine = mine + 0
  theirs = theirs + 0
  if mine < theirs then return -1
  if mine > theirs then return 1
  return 0

::method '='
  use strict arg other
  if .nil == other then return 0
  return self~compareTo(other)~'='(0)

::method '=='
  use strict arg other
  if .nil == other then return 0
  return self~compareTo(other)~'='(0)

::method '<'
  use strict arg other
  if .nil == other then raise syntax 93.973 array(1)
  return self~compareTo(other)~'<'(0)

::method '<<'
  use strict arg other
  if .nil == other then raise syntax 93.973 array(1)
  return self~compareTo(other)~'<'(0)

::method '>'
  use strict arg other
  if .nil == other then raise syntax 93.973 array(1)
  return self~compareTo(other)~'>'(0)

::method '>>'
  use strict arg other
  if .nil == other then raise syntax 93.973 array(1)
  return self~compareTo(other)~'>'(0)

::method '<='
  use strict arg other
  if .nil == other then raise syntax 93.973 array(1)
  return self~compareTo(other)~'<='(0)

::method '<<='
  use strict arg other
  if .nil == other then raise syntax 93.973 array(1)
  return self~compareTo(other)~'<='(0)

::method '>='
  use strict arg other
  if .nil == other then raise syntax 93.973 array(1)
  return self~compareTo(other)~'>='(0)

::method '>>='
  use strict arg other
  if .nil == other then raise syntax 93.973 array(1)
  return self~compareTo(other)~'>='(0)

::method '\='
  use strict arg other
  if .nil == other then return 1
  return self~compareTo(other)~'\='(0)

::method '\=='
  use strict arg other
  if .nil == other then return 1
  return self~compareTo(other)~'\='(0)

::method '<>'
  use strict arg other
  return self~'\='(other)

::method '><'
  use strict arg other
  return self~'\='(other)

::method '\<'
  use strict arg other
  return self~'>='(other)

::method '\>'
  use strict arg other
  return self~'<='(other)

::method '\<<'
  use strict arg other
  return self~'>>='(other)

::method '\>>'
  use strict arg other
  return self~'<<='(other)


::class "Comparable" public mixinclass Object

-- r[class.comparable.compareto]
::method compareTo
  use strict arg other
  numeric digits 18
  mine = self~identityHash
  theirs = other~identityHash
  mine = mine + 0
  theirs = theirs + 0
  if mine < theirs then return -1
  if mine > theirs then return 1
  return 0

-- r[class.comparators.comparator.compare]
::class "Comparator" public mixinclass Object
::method compare
  use strict arg first, second
  return first~compareTo(second)

-- r[class.comparators.caseless.compare]
::class "CaselessComparator" public mixinclass Comparator
::method compare
  use strict arg first, second
  return first~caselessCompareTo(second)

-- r[class.comparators.descending.compare]
::class "DescendingComparator" public mixinclass Comparator
::method compare
  use strict arg first, second
  answer = first~compareTo(second)
  return -answer

-- r[class.comparators.caselessdescending.compare]
::class "CaselessDescendingComparator" public mixinclass Comparator
::method compare
  use strict arg first, second
  answer = first~caselessCompareTo(second)
  return -answer

-- r[class.comparators.column.init], r[class.comparators.column.compare]
::class "ColumnComparator" public mixinclass Comparator
::method init
  expose start length
  use strict arg start, length
::method compare
  expose start length
  use strict arg first, second
  return first~compareTo(second, start, length)

-- r[class.comparators.caselesscolumn.init],
-- r[class.comparators.caselesscolumn.compare]
::class "CaselessColumnComparator" public mixinclass Comparator
::method init
  expose start length
  use strict arg start, length
::method compare
  expose start length
  use strict arg first, second
  return first~caselessCompareTo(second, start, length)

-- r[class.comparators.inverting.init], r[class.comparators.inverting.compare]
::class "InvertingComparator" public mixinclass Comparator
::method init
  expose held
  use strict arg held
::method compare
  expose held
  use strict arg first, second
  answer = held~compare(first, second)
  return -answer

-- r[class.comparators.numeric.init], r[class.comparators.numeric.compare]
::class "NumericComparator" public mixinclass Comparator
::method init
  expose digits
  use strict arg digits = 9
  numeric digits digits
::method compare
  expose digits
  use strict arg first, second
  numeric digits digits
  difference = first - second
  return sign(difference)
