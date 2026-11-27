/* Report byte values without involving the console output code page. */
report = value('INTERREXX_KEY_PROBE_REPORT',, 'ENVIRONMENT')
if report = '' then exit 27
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
