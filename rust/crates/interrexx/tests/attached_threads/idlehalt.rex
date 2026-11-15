/* A thread attaches and stays idle while the main program calls Halt through the instance, then
   sends THREE: the halt asked nothing of the idle activity, so the message runs to its end and
   leaves no condition. The main program traps its own HALT.
   spec: native.threads.halt */
t = .ticks~new
call AtAttachIdle t, 'THREE'
do while AtAttached() = 0
  call SysSleep 0.05
end
call on halt name halted
call AtHalt 0
nop
call AtOpen
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
