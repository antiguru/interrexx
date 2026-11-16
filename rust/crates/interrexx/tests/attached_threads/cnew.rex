/* The members of the thread interface that ask an object IsInstanceOf and HasMethod, called through
   a routine's own context on an instance of a subclass of a class Class~new made, whose methods of
   the matching names, given to it with define, say that they run. The program's own message runs
   one of them, the members neither.
   spec: native.threads.members */
c = .class~new('M')
c~define('ISINSTANCEOF', 'say "ISINSTANCEOF runs"; forward class (super)')
c~define('HASMETHOD', 'say "HASMETHOD runs"; forward class (super)')
x = c~subclass('X')
say x~superclasses~makestring('l', ' ')
o = x~new
say o~isinstanceof(.object)
s = .stem~new('S.')
p = .supplier~new(.array~of('i1'), .array~of('x1'))
k = .package~new('members', .array~of('::routine r public', '  return 1'))
r = .routine~new('r', 'return 1')
m = .method~new('m', 'return 1')
do line over AtMembers(.array~new, s, p, .directory~new, .stringtable~new, o, k, r, m)
  if line~left(2) == 'Is' | line~left(3) == 'Has' then say line
end
::requires 'attach' LIBRARY
