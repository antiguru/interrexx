/* A second attached thread's TICK waits for the guard of an object whose guarded method the first
   thread's message runs.
   spec: native.threads.attach.deviation.174 */
t = .ticks~new
call AtSend t, 'BUSY'
call AtSendAfter t, 'TICK', 200
do while AtDone() < 2
  call SysSleep 0.05
end
say 'ticked' t~count
::requires 'ticks.cls'
::requires 'attach' LIBRARY
