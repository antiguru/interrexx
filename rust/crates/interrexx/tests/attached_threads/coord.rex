/* An attached thread's message waits on an event semaphore that a second attached thread's
   message, sent a second later, posts.
   spec: native.threads.attach */
t = .ticks~new
call AtSend t, 'AWAITSEM'
call AtSendAfter t, 'POSTSEM', 1000
do while AtDone() < 2
  call SysSleep 0.05
end
say 'done'
::requires 'ticks.cls'
::requires 'attach' LIBRARY
