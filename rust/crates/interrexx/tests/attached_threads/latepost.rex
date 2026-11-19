/* An attached thread's message waits on an event semaphore that a started activity posts after a
   sleep of seven seconds, a sleep it is in when the message arrives.
   spec: native.threads.attach */
t = .ticks~new
.poster~new~start('go', t)
call SysSleep 0.5
call AtSend t, 'AWAITSEM'
do while AtDone() < 1
  call SysSleep 0.05
end
say 'done'
::requires 'ticks.cls'
::requires 'attach' LIBRARY
::class poster
::method go
  use arg t
  call SysSleep 7
  t~postsem
