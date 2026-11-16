/* Two started activities take turns with the guard of an object, each sleeping while it holds it
   and between its turns, while an attached thread's POKE needs the same guard. Whichever
   activity POKE's message stops, POKE runs once and the program ends.
   spec: native.threads.attach.deviation.178 */
o = .contended~new
call AtSendAfter o, 'POKE', 200
o~start('SPIN')
o~start('SPIN')
::requires 'attach' LIBRARY
::class contended
::method spin unguarded
  do 60
    self~step
    call SysSleep 0.003
  end
::method step
  call SysSleep 0.01
::method poke
  say 'poke'
