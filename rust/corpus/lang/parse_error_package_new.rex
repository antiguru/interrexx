/* Package~new over source lines that do not parse raises SYNTAX at the
   failing line, trapped and then untrapped. A built-in level is not printed:
   whether a build has one is not specified (r[class.stackframe.builtin]), and
   its trace line has no line number. */
signal on syntax
p = .package~new('x', .array~of('say 1', 'say (  ; nop'))
say 'not reached'
exit
syntax:
  o = condition('O')
  say o~code o~position rc
  do l over o~traceback
    if word(l, 1) \== '*-*' then say '  tb:' l
  end
  p = .package~new('y', .array~of('nop', 'lab: if then'))
  say 'not reached'
