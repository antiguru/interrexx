/* An attached thread sends a message while the main program runs Rexx code.
   spec: native.threads.attach */
t = .ticks~new
call AtTicker t, 1
do while AtDone() < 1
  call SysSleep 0.05
end
say 'ticked' t~count
::requires 'ticks.cls'
::requires 'attach' LIBRARY
