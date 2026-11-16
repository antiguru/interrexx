/* A thread attaches and stays idle while the main program calls SetTrace through the instance,
   then sends THREE: the trace asked nothing of the idle activity, so the message runs untraced.
   spec: native.threads.trace */
t = .ticks~new
call AtAttachIdle t, 'THREE'
do while AtAttached() = 0
  call SysSleep 0.05
end
call AtTrace 1
call AtOpen
call AtAwaitDone
call AtTrace 0
say AtReport()
::requires 'ticks.cls'
::requires 'attach' LIBRARY
