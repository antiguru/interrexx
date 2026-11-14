/* The members of the thread interface that store into or fetch from a collection, or ask an object,
   a package, a routine or a method, called through a routine's own context on objects whose
   classes redefine the methods of those names. Each redefinition says that it runs: none of them
   is sent to.
   spec: native.threads.attach */
a = .xarray~new
s = .xstem~new('S.')
p = .xsupplier~new(.array~of('i1'), .array~of('x1'))
d = .xdirectory~new
t = .xstringtable~new
o = .xobject~new
k = .xpackage~new('members', .array~of('::routine r public', '  return 1'))
r = .xroutine~new('r', 'return 1')
m = .xmethod~new('m', 'return 1')
do line over AtMembers(a, s, p, d, t, o, k, r, m)
  say line
end
say 'array' a~items 'directory' d~items 'table' t~items
::requires 'attach' LIBRARY
::class xarray subclass array
::method put
  say 'PUT runs'
  forward class (super)
::method at
  say 'AT runs'
  forward class (super)
::method '[]'
  say '[] runs'
  forward class (super)
::method '[]='
  say '[]= runs'
  forward class (super)
::method hasindex
  say 'HASINDEX runs'
  forward class (super)
::method append
  say 'APPEND runs'
  forward class (super)
::method size
  say 'SIZE runs'
  forward class (super)
::method items
  say 'ITEMS runs'
  forward class (super)
::method dimension
  say 'DIMENSION runs'
  forward class (super)
::class xstem subclass stem
::method at
  say 'AT runs'
  forward class (super)
::method put
  say 'PUT runs'
  forward class (super)
::method '[]'
  say '[] runs'
  forward class (super)
::method '[]='
  say '[]= runs'
  forward class (super)
::method hasindex
  say 'HASINDEX runs'
  forward class (super)
::method remove
  say 'REMOVE runs'
  forward class (super)
::method todirectory
  say 'TODIRECTORY runs'
  forward class (super)
::class xsupplier subclass supplier
::method available
  say 'AVAILABLE runs'
  forward class (super)
::method item
  say 'ITEM runs'
  forward class (super)
::method index
  say 'INDEX runs'
  forward class (super)
::method next
  say 'NEXT runs'
  forward class (super)
::class xdirectory subclass directory
::method put
  say 'PUT runs'
  forward class (super)
::method at
  say 'AT runs'
  forward class (super)
::method remove
  say 'REMOVE runs'
  forward class (super)
::method hasindex
  say 'HASINDEX runs'
  forward class (super)
::class xstringtable subclass stringtable
::method put
  say 'PUT runs'
  forward class (super)
::method at
  say 'AT runs'
  forward class (super)
::method remove
  say 'REMOVE runs'
  forward class (super)
::method hasindex
  say 'HASINDEX runs'
  forward class (super)
::class xobject
::method isinstanceof
  say 'ISINSTANCEOF runs'
  forward class (super)
::method hasmethod
  say 'HASMETHOD runs'
  forward class (super)
::class xpackage subclass package
::method findclass
  say 'FINDCLASS runs'
  forward class (super)
::method routines
  say 'ROUTINES runs'
  forward class (super)
::method publicroutines
  say 'PUBLICROUTINES runs'
  forward class (super)
::method classes
  say 'CLASSES runs'
  forward class (super)
::method publicclasses
  say 'PUBLICCLASSES runs'
  forward class (super)
::method definedmethods
  say 'DEFINEDMETHODS runs'
  forward class (super)
::class xroutine subclass routine
::method package
  say 'PACKAGE runs'
  forward class (super)
::class xmethod subclass method
::method package
  say 'PACKAGE runs'
  forward class (super)
