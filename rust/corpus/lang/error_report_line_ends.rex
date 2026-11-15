/* An error report goes to the trace output one line at a time, and a line
   keeps a line end that an echoed clause holds. The text of a report line is
   not specified (r[conv.error.text]), so such a line is printed as its code. */
lines = .LineCollector~new
.TraceOutput~destination(lines)
o = .replier~new
o~raise_after_reply
do i = 1 to 200 while lines~items < 4
  call SysSleep 0.01
end
.TraceOutput~destination
say 'reply thread lines' lines~items
do i = 1 to lines~items
  call show lines[i]
end
.TraceOutput~destination(.Echo~new)
call interpreted
exit

show: procedure
  parse arg line
  if word(line, 1) == 'Error' then say 'line' word(line, 1) word(line, 2)
  else say 'line' c2x(line)
  return

::routine interpreted
interpret "x = 1 +" '0a'x "+"

::class replier
::method raise_after_reply unguarded
reply
raise syntax 98.900 array('a' || '01'x || .endofline || 'second line')

::class LineCollector subclass Array
::method lineOut
  self~append(arg(1)~makeString)
  return 0

::class Echo
::method lineOut
  line = arg(1)~makeString
  if word(line, 1) == 'Error' then say 'line' word(line, 1) word(line, 2)
  else say 'line' c2x(line)
  return 0
