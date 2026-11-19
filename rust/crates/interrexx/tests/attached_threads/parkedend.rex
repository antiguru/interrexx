/* After the main program has ended, while the end waits for a started activity that naps, an
   attached thread's message arrives and waits, parked, on an event semaphore that a second thread
   posts after the nap: the parked message holds the end past the nap, and says it saw the post.
   spec: native.threads.end */
t = .ticks~new
call AtSendAfter t, 'AWAITSEM', 300
call AtSendAfter t, 'POSTSEM', 1500
t~start('NAP')
say 'main ends'
::requires 'ticks.cls'
::requires 'attach' LIBRARY
