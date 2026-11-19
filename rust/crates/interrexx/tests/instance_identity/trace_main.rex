/* SetTrace(1) traces the main activity from its next clause, and SetTrace(0) ends it.
   spec: native.threads.trace */
call htsave
call httrace 1
call sub
call httrace 0
call sub
say 'end'
exit
sub:
  x = 1 + 2
  return
::requires 'halttrace' LIBRARY
