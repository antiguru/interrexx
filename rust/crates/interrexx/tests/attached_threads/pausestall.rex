/* pausecall.rex with the post fifteen seconds after the wait begins: the inverted wait goes on
   past the five seconds after which stderr says that it waits while the main program makes no
   progress, and the post still ends it.
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
  r = .routine~new('r', "call AtSendAfter .sem, 'POST', 15000; .sem~wait; .environment~done = 1;",
    "return 'posted'")
  return AtPauseCall(300, r)
