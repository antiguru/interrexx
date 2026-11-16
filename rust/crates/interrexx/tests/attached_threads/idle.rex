/* A thread that has sent its message and stays attached, idle, for two seconds does not keep
   the program running: it ends as soon as the main program does.
   spec: native.threads.end */
t = .ticks~new
call AtIdle t
do while t~count < 1
  call SysSleep 0.05
end
say 'main ends'
::requires 'ticks.cls'
::requires 'attach' LIBRARY
