/* Run by the driver hostwait through RexxStart: a started method waits for its own message's
   result before it replies, which nothing can end, while the main program goes on to its end.
   spec: conc.activities.deviation.20 */
m = .message~new(.k~new, 'run')
.environment~mm = m
m~start
call SysSleep 0.1
say 'main done'
return 'main'
::class k
::method run
  say 'in run'
  say .environment~mm~result
