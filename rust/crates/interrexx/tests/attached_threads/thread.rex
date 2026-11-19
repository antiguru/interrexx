/* A callback through an attached thread's context runs as an activity of its own, whose thread
   number is not the main program's.
   spec: native.threads.attach */
t = .ticks~new
call AtTicker t, 1
do while AtDone() < 1
  call SysSleep 0.05
end
if t~thread == .context~thread then say 'same'
else say 'differs'
::requires 'ticks.cls'
::requires 'attach' LIBRARY
