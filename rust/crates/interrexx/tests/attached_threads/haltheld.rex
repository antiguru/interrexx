/* One attached thread's message loops for ever, and a second attached thread sends THREE while
   it does. The main program, from a call into the extension, then calls Halt through the
   instance. The looping message ends with the HALT condition, which it does not trap, so its
   thread finds SYNTAX 4.1. THREE runs to its end, and the main program traps its own HALT.
   spec: native.threads.halt */
t = .ticks~new
call on halt name halted
call AtAttachIdle t, 'SPIN'
call AtAttachHeld t, 'THREE'
do while AtAttached() = 0
  call SysSleep 0.05
end
call AtHaltHeld
nop
do while AtDone() < 2
  call SysSleep 0.05
end
say AtReport()
exit
halted:
  say 'main halted'
  return
::requires 'ticks.cls'
::requires 'attach' LIBRARY
