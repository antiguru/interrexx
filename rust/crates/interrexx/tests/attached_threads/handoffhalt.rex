/* handoff.rex where the started activity sleeps three seconds and a thread of the extension calls
   Halt through the instance a second in, while POKE's message waits behind the main program: the
   started activity traps the HALT and releases the guard, which goes to the main program, and
   POKE is sent again behind it. The main program traps its HALT in g, and POKE runs.
   spec: native.threads.attach.deviation.178, native.threads.halt */
o = .handoff~new
o~start('HOLD')
call SysSleep 0.1
call AtSendAfter o, 'POKE', 300
call AtHaltAfter 1000
o~g
do while AtDone() < 1
  call SysSleep 0.05
end
::requires 'attach' LIBRARY
::class handoff
::method hold
  signal on halt
  call SysSleep 3
  say 'hold'
  return
halt:
  say 'hold halted'
::method g
  signal on halt
  say 'g'
  return
halt:
  say 'g halted'
::method poke
  say 'poke'
