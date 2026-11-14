/* An attached thread's message, sent while the main program waits on an event semaphore, posts
   it, sleeps a second and says it ends: the order of that and the main program's wake.
   spec: native.threads.attach.deviation.174 */
t = .ticks~new
call AtSendAfter t, 'POSTNAP', 200
t~awaitsem
do while AtDone() < 1
  call SysSleep 0.05
end
::requires 'ticks.cls'
::requires 'attach' LIBRARY
