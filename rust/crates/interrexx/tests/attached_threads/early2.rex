/* After an external library loaded, a started method waits for its own message's result before
   it replies, and the main program then raises an error: the error is reported, and the wait is
   refused only then.
   spec: conc.activities.deviation.20 */
m = .message~new(.k~new, 'run')
.environment~mm = m
m~start
call SysSleep 0.2
say 'main' 1
x = 1 + 'a'
::class k
::method run
  say 'in run'
  say .environment~mm~result
::requires 'attach' LIBRARY
