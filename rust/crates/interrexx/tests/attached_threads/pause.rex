/* An attached thread's message, sent while the main program is in a call into the extension that
   blocks for a second, sleeps two seconds: the order of its end and the call's return.
   spec: native.threads.attach.deviation.176 */
t = .ticks~new
call AtSendAfter t, 'SLOW', 200
call AtPause 1000
say 'main returned'
do while AtDone() < 1
  call SysSleep 0.05
end
::requires 'ticks.cls'
::requires 'attach' LIBRARY
