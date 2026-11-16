/* A started activity calls into the extension, which calls back through its own call's context
   once the main program has begun to run Rexx clauses until a flag is set. The routine it calls
   waits on an event semaphore that a thread of the extension attaches and posts a second later.
   The main program, interrupted below the callback, cannot run, so the wait is inverted, but the
   extension may still end it: the wait goes on, the post ends it, and the routine sets the flag.
   spec: conc.locks.deviation.14 */
.environment~sem = .eventsemaphore~new
.environment~done = 0
t = .waiter~new~start('go')
do until .environment~done = 1
end
say 'main saw done, result' t~result
::requires 'attach' LIBRARY
::class waiter
::method go
  r = .routine~new('r', "call AtSendAfter .sem, 'POST', 1000; .sem~wait; .environment~done = 1;",
    "return 'posted'")
  return AtPauseCall(300, r)
