/* The library is loaded after `sub` has run once, so `sub` was compiled before any SetTrace.
   Entered again after the first SetTrace, its clauses after the second are traced.
   spec: native.threads.trace */
call sub 0
ok = .context~package~loadLibrary('halttrace')
call htsave
call httrace 0
call sub 1
say 'end'
exit
sub:
  if arg(1) then call httrace 1
  x = 1 + 2
  return
