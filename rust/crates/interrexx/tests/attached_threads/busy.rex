/* A second attached thread's TICK arrives while the first thread's message runs Rexx code in a
   guarded method of the same object.
   spec: native.threads.attach */
t = .ticks~new
call AtSend t, 'BUSY'
call AtSendAfter t, 'TICK', 200
do while AtDone() < 2
  call SysSleep 0.05
end
say 'ticked' t~count
::requires 'ticks.cls'
::requires 'attach' LIBRARY
