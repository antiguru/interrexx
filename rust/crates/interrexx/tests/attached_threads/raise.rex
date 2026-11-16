/* A callback that raises a condition leaves it for the attached thread to check and clear, and
   the attachment sends again afterwards.
   spec: native.threads.attach */
t = .ticks~new
call AtRaising t
do while AtDone() < 1
  call SysSleep 0.05
end
say AtReport()
say 'ticked' t~count
::requires 'ticks.cls'
::requires 'attach' LIBRARY
