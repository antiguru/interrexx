/* A thread attaches and sends STALLED, which waits in a call into the extension until the main
   program, from a call of its own, has called HaltThread, SetThreadTrace and DetachThread on that
   thread's context. The three return at once and do nothing, so the main program's call lets
   STALLED go on, which ends without a condition.
   spec: native.threads.attach.deviation.175 */
t = .ticks~new
call on halt name halted
call AtAttachIdle t, 'STALLED'
do while AtAttached() = 0
  call SysSleep 0.05
end
took = AtMisuse()
say 'returned at once' (took < 1000)
do while AtDone() < 1
  call SysSleep 0.05
end
say AtReport()
exit
halted:
  say 'main halted'
  return
::requires 'ticks.cls'
::requires 'attach' LIBRARY
