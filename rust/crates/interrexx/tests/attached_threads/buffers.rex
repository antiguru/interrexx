/* One attachment makes, writes, finishes and releases thirty thousand buffer strings of 100000
   bytes: each answers its own area and length, and what the released ones held does not pile up.
   spec: native.threads.attach */
t = .ticks~new
call AtBuffers t, 30000
do while AtDone() < 1
  call SysSleep 0.05
end
say AtReport()
::requires 'ticks.cls'
::requires 'attach' LIBRARY
