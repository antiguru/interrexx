/* A started activity calls into the extension, which calls back through its own call's context
   once the main program has begun to run Rexx clauses for ever. The routine it calls waits for
   the result of a message nothing sends, which only the main program, interrupted below the
   callback, could run on to give: the wait is inverted and refused.
   spec: conc.locks.deviation.14 */
t = .waiter~new~start('go')
call SysSleep 0.1
do forever
end
::requires 'attach' LIBRARY
::class waiter
::method go
  r = .routine~new('r', "return .message~new('abc', 'length')~result")
  return AtPauseCall(300, r)
