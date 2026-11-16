/* The members of the thread interface that ask an object IsInstanceOf and HasMethod, called through
   a routine's own context on an instance of a class of the program that inherits Comparable, a
   mixin class of the interpreter's own library, and then a mixin class of the program that
   redefines the methods of the matching names. Comparable comes ahead of that mixin in the
   instance's class order. Each redefinition says that it runs: neither of them is sent to.
   spec: native.threads.members */
s = .stem~new('S.')
p = .supplier~new(.array~of('i1'), .array~of('x1'))
k = .package~new('members', .array~of('::routine r public', '  return 1'))
r = .routine~new('r', 'return 1')
m = .method~new('m', 'return 1')
do line over AtMembers(.array~new, s, p, .directory~new, .stringtable~new, .mixed~new, k, r, m)
  if line~left(2) == 'Is' | line~left(3) == 'Has' then say line
end
::requires 'attach' LIBRARY
::class mixed inherit comparable asker
::class asker mixinclass object
::method isinstanceof
  say 'ISINSTANCEOF runs'
  forward class (super)
::method hasmethod
  say 'HASMETHOD runs'
  forward class (super)
