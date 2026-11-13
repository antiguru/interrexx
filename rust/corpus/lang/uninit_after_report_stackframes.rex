/* An UNINIT that the termination sweep runs after the main program's
   untrapped failure has been reported: the condition it traps holds its own
   frames, and none of the levels the reported failure unwound through. */
o = .K~new
n = 0
zs = 'n = n + 1; if n < 3 then interpret zs; else x = 1/0'
interpret zs

::class K
::method uninit
  signal on syntax
  x = 2/0
  syntax:
  do frame over condition('o')~stackframes
    say 'frame' frame~line frame~traceline
  end
