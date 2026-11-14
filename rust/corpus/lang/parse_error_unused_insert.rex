/* The code of a trapped translation error, for strings that raise one beside
   controls that raise none. */
call show 'use arg (a)'
call show 'call ns:"x"'
call show 'call ns:'
call show 'do while 1 until 2; end'
call show 'nop 1'
call show 'x: nop'
call show 'expose a'
exit

show: procedure
  parse arg src
  signal on syntax
  interpret src
  say src '-> no error'
  return
syntax:
  c = condition('o')
  say src '->' c~code
  return
