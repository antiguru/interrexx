quiet_ = .output~destination(.stream~new('/dev/null'))
/* USE ARG >name refuses with 98.995 a target that is not an uninitialised
   local variable: one EXPOSE or USE LOCAL bound to the object's pool, set or
   not, one already assigned, an empty stem of another name, and one an
   earlier reference bound to the object's pool. An unset local target, a
   USE LOCAL name, a dropped stem, and a second reference to an unset local
   variable bind. A second >stem. compares the stem with the name of the
   variable that owns the storage, however many references and internal
   calls lie between. */
o = .k~new
a = 1; a. = 'stem default'; a.1 = 'one'; b. = 'b default'
call try o, 'kindFirst', >a.
o~twiceExposed
say c.
say 'c.1' c.1
drop d.
say 'd.1' d.1
e.1 = 'x'; drop e.1
say 'e.1' e.1
f. = g.
say h. j. k. l.
say 'h.1' h.1 'j.1' j.1 'k.1' k.1 'l.1' l.1
say 'a' a 'a.1' a.1
exit

try: procedure
  use arg o, message, first, second
  signal on syntax name caught
  if arg() = 3 then answer = o~send(message, first)
  else answer = o~send(message, first, second)
  say message 'answers' answer
  return
caught:
  do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return

::class k
::method refExposed
  expose abc
  use arg >abc
  return abc
::method refAutoExposed
  use local
  use arg >abc
  return abc
::method refLocal
  use local abc
  use arg >abc
  return abc
::method refExposedStem
  expose abc.
  use arg >abc.
  return abc.1
::method refAutoExposedStem
  use local
  use arg >abc.
  return abc.1
::method refLocalStem
  use local abc.
  use arg >abc.
  return abc.1
::method doubleArgStem
  use arg abc., >abc.
  return abc.1
::method refAfterDrop
  use arg abc.
  drop abc.
  use arg , >abc.
  return abc.1
::method doubleArg
  use arg abc, >abc
  return abc
::method refTwice
  use arg >abc
  use arg >abc
  return abc
::method refTwiceUnset
  use arg >abc
  use arg >abc
  return 'bound twice'
::method kindFirst
  expose abc
  use arg >abc
::method unsetPool
  expose pooled
  use arg >pooled
::method setPool
  expose pooled
  pooled = 'object value'
  use arg >pooled
::method twiceStem
  use arg >abc.
  use arg >abc.
  abc.1 = 'set'
  return 'bound twice'
::method innerThenOuter
  call bindOnce arg(1)
  use arg >abc.
  abc.1 = 'set'
  return 'bound after the internal call'
bindOnce:
  use arg >abc.
  return
::method innerTwiceOuter
  call bindTwice arg(1)
  use arg >abc.
  abc.1 = 'set'
  return 'bound after the internal call'
bindTwice:
  use arg >abc.
  use arg >abc.
  return
::method passedOnTwice
  call first arg(1)
  return result
first: procedure
  use arg >x.
  call second >x.
  return result
second: procedure
  use arg >y.
  call third >y.
  return result
third: procedure
  use arg >z.
  use arg >z.
  z.1 = 'set'
  return 'bound three levels down'
::method innerOfPassedOn
  call outer arg(1)
  return result
outer: procedure
  use arg >x.
  call middle >x.
  return result
middle: procedure
  call innermost arg(1)
  use arg >w.
  w.1 = 'set'
  return 'bound after the passed-on internal call'
innermost:
  use arg >w.
  return
::method twiceExposed
  expose v
  signal on syntax name caught
  say 'twiceExposed answers' self~bindTwice(>v)
  return
caught:
  do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
::method bindTwice
  use arg >q
  use arg >q
  return 'bound twice'
