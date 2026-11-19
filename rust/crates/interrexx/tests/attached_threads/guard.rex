/* An attached thread's TICK waits for the guard of an object whose guarded method the main
   program runs, a method that started the thread.
   spec: native.threads.attach */
t = .ticks~new
t~hold
do while AtDone() < 1
  call SysSleep 0.05
end
say 'ticked' t~count
::requires 'ticks.cls'
::requires 'attach' LIBRARY
