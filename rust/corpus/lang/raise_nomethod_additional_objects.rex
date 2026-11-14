/* An untrapped RAISE NOMETHOD is a 97.1 error whose ADDITIONAL holds the
   objects the clause's ADDITIONAL and DESCRIPTION gave, unconverted: a
   reader that takes one's string value runs its MAKESTRING then. Only that
   is compared; the count and order of the items are not specified. */
signal on syntax name s1
k = .key~new
raise nomethod additional ('obj') description (k)
s1: o = condition('O'); say o~code o~additional~hasItem(k)
call find o~additional
signal on syntax name s2
a = .array~of('p', 'q')
raise nomethod additional (a) description 'm'
s2: o = condition('O'); say o~code o~additional~hasItem(a)
signal on syntax name s3
k = .key~new
raise nomethod additional (k)
s3: o = condition('O'); say o~code o~additional~hasItem(k)
signal on syntax name s4
k = .key~new
call sub k
s4: o = condition('O'); say o~code o~additional~hasItem(k)
call find o~additional
exit
sub: raise nomethod additional ('obj') description (arg(1)) exit
find: procedure
do item over arg(1)
  if item~isA(.key) then found = item
end
say 'found' found~class~id
say 'reading'
say '['found']'
return
::class key
::method makestring
  say 'makestring ran'
  return 'xyz'
::method string
  say 'string ran'
  return 'qqq'
