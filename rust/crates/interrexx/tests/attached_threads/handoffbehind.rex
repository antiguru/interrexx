/* handoff.rex with two started activities that queue for the guard after POKE has arrived and
   before the started activity releases it. The release grants the guard to the main program, which
   POKE's message stopped, and POKE, sent again once the main program has it, keeps the place its
   wait took when it arrived, ahead of the two, so it gets the guard before them.
   spec: native.threads.attach.deviation.178 */
o = .handoff~new
o~start('HOLD')
.queued~new~start('RUN', o, 'M', 0.7)
.queued~new~start('RUN', o, 'N', 0.8)
call SysSleep 0.1
call AtSendAfter o, 'POKE', 300
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
::method turn
  use arg name
  say name
::method poke
  say 'poke'
::class queued
::method run
  use arg o, name, after
  call SysSleep after
  o~turn(name)
