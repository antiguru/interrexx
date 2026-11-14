/* An attached thread's message waits for the result of the message whose send the main program
   runs, the send that started the thread.
   spec: native.threads.attach.deviation.176 */
t = .ticks~new
m = .message~new(t, 'SERVE')
t~msg = m
m~send
do while AtDone() < 1
  call SysSleep 0.05
end
say 'done'
::requires 'ticks.cls'
::requires 'attach' LIBRARY
