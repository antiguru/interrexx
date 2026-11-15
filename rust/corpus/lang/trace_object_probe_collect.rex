/* .TraceObject~option 'P' with a collector: every trace line becomes a
   TraceObject, collected and not written. A >V> or >=> line's object carries
   VARIABLE (a compound's has no name and no value), NOTIFY gets each object,
   a method whose first clause is EXPOSE announces from its TRACE, and a
   class whose INIT is traced still answers NEW. The wording of an
   announcement is not specified, so a >I> or <I< line is printed as its
   flag. */
got = .array~new
told = .array~new
.traceObject~notify = told
.traceObject~collector = got
.traceObject~option = 'P'
src = .array~of( -
  'a = 1', -
  'b.x = a', -
  'c = b.x b.', -
  'b. = 5', -
  "parse value 'p q' with p q", -
  'o = .t~new(3)', -
  'say o~v', -
  'do i = 1 to 1; end', -
  'return i', -
  '::class t', -
  '::method init', -
  '  expose v', -
  '  trace i', -
  '  use arg v', -
  '::method v', -
  '  expose v', -
  '  trace a', -
  '  return v', -
  '::options trace i')
say .routine~new('probe', src)~call
.traceObject~collector = .nil
.traceObject~notify = .nil
.traceObject~option = 'N'
say got~items told~items
do n = 1 to got~items
  t = got[n]
  line = t~traceLine
  if wordpos(word(line, 1), '>I> <I<') > 0 then line = word(line, 1)
  same = (told[n] == t)
  v = t~variable
  if v == .nil then say same line
  else say same line '|' v~name '|' v~value '|' v~assignment
end
