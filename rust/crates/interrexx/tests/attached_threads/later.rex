/* An attached thread's message, unguarded, sends TICK to an object whose guarded method the main
   program runs, a method that started the thread.
   spec: native.threads.attach.deviation.178 */
t = .ticks~new
t~holdlater
do while AtDone() < 1
  call SysSleep 0.05
end
say 'ticked' t~count
::requires 'ticks.cls'
::requires 'attach' LIBRARY
