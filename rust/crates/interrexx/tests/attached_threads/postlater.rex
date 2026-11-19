/* The main program waits on an event semaphore, without a limit, before the thread that posts it
   has attached: the thread waits a while first. The wait ends once the thread's message posts.
   spec: conc.activities.deviation.20 */
t = .ticks~new
call AtSendAfter t, 'POSTSEM', 300
t~awaitsem
say 'posted'
do while AtDone() < 1
  call SysSleep 0.05
end
::requires 'ticks.cls'
::requires 'attach' LIBRARY
