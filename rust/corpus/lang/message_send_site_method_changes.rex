/* One send site, reached again after every way a method set can change while
   a program runs: each probe below sends ~m from the same clause. */
k = .K~new
call probe k, 'start'
.K~define('M', 'return "K.m defined"')
call probe k, 'define, instance made before'
call probe .K~new, 'define, instance made after'
s = .S~new
call probe s, 'subclass inherits the define'
.K~define('M', 'return "K.m again"')
call probe s, 'subclass sees a second define'
.K~delete('M')
call probe s, 'delete'
call probe .K~new, 'delete, instance made after'
.S~inherit(.Mix)
call probe s, 'inherit'
.S~uninherit(.Mix)
call probe s, 'uninherit'
o = .S~new
o~setOwn('M', 'return "own m"')
call probe o, 'setMethod'
o~unsetOwn('M')
call probe o, 'unsetMethod'
d = .directory~new
d~put('return "enhanced m"', 'M')
e = .S~enhanced(d)
call probe e, 'enhanced'
call probe .S~new, 'after enhanced'

/* The class side. */
call probe .C, 'class method'
.Meta~define('M', 'return "Meta.m"')
call probe .C, 'metaclass define'
call probe .object~subclass('C3', .Meta), 'class made after the metaclass define'
.C~setOwn('M', 'return "C own m"')
call probe .C, 'class setMethod'

/* One site, two receiver classes in turn. */
list = .array~of(.K~new, .S~new, .Mix2User~new, .S~new, .K~new)
do r over list
  call probe r, 'alternating' r~class~id
end

/* Access changes between sends. */
p = .P~new
call probe p, 'public'
.P~method('M')~setPrivate
signal on syntax name private
call probe p, 'private'
private:
say 'private: error' rc
signal off syntax

/* Classes made and dropped: a later class may get a dropped class's
   behaviour, and the site must not answer the dropped class's method. */
wrong = 0
do i = 1 to 300
  t = .object~subclass('T' || i)
  t~define('M', 'return' i)
  if t~new~m \== i then wrong += 1
  v = .test_routines~probeValue(t~new)
  if v \== i then wrong += 1
  drop t
  if i // 50 = 0 then call collect
end
say 'made and dropped:' wrong 'wrong'
exit

probe: procedure
  use arg obj, label
  say label':' obj~m
  return

collect:
  do 2000
    a = .array~new(100)
  end
  return

::class K
::method m
  return 'K.m'
::method unknown
  return 'unknown' arg(1)

::method setOwn
  use arg name, source
  self~setMethod(name, source)
::method unsetOwn
  use arg name
  self~unsetMethod(name)

::class S subclass K

::class Mix mixinclass object
::method m
  return 'Mix.m'

::class Mix2User subclass object inherit Mix

::class Meta subclass class

::class C metaclass Meta
::method m class
  return 'C.m'
::method unknown class
  return 'C unknown' arg(1)
::method setOwn class
  use arg name, source
  self~setMethod(name, source)

::class P
::method m
  return 'P.m'

::class test_routines
::method probeValue class
  use arg obj
  return obj~m
