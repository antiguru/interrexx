/* An attached thread's message waits, with GUARD OFF WHEN, for a flag the main program sets once
   it wakes from a sleep of an hour. The message arrives during the sleep.
   spec: native.threads.attach.deviation.176 */
t = .ticks~new
t~flag = 0
call AtSend t, 'AWAITFLAG'
call SysSleep 3600
t~flag = 1
say 'done'
::requires 'ticks.cls'
::requires 'attach' LIBRARY
