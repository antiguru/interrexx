/* The supplier members of the thread interface, called through a routine's own context on the
   supplier a stream answers, an instance of a subclass of Supplier that the interpreter's own
   library defines. That subclass's own methods answer, so the members read the stream's lines.
   spec: native.threads.attach */
call lineout 'lines.txt', 'line1'
call lineout 'lines.txt', 'line2'
call lineout 'lines.txt'
f = .stream~new('lines.txt')
p = f~supplier
say p~class~id p~available p~item p~index
p = .stream~new('lines.txt')~supplier
a = .array~new
s = .stem~new('S.')
d = .directory~new
t = .stringtable~new
o = .object~new
k = .package~new('members', .array~of('::routine r public', '  return 1'))
r = .routine~new('r', 'return 1')
m = .method~new('m', 'return 1')
do line over AtMembers(a, s, p, d, t, o, k, r, m)
  if line~left(8) == 'Supplier' then say line
end
say 'after:' p~available p~item
::requires 'attach' LIBRARY
