/* An operator on a variable reference runs the referenced value's own
   operator method; a failure's traceback shows the levels of Rexx code. */
s = 'abc'; n = 5; m. = 'q'; drop u
call try 't = >s; say t + 0'
call try 't = >s; say -t'
call try 't = >n; say t / 0'
call try 't = >n; say t ** "x"'
call try 't = >u; say t + 1'
call try 't = >m.; say t * 2'
call try 'tt = >s; t = >tt; say t - 1'
call try 't = >s; say \t'
call try 't = >s; say t & 1'
call try 't = >s; do i = t to 5; end'
call try 't = >n; numeric digits 1; n = "9.9E999999999"; say t > 1'
call try 't = >s; say t = 1'
call try 'say s + 0'
exit
try:
  signal on syntax
  interpret arg(1)
  say arg(1) 'ok'
  return
syntax:
  say arg(1) condition('o')~code
  do l over condition('o')~traceback; if l~word(1)~datatype('W') then say '   ' l; end -- numbered levels only: a built-in one may be absent, r[class.stackframe.builtin]
  return
