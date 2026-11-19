/* The main program waits in GUARD WHEN for a variable that a thread's message sets, and then for
   the result of a message that a second thread's message sends, each before its thread has
   attached. Each wait ends once the thread's message has run.
   spec: conc.activities.deviation.20 */
w = .waiter~new
call AtSendAfter w, 'RAISE', 300
w~await
say 'flag seen'
m = .message~new(w, 'ANSWER')
w~msg = m
call AtSendAfter w, 'SENDMSG', 300
say m~result
do while AtDone() < 2
  call SysSleep 0.05
end
::requires 'attach' LIBRARY
::class waiter
::attribute msg
::method init
  expose flag
  flag = 0
::method await
  expose flag
  guard on when flag = 1
::method raise unguarded
  expose flag
  flag = 1
::method answer unguarded
  return 'answered'
::method sendmsg unguarded
  expose msg
  msg~send
