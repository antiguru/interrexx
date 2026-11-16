/* A started activity's call into the extension calls back, through its own call's context, a
   routine that waits on an event semaphore nothing posts, while the main program runs Rexx
   clauses for ever. The wait is inverted and goes on. Six seconds later a thread of the
   extension attaches and sends STOP, whose method calls Halt through the instance: the HALT ends
   the wait, which the routine traps, and the main program traps its own HALT.
   spec: conc.locks.deviation.14, conc.halt.signal */
.environment~stopper = .halter~new
t = .waiter~new~start('go')
call SysSleep 0.1
signal on halt
do forever
end
halt:
say 'main halted'
say 'waiter' t~result
::requires 'attach' LIBRARY
::class waiter
::method go
  r = .routine~new('r', "call AtSendAfter .stopper, 'STOP', 6000; signal on halt;",
    ".eventsemaphore~new~wait; return 'not halted'; halt: return 'halted'")
  return AtPauseCall(300, r)
::class halter
::method stop
  call AtHalt 0
