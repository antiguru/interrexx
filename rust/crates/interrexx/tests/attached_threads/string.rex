/* The bytes StringData answers on an attached thread stay readable after a later callback.
   spec: native.threads.attach */
t = .ticks~new
call AtString t
do while AtDone() < 1
  call SysSleep 0.05
end
say AtReport()
say 'ticked' t~count
::requires 'ticks.cls'
::requires 'attach' LIBRARY
