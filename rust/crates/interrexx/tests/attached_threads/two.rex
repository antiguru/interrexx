/* Two attached threads send at once, each as an activity of its own.
   spec: native.threads.attach */
t = .ticks~new
call AtTicker t, 5
call AtTicker t, 5
do while AtDone() < 2
  call SysSleep 0.05
end
say 'ticked' t~count
::requires 'ticks.cls'
::requires 'attach' LIBRARY
