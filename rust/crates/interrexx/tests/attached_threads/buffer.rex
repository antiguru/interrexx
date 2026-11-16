/* A buffer string an attached thread makes is written through BufferStringData in a later
   callback, finished, and passed to a method.
   spec: native.threads.attach */
t = .ticks~new
call AtBuffer t
do while AtDone() < 1
  call SysSleep 0.05
end
say AtReport()
::requires 'ticks.cls'
::requires 'attach' LIBRARY
