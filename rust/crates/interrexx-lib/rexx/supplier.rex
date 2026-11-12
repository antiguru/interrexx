/* SupplierMixin: the methods a Supplier shares that read the arrays it holds.
   The class is not public; Supplier takes its methods from it.
   See spec/src/classes/supplier.md, rules class.supplier.allitems,
   class.supplier.allitems.sends and class.supplier.supplier. */

::class SupplierMixin

/* r[class.supplier.allitems] */
::method allItems
  if arg() > 0 then raise syntax 93.902 array (0)
  return self~getArrays~at(1)

/* r[class.supplier.allitems] */
::method allIndexes
  if arg() > 0 then raise syntax 93.902 array (0)
  return self~getArrays~at(2)

/* The two arrays of the entries from the current one on, as an array of the
   two. The first call walks the receiver by messages, as
   r[class.supplier.allitems.sends] says, and keeps the pair; every later call
   answers that pair. The private message itself is the one the rule says is
   not specified. */
::method getArrays private
  pair = self~interrexx_collected
  if pair \== .nil then return pair
  items = .array~new
  indexes = .array~new
  do while self~available
    indexes~append(self~index)
    items~append(self~item)
    self~next
  end
  pair = .array~of(items, indexes)
  self~interrexx_collected(pair)
  return pair

/* The cache of getArrays lives in the receiver's Supplier state, where init
   drops it (r[class.supplier.deviation.138]) and copy shares it
   (r[class.supplier.copy]). With the argument .nil it drops the pair. */
::method interrexx_collected private external 'LIBRARY REXX interrexx_supplier_collected'

/* r[class.supplier.supplier] */
::method supplier
  if arg() > 0 then raise syntax 93.902 array (0)
  return self
