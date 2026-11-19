/* A second attached thread's LATER arrives while the first thread's message runs Rexx code in a
   guarded method of an object, and LATER sends TICK to that object from inside its own method.
   spec: native.threads.attach */
t = .ticks~new
call AtSend t, 'BUSY'
call AtSendAfter t, 'LATER', 200
do while AtDone() < 2
  call SysSleep 0.05
end
say 'ticked' t~count
::requires 'ticks.cls'
::requires 'attach' LIBRARY
