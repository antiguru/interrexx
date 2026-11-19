/* An attached thread's message waits on an event semaphore that a started activity posts once its
   call into the extension, which blocks for seven seconds, returns. The activity is in that call
   when the message arrives.
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
  call AtPause 7000
  t~postsem
