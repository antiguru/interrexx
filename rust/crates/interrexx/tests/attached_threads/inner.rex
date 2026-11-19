/* A thread detaches from its attachment from inside a message it sent through it: the activity
   ends once that message returns, and the thread's next attach takes its context.
   spec: native.threads.attach */
t = .ticks~new
call AtInner t
do while AtDone() < 1
  call SysSleep 0.05
end
say AtReport()
::requires 'ticks.cls'
::requires 'attach' LIBRARY
