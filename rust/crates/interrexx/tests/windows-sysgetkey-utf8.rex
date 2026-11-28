/* spec: util.windows.interaction */
reader = .reader~new
first = reader~start('key')
second = reader~start('key')
a = first~result
b = second~result
if (a || b) \== 'F09F'x & (b || a) \== 'F09F'x then exit 34
if SysGetKey('NOECHO') \== '98'x then exit 35
if SysGetKey('NOECHO') \== '80'x then exit 36
exit 0

::class reader
::method key unguarded
  return SysGetKey('NOECHO')
