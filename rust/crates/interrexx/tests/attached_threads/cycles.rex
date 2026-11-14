/* One thread attaches, sends a message and detaches ten thousand times: every message runs, and
   the attaches reuse the thread contexts of the ones before.
   spec: native.threads.attach */
t = .ticks~new
call AtCycles t, 10000
do while AtDone() < 1
  call SysSleep 0.05
end
say 'ticks' t~count
say AtReport()
::requires 'ticks.cls'
::requires 'attach' LIBRARY
