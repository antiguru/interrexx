/* A thread attaches and sends LONG, which loops until the misuse calls return. While it runs, the main
   program, from a call into the extension, calls HaltThread, SetThreadTrace and DetachThread on
   that thread's context. The three must return before LONG can end: it runs to its end untraced
   and leaves no condition, and the thread detaches itself afterwards.
   spec: native.threads.attach.deviation.177 */
t = .ticks~new
call on halt name halted
call AtAttachIdle t, 'LONG'
do while AtAttached() = 0
  call SysSleep 0.05
end
call AtMisuse
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
