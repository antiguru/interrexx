/* A DetachThread from a thread not attached to the context, and a second DetachThread from the
   one that was, do nothing: the attachment sends again between them.
   spec: native.threads.attach */
t = .ticks~new
call AtWrong t
do while AtDone() < 1
  call SysSleep 0.05
end
say AtReport()
say 'ticked' t~count
::requires 'ticks.cls'
::requires 'attach' LIBRARY
