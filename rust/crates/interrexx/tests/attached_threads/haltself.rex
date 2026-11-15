/* An attached thread's message calls HaltThread on the thread context of the thread running it,
   its attachment's: the message ends with the HALT condition at its next clause, which it does
   not trap, so the thread finds SYNTAX 4.1, and the main program, not asked, runs on.
   spec: native.threads.halt */
t = .ticks~new
call on halt name halted
call AtOpen
call AtAttachIdle t, 'HALTSELF'
do while AtDone() < 1
  call SysSleep 0.05
end
say AtReport()
say 'main done'
exit
halted:
  say 'main halted'
  return
::requires 'ticks.cls'
::requires 'attach' LIBRARY
