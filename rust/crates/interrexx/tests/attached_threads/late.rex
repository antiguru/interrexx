/* A message sent through a thread context after its thread detached from it is refused: the
   send answers nothing, and the message does not run.
   spec: native.threads.attach */
t = .ticks~new
call AtLate t
do while AtDone() < 1
  call SysSleep 0.05
end
say AtReport()
say 'ticked' t~count
::requires 'ticks.cls'
::requires 'attach' LIBRARY
