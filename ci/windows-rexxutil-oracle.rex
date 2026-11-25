/* Compare public RexxUtil behavior in an isolated Windows directory. */
parse arg root
if root = '' then exit 1
if directory(root) = '' then exit 2
call SysMkDir 'sub'
call makeFile 'Mixed.TXT'
call makeFile 'plain'
call makeFile 'sub\inner.txt'
call SysSetFileDateTime 'Mixed.TXT', '2024-02-03', '14:05:06'
call list 'mixed.txt', 'F'
call list 'Mixed.TXT', 'FT'
call list 'Mixed.TXT', 'FLH'
call list '*.*', 'FO'
call list '*.TXT', 'FSO'
call list '.\', 'DO'
call list '.', 'DO'
call list 'mixed.txt', 'FI'
call list 'missing\*', 'FO'
call list 'Mixed.TXT', 'F', '**+**'
call list 'Mixed.TXT', 'FO', , '**+**'
call list 'Mixed.TXT', 'F', '**+**'
call list 'Mixed.TXT', 'F', , '**+**'
call list 'Mixed.TXT', 'F', '**+**'
call list 'Mixed.TXT', 'F', , '**-**'
call list 'Mixed.TXT', 'F', '-'
call list 'Mixed.TXT', 'F', '*'
call list 'Mixed.TXT', ''
call list 'Mixed.TXT', 'X'
entries = .array~of('old', 'old')
say 'array:' SysFileTree('Mixed.TXT', entries, 'FO') entries~items entries~hasIndex(2)
exit 0

makeFile: procedure
  parse arg name
  call lineout name, 'data'
  call lineout name
  return

list: procedure expose root
  use arg spec, options
  signal on syntax name invalid
  say 'tree:' spec options arg(3) arg(4)
  select
    when arg(3, 'E') & arg(4, 'E') then rc = SysFileTree(spec, 'items.', options, arg(3), arg(4))
    when arg(3, 'E') then rc = SysFileTree(spec, 'items.', options, arg(3))
    when arg(4, 'E') then rc = SysFileTree(spec, 'items.', options, , arg(4))
    otherwise rc = SysFileTree(spec, 'items.', options)
  end
  say 'rc/count:' rc items.0
  do i = 1 to items.0
    say '['changestr(root, items.i, '<ROOT>')']'
  end
  return
invalid:
  say 'syntax:' condition('O')~code
  return
