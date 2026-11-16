/* SetTrace(1) traces the main program from its next clause, and the routine it calls
   afterwards starts with its own setting, so none of its clauses is traced.
   spec: native.threads.trace */
call htsave
call httrace 1
call r
call httrace 0
say 'end'
exit
::requires 'halttrace' LIBRARY
::routine r
do i = 1 to 2
  nop
end
