/* A started activity holds the guard of an object while it sleeps a second, and the main program
   waits for that guard. An attached thread's POKE, which needs the guard too, arrives during the
   main program's wait and queues behind it. The release grants the guard to the main program, which
   POKE's message stopped, so POKE gives the interpreter back and is sent again behind it: g runs,
   then POKE.
   spec: native.threads.attach.deviation.178 */
o = .handoff~new
o~start('HOLD')
call SysSleep 0.1
call AtSendAfter o, 'POKE', 300
o~g
do while AtDone() < 1
  call SysSleep 0.05
end
::requires 'attach' LIBRARY
::class handoff
::method hold
  call SysSleep 1
  say 'hold'
::method g
  say 'g'
::method poke
  say 'poke'
