/* Two attached threads' messages sleep, the second sent once the first runs, which then waits
   for it to start and sleeps a second, so the second ends a second after the first would: the
   order in which they finish.
   spec: native.threads.attach.deviation.178 */
t = .ticks~new
/* Start B's waiting worker before A can suspend the main program. */
call AtSendWhenStarted t, 'LIFOB'
call AtSend t, 'LIFOA'
do while AtDone() < 2
  call SysSleep 0.05
end
say t~order
::requires 'ticks.cls'
::requires 'attach' LIBRARY
