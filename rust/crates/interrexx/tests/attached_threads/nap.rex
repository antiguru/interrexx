/* An attached thread's message sleeps a second while the main program runs Rexx code, which
   counts its own sleeps until the message has napped.
   spec: native.threads.attach.deviation.178 */
t = .ticks~new
call AtSend t, 'NAP'
do while t~napping = 0 & t~napped = 0
  call SysSleep 0.05
end
count = 0
do while t~napped = 0
  count = count + 1
  call SysSleep 0.05
end
if count >= 5 then say 'main ran'
else say 'main stopped'
do while AtDone() < 1
  call SysSleep 0.05
end
::requires 'ticks.cls'
::requires 'attach' LIBRARY
