/* An attached thread's message that started before the main program ends, and runs past it,
   keeps the program running until it finishes.
   spec: native.threads.end, native.threads.attach.deviation.178 */
t = .ticks~new
call AtSlow t
do while t~started \= 1
  call SysSleep 0.05
end
say 'main ends'
::requires 'ticks.cls'
::requires 'attach' LIBRARY
