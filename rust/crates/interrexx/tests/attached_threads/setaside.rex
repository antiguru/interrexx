/* The main program waits, inside a comparator that sortWith calls, for the result of a message
   that a started activity sends. The activity first has a thread of the extension post an event
   semaphore six seconds later, then sends the message, which makes the main program ready, and
   then waits on the semaphore inside a comparator of its own. The main program's frames lie
   below that wait, so it is set aside and the wait is inverted, but the extension may still end
   it: the wait goes on past the five seconds after which stderr says so, the post ends it, and
   both sorts finish.
   spec: conc.locks.deviation.14 */
.environment~sem = .eventsemaphore~new
m1 = .message~new('abc', 'length')
t = .s~new~start('RUN', m1)
a = .array~of(2, 1)
a~sortWith(.c~new(m1))
say 'main sorted'
say t~result
::requires 'attach' LIBRARY
::class s
::method run
  use arg m1
  call AtSendAfter .sem, 'POST', 6000
  m1~send
  b = .array~of(2, 1)
  b~sortWith(.w~new)
  return 'run posted'
::class c
::method init
  expose m
  use arg m
::method compare
  expose m
  use arg l, r
  say 'main got' m~result
  return l - r
::class w
::method compare
  use arg l, r
  .sem~wait
  return l - r
