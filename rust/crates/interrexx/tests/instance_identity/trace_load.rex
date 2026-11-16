/* The library is loaded after the routine `r` has run once, so `r` was compiled before any
   SetTrace. Entered again after the first SetTrace, its clauses after the second are traced
   with every line, the `>K>` lines of its DO header among them.
   spec: native.threads.trace.deviation.166 */
call r 0
ok = .context~package~loadLibrary('halttrace')
call htsave
call httrace 0
call r 1
say 'end'
exit
::routine r
if arg(1) then call httrace 1
do i = 1 to 2 by 1 for 2
  nop
end
if arg(1) then call httrace 0
