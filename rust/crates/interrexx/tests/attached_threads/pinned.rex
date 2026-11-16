/* An attached thread's message makes the native call that starts a second thread and joins it:
   the second thread's message runs while that call blocks inside the first message.
   spec: native.threads.attach */
t = .ticks~new
call AtSendJoin t
do while AtDone() < 1
  call SysSleep 0.05
end
say 'ticked' t~count
::requires 'ticks.cls'
::requires 'attach' LIBRARY
