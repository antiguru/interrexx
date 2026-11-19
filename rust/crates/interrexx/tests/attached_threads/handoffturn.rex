/* Two started activities take turns with the guard of an object, each sleeping while it holds it
   and between its turns, so the guard is never free while both run, until the attached thread's
   POKE, which needs the guard too, has run or each has taken the guard five hundred times. POKE,
   sent again from its place in the queue, gets its turn while they take theirs, so each loop ends
   because POKE ran.
   spec: native.threads.attach.deviation.178 */
o = .contended~new
call AtSendAfter o, 'POKE', 200
a = o~start('SPIN')
b = o~start('SPIN')
say a~result
say b~result
::requires 'attach' LIBRARY
::class contended
::method init
  expose poked
  poked = 0
::method spin unguarded
  expose poked
  do 500 until poked
    self~step
    call SysSleep 0.003
  end
  if poked then return 'ended by poke'
  return 'not poked'
::method step
  call SysSleep 0.01
::method poke
  expose poked
  poked = 1
  say 'poke'
