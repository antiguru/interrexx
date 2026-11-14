/* Two attached threads send at once, each as an activity of its own, each to an object of its
   own.
   spec: native.threads.attach */
t = .ticks~new
u = .ticks~new
call AtTicker t, 5
call AtTicker u, 5
do while AtDone() < 2
  call SysSleep 0.05
end
say 'ticked' t~count u~count
::requires 'ticks.cls'
::requires 'attach' LIBRARY
