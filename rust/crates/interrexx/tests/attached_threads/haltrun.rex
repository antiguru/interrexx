/* An attached thread's message loops for ever while the main program is in a call into the
   extension, which calls Halt through the instance once the message has started: the message
   ends with the HALT condition, which it does not trap, so the thread finds SYNTAX 4.1, and the
   main program traps its own HALT once the call returns.
   spec: native.threads.halt */
t = .ticks~new
call on halt name halted
call AtSendReporting t, 'SPIN'
call AtHaltStarted
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
