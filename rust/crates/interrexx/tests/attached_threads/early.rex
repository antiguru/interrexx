/* After an external library loaded, a started method waits for its own message's result before
   it replies, while the main program still has lines to print: the main program runs to its end,
   and the wait is refused only then.
   spec: conc.activities.deviation.20 */
m = .message~new(.k~new, 'run')
.environment~mm = m
m~start
do i = 1 to 3
  call SysSleep 0.1
  say 'main' i
end
say 'main done'
::class k
::method run
  say 'in run'
  say .environment~mm~result
::requires 'attach' LIBRARY
