/* After the extension's library loaded, an attached thread sends SEND to a message whose method
   waits for that message's own result before it replies, while the main program waits for the
   method to start: the wait in the attached thread's callback is refused.
   spec: conc.activities.deviation.20 */
m = .message~new(.k~new, 'run')
.environment~mm = m
.environment~started = .eventsemaphore~new
call AtSend m, 'SEND'
-- The method posts once it runs, so the main program ends with the callback under way.
.environment~started~wait
say 'main done'
::class k
::method run
  say 'in run'
  .environment~started~post
  say .environment~mm~result
::requires 'attach' LIBRARY
