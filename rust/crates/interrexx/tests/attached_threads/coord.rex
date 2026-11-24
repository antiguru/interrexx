/* An attached thread waits on an event semaphore. A second thread sends the post
   once the waiting callback has started.
   spec: native.threads.attach */
t = .coordinatedTicks~new
call AtSendWhenStarted t, 'POSTSEM'
call AtSend t, 'AWAITSEM'
do while AtDone() < 2
  call SysSleep 0.05
end
say 'done'
::requires 'ticks.cls'
::requires 'attach' LIBRARY
::class coordinatedTicks subclass ticks
::method awaitsem unguarded
  call AtStart
  forward class (super)
