/* A thread attaches in the simulation mode: the attach is refused, and the run fails.
   spec: native.threads.attach */
t = .ticks~new
call AtSend t, 'TICK'
do while AtDone() < 1
  call SysSleep 0.05
end
say 'ticked' t~count
::requires 'ticks.cls'
::requires 'attach' LIBRARY
