/* A thread attaches and stays idle while the main program, from a call into the extension, calls
   HaltThread and SetThreadTrace on that thread's context. Both return and do nothing: the thread
   then sends THREE, which runs to its end untraced and leaves no condition, and the main program
   is not halted either.
   spec: native.threads.attach.deviation.175 */
t = .ticks~new
call on halt name halted
call AtAttachIdle t, 'THREE'
do while AtAttached() = 0
  call SysSleep 0.05
end
call AtHaltIdle 0
call AtTraceIdle 1
call AtOpen
do while AtDone() < 1
  call SysSleep 0.05
end
say AtReport()
say 'main done'
exit
halted:
  say 'main halted'
  return
::requires 'ticks.cls'
::requires 'attach' LIBRARY
