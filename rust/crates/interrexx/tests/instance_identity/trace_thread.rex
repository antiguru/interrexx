/* SetThreadTrace on the thread of the call turns the trace of the routine that calls it on
   and off. The first call, from the main program, is before `r` is entered, so `r` prints
   every line. spec: native.threads.trace.deviation.166 */
call htthreadtrace 0
call r
say 'end'
exit
::requires 'halttrace' LIBRARY
::routine r
nop
call htthreadtrace 1
do i = 1 to 2 by 1 for 2
  nop
end
call htthreadtrace 0
