/* The first SetTrace, served in the routine `r`, finds the main program running. The internal
   routine `sub`, called from the main program afterwards, prints every line, the `>K>` lines
   of its DO header among them. spec: native.threads.trace.deviation.166 */
call htsave
call r
call sub
say 'end'
exit
sub:
call httrace 1
do j = 1 to 2 by 1
end
call httrace 0
return
::requires 'halttrace' LIBRARY
::routine r
call httrace 1
call httrace 0
