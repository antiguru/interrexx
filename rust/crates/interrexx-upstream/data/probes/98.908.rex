quiet_ = .output~destination(.stream~new('/dev/null'))
say .k~hello .k~both
say .sub~hello .sub~both .sub~metaclass~id
say .sub2~hello .sub2~metaclass~id
say .plain~metaclass~id
say .k4~hello .k5~hello .k6~hello
call try '::class a metaclass spec_dir_nometa'
exit
try:
  signal on syntax name e
  p = .package~new('spec-dir-class-meta', .array~of(arg(1)))
  say arg(1) '=> ok'
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
::class meta subclass class
::method hello
  return 'meta hello'
::method both
  return 'meta both'
::class k metaclass meta
::method both class
  return 'k both'
::class sub subclass k
::class meta2 subclass class
::method hello
  return 'meta2 hello'
::class sub2 subclass k metaclass meta2
::class plain
::class base
::method hello class
  return 'base hello'
::class k4 subclass base
::class k5 subclass base metaclass meta
::class k6 subclass base metaclass meta
::method hello class
  return 'k6 hello'
