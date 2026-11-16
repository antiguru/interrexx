/* A thread attaches and sends LONG, which runs Rexx code for two seconds. While it runs, the main
   program, from a call into the extension, calls HaltThread, SetThreadTrace and DetachThread on
   that thread's context. The three return at once and do nothing: LONG runs to its end untraced
   and leaves no condition, and the thread detaches itself afterwards.
   spec: native.threads.attach.deviation.177 */
t = .ticks~new
call on halt name halted
call AtAttachIdle t, 'LONG'
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
