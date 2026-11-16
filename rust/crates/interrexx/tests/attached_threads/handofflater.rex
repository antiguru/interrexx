/* handoff.rex where the attached thread's unguarded LATER says a line and then sends POKE from
   inside its own method. POKE's wait comes after the message has begun, so when the release grants
   the guard to the main program, which the message stopped, the wait is refused as an inverted
   wait, and the program ends with rc 120.
   spec: native.threads.attach.deviation.178 */
o = .handoff~new
o~start('HOLD')
call SysSleep 0.1
call AtSendAfter o, 'LATER', 300
o~g
do while AtDone() < 1
  call SysSleep 0.05
end
::requires 'attach' LIBRARY
::class handoff
::method hold
  call SysSleep 1
  say 'hold'
::method g
  say 'g'
::method later unguarded
  say 'later'
  self~poke
::method poke
  say 'poke'
