/* After the extension's library loaded, an attached thread sends SEND to a message whose method
   waits for that message's own result before it replies, while the main program waits for the
   thread a while: the wait in the attached thread's callback is refused.
   spec: conc.activities.deviation.20 */
m = .message~new(.k~new, 'run')
.environment~mm = m
call AtSend m, 'SEND'
do 20 while AtDone() < 1
  call SysSleep 0.05
end
say 'main done'
::class k
::method run
  say 'in run'
  say .environment~mm~result
::requires 'attach' LIBRARY
