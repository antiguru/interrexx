/* Two attached threads' messages sleep, the second sent while the first sleeps and ending a
   second after it would: the order in which they finish.
   spec: native.threads.attach.deviation.176 */
t = .ticks~new
call AtSend t, 'LIFOA'
call AtSendAfter t, 'LIFOB', 1000
do while AtDone() < 2
  call SysSleep 0.05
end
say t~order
::requires 'ticks.cls'
::requires 'attach' LIBRARY
