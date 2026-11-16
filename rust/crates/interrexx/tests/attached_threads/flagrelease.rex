/* flagpinned.rex where a second message, seven and a half seconds in, posts the semaphore the
   started activity's pinned wait waits on. The callback's wait for the flag has been reported by
   then, and the pinned wait ending does not report it again.
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
call AtSendAfter f, 'RELEASE', 7500
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
  say 'pinner released'
::class flagged
::attribute flag
::method init
  expose flag
  flag = 0
::method release unguarded
  .never~post
  say 'release sent'
::method awaitflag unguarded
  expose flag
  .go~post
  guard off when flag = 1
  say 'flag seen'
