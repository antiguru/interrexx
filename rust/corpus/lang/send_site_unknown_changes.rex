/* One send site whose message the receiver's class answers through UNKNOWN,
   reached again after every change that moves what it answers: a method of
   the class defined, hidden and deleted, a mixin inherited and uninherited,
   an UNKNOWN of the class's own, an entry method and an unknown method of the
   directory, receivers with methods of their own from enhanced and setMethod,
   receivers of other classes, and a method object made. Each class change is
   read through an instance made after it. */
d = .dir~new
d~put('entry', 'A')
withA = .table~new
withA['A'] = 'return "enhanced"'
withUnknown = .table~new
withUnknown['UNKNOWN'] = 'use arg name; return "enhanced unknown" name'
do i = 1 to 19
  select
    when i <= 8 then r = .dir~new~~put('fresh' i, 'A')
    when i = 12 then r = .dir~enhanced(withA)
    when i = 13 then r = .dir~enhanced(withUnknown)
    when i = 14 then r = .directory~new~~put('plain', 'A')
    when i = 17 then r = .other~new
    when i = 18 then r = .other~new~~ownUnknown
    when i = 19 then r = .other~new
    otherwise r = d
  end
  call show r, i
  select
    when i = 1 then .dir~define('A', 'return "defined"')
    when i = 2 then .dir~define('A')
    when i = 3 then .dir~delete('A')
    when i = 4 then .dir~inherit(.mixa)
    when i = 5 then .dir~uninherit(.mixa)
    when i = 6 then .dir~define('UNKNOWN', 'use arg name; return "class unknown" name')
    when i = 7 then .dir~delete('UNKNOWN')
    when i = 9 then do; d~remove('A'); d~setMethod('A', 'return "entry method"'); end
    when i = 10 then do; d~unsetMethod('A'); d~setMethod('UNKNOWN', 'use arg name; return "entry unknown" name'); end
    when i = 11 then do; d~unsetMethod('UNKNOWN'); d~a = 'entry again'; end
    when i = 15 then m = .method~new('m', 'return 1')
    otherwise nop
  end
end
exit
show: procedure
  use arg r, i
  say i r~a
  return
::class dir subclass directory
::class mixa mixinclass object
::method a
  return "mixin"
::class other
::method ownUnknown
  self~setMethod('UNKNOWN', 'use arg name; return "own unknown" name')
::method unknown
  use arg name
  return "other unknown" name
