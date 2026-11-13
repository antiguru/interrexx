/* probe 98.935 */
o = .rep~new
x = o~m
o~go
o~wait
exit
::class rep
::method init
  expose go done
  go = .false; done = .false
::method go unguarded
  expose go
  go = .true
::method wait unguarded
  expose done
  guard on when done
::method m
  expose go done
  signal on syntax name oops
  reply 'one'
  guard on when go
  go = .false
  reply 'two'
oops: c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end
  done = .true
