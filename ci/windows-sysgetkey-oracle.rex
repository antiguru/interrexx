/* Report byte values without involving the console output code page. */
parse arg report
signal on syntax
do 4
  key = SysGetKey('NOECHO')
  call lineout report, c2x(key)
end
call lineout report
exit 0
syntax:
call lineout report, 'SYNTAX' condition('O')~code
call lineout report
exit 1
