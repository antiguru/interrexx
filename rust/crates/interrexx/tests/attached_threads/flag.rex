/* An attached thread's message waits with GUARD OFF WHEN for a flag that the main program sets
   once it has run Rexx code for a second.
   spec: native.threads.attach.deviation.174 */
t = .ticks~new
t~flag = 0
call AtSend t, 'AWAITFLAG'
call time 'R'
do until time('E') > 1
end
t~flag = 1
do while AtDone() < 1
  call SysSleep 0.05
end
say 'done'
::requires 'ticks.cls'
::requires 'attach' LIBRARY
