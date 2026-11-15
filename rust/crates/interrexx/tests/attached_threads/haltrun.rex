/* An attached thread's message loops for ever while the main program is in a call into the
   extension, which calls Halt through the instance after half a second: the message ends with
   the HALT condition, which it does not trap, so the thread finds SYNTAX, and the main program
   traps its own HALT once the call returns.
   spec: native.threads.halt */
t = .ticks~new
call on halt name halted
call AtSendReporting t, 'SPIN'
call AtHalt 500
do while AtDone() < 1
  call SysSleep 0.05
end
say AtReport()
exit
halted:
  say 'main halted'
  return
::requires 'ticks.cls'
::requires 'attach' LIBRARY
