/* Two attached threads' messages sleep, the second starting while the first sleeps and
   sleeping longer: the order in which they finish.
   spec: native.threads.attach.deviation.174 */
t = .ticks~new
call AtSend t, 'LIFOA'
do while t~astarted = 0
  call SysSleep 0.05
end
call AtSend t, 'LIFOB'
do while AtDone() < 2
  call SysSleep 0.05
end
say t~order
::requires 'ticks.cls'
::requires 'attach' LIBRARY
