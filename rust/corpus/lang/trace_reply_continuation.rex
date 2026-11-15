/* A replied method's continuation announces >I> and <I< again on its own
   thread under ::OPTIONS TRACE LABELS, an empty rest after a forwarded
   REPLY included. The wording of an announcement is not specified, so a >I>
   or <I< line is printed as its flag. Each send waits for the replied threads
   before the next. */
.traceOutput~destination(.shown~new)
src = .array~of( -
  'return .test', -
  '::class test', -
  '::attribute count', -
  '::method one', -
  '  forward message "onetwo"', -
  '::method onetwo', -
  '  reply', -
  '::method three', -
  '  reply 42', -
  '  forward message "threefour" continue', -
  '::method threefour', -
  '  return 42', -
  '::method m_reply', -
  '  expose count', -
  '  reply', -
  '  count -= 1', -
  '::options trace labels')
t = .package~new('replies', src)~findClass('TEST')~new
t~one
call syssleep 0.1
say 'three' t~three
call syssleep 0.1
t~count = 1
t~m_reply
call syssleep 0.1
say 'count' t~count
.traceOutput~destination
::class shown
::method lineout
  use arg traced
  line = traced~makeString
  if wordpos(word(line, 1), '>I> <I<') > 0 then line = left(line, pos(word(line, 1), line) + 2)
  say 'trace:' line
  return 0
