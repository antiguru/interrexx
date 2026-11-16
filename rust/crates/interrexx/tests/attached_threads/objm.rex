/* The members of the thread interface that store into or fetch from a collection, or ask an object,
   a package, a routine or a method, called through a routine's own context on instances of the
   built-in classes themselves, each given methods of its own under the names of the matching
   messages with setMethod and the scope OBJECT. Each such method says that it runs: none of them
   runs. The program's own `a~items` is a message, so that method says it runs.
   spec: native.threads.members */
e = .directory~new
e['OBJSET'] = 'use arg n, src; self~setMethod(n, src, "OBJECT")'
a = .array~enhanced(e)
s = .stem~enhanced(e, 'S.')
p = .supplier~enhanced(e, .array~of('i1'), .array~of('x1'))
d = .directory~new
t = .stringtable~new
o = .object~enhanced(e)
k = .package~enhanced(e, 'members', .array~of('::routine r public', '  return 1'))
r = .routine~enhanced(e, 'r', 'return 1')
m = .method~enhanced(e, 'm', 'return 1')
say a~class~id s~class~id p~class~id o~class~id k~class~id r~class~id m~class~id
call put a, 'PUT AT HASINDEX APPEND SIZE ITEMS DIMENSION'
call put s, 'AT PUT [] []= HASINDEX REMOVE TODIRECTORY'
call put p, 'AVAILABLE ITEM INDEX NEXT'
call put o, 'ISINSTANCEOF HASMETHOD'
call put k, 'FINDCLASS ROUTINES PUBLICROUTINES CLASSES PUBLICCLASSES DEFINEDMETHODS'
call put r, 'PACKAGE'
call put m, 'PACKAGE'
say 'direct:' a~items
do line over AtMembers(a, s, p, d, t, o, k, r, m)
  say line
end
exit
put: procedure
  use arg obj, names
  do n over names~makearray(' ')
    obj~objset(n, "say '"n" runs'; return 'fake'")
  end
  return
::requires 'attach' LIBRARY
