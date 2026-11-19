/* flagsleep.rex where the message, before it waits for the flag, posts an event semaphore that a
   started activity waits on. Half a second later the activity waits in an INTERPRET on an event
   semaphore nothing posts, in the loop of the message's wait: a pinned wait that is no callback's.
   The main program sleeps only once the activity waits, and the message comes half a second into
   that sleep, so it stops the main program where it is parked.
   spec: native.threads.attach.deviation.178, conc.locks.deviation.14 */
.environment~go = .eventsemaphore~new
.environment~never = .eventsemaphore~new
.environment~waiting = 0
b = .pinner~new~start('go')
do until .waiting = 1
  call SysSleep 0.05
end
f = .flagged~new
call AtSendAfter f, 'AWAITFLAG', 500
call SysSleep 3600
f~flag = 1
say 'done'
::requires 'attach' LIBRARY
::class pinner
::method go
  .environment~waiting = 1
  .go~wait
  call SysSleep 0.5
  interpret '.never~wait'
::class flagged
::attribute flag
::method init
  expose flag
  flag = 0
::method awaitflag unguarded
  expose flag
  .go~post
  guard off when flag = 1
  say 'flag seen'
