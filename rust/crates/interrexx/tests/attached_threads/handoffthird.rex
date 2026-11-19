/* handoff.rex with a third activity that queues for the guard after the main program and before
   the attached thread's POKE arrives: the guard goes to the main program, then to the third
   activity, and POKE, sent again, keeps its place behind both.
   spec: native.threads.attach.deviation.178 */
o = .handoff~new
o~start('HOLD')
call SysSleep 0.1
.third~new~start('RUN', o)
call AtSendAfter o, 'POKE', 500
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
::method t
  say 't'
::method poke
  say 'poke'
::class third
::method run
  use arg o
  call SysSleep 0.2
  o~t
