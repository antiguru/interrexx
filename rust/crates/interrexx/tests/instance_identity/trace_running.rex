/* Deviation 166: the routine `r` is running when the first SetTrace is served, so the `>K>`
   lines of its DO header for TO, BY and FOR are left out. The one for WHILE is printed.
   spec: native.threads.trace.deviation.166 */
call htsave
call r
say 'end'
exit
::requires 'halttrace' LIBRARY
::routine r
nop
call httrace 1
do i = 1 to 2 by 1 for 2 while i < 5
  nop
end
call httrace 0
