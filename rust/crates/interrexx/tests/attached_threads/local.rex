/* A string an attached thread makes in one callback is still its own in a later one.
   spec: native.threads.attach */
t = .ticks~new
call AtLocal t
do while AtDone() < 1
  call SysSleep 0.05
end
say 'ticked' t~count
::requires 'ticks.cls'
::requires 'attach' LIBRARY
