/* The code of a trapped INTERPRET error, for strings that hold a translation
   error. */
call show 'y = (1'
call show 'nop; nop; y = a[1'
call show 'say 1 /* open'
call show 'if 1 = 1'
call show 'select; when 1 = 1'
call show 'if 1 = 1; nop'
call show 'select; when 1 = 1; nop; end'
call show 'do; nop'
call show 'loop 3; nop'
call show 'select; nop; end'
call show 'select; end'
call show 'do i = 1; end j'
call show 'do label a; end b'
call show 'do; end x'
call show 'select label s; when 1 then nop; end t'
call show 'select; when 1 then nop; end t'
call show 'if 1 = 1 then'
call show 'if 1 = 1 then nop; else'
call show 'select; when 1 then nop; otherwise'
call show 'nop' || '0a'x || 'y = (1'
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
