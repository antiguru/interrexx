/* After an external library loaded, a method that has replied waits for its own message's result
   and completion: the reply settled both, so neither wait blocks.
   spec: conc.activities.deviation.20 */
.environment~done = .eventsemaphore~new
m = .message~new(.k~new, 'run')
.environment~mm = m
r = m~send
.environment~done~wait
say 'sent' r
::class k
::method run
  reply 'r'
  say 'result' .environment~mm~result
  .environment~mm~wait
  say 'waited'
  .environment~done~post
::requires 'attach' LIBRARY
