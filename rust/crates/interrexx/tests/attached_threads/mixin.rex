/* The array members of the thread interface, called through a routine's own context on an
   instance of a class of the program that inherits a mixin class of the interpreter's own library
   ahead of a second class of the program, a subclass of Array that redefines the methods of the
   matching names. Each redefinition says that it runs: none of them is sent to.
   spec: native.threads.attach */
a = .mixed~new
s = .stem~new('S.')
p = .supplier~new(.array~of('i1'), .array~of('x1'))
d = .directory~new
t = .stringtable~new
o = .object~new
k = .package~new('members', .array~of('::routine r public', '  return 1'))
r = .routine~new('r', 'return 1')
m = .method~new('m', 'return 1')
do line over AtMembers(a, s, p, d, t, o, k, r, m)
  if line~left(5) == 'Array' then say line
end
::requires 'attach' LIBRARY
::class mixed subclass xarray inherit comparable
::class xarray subclass array
::method put
  say 'PUT runs'
  forward class (super)
::method at
  say 'AT runs'
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
