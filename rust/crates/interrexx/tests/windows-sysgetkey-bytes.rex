/* spec: util.windows.interaction */
parse arg page
if SysGetKey('NOECHO') \== 'E0'x then exit 27
if SysGetKey('NOECHO') \== '48'x then exit 28
if SysGetKey('NOECHO') \== '00'x then exit 32
if SysGetKey('NOECHO') \== '3B'x then exit 33
select
  when page = 1252 then expected = 'E9'x
  when page = 437 then expected = '82'x
  otherwise exit 29
end
if SysGetKey('NOECHO') \== expected then exit 30
if SysGetKey('NOECHO') \== 'A' then exit 31
exit 0
