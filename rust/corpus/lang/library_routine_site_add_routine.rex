/* A call site that kept what the search finds after every routine table (an
   external file, a routine loadLibrary registered) finds the routine
   ~addRoutine or ~addPublicRoutine put in front of it, on its next pass. A
   builtin is not found that way: library_routine_site_add_routine_filespec
   has FILESPEC. */
p = .context~package
do i = 1 to 2
  say 'function' i ext1(i)
  if i = 1 then p~addRoutine('EXT1', .routine~new('x', 'return "added" arg(1)'))
end
say 'fresh' ext1(3)
do i = 1 to 2
  call ext2 i
  say 'call' i result
  if i = 1 then p~addRoutine('EXT2', .routine~new('x', 'return "added" arg(1)'))
end
x = p~loadLibrary('rxmathc')
do i = 1 to 2
  say 'library' i RxCalcSqrt(16)
  if i = 1 then p~addPublicRoutine('RXCALCSQRT', .routine~new('x', 'return "public" arg(1)'))
end
