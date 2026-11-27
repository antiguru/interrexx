/* A call site that found FILESPEC, a routine of the REXX package at language
   level 6.06 (Deviation 179), finds the routine ~addRoutine put in front of
   it, on its next pass. */
p = .context~package
do i = 1 to 2
  say 'internal' i filespec('N', '/a/b.c')
  if i = 1 then p~addRoutine('FILESPEC', .routine~new('x', 'return "added" arg(1)'))
end
