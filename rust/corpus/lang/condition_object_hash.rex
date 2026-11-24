/* A condition object is a Directory: its hash methods answer, CONDITION('O')
   is a copy of it, and re-raising it through ~result keeps its identity. The
   order of the interpreter's own entries is not specified, so the indexes are
   listed sorted. */
/* The THROWN entry, of level 6.07, is left out (Deviation 179, #804). */
m = .message~new(.t~new, 'BOOM')
signal on syntax name s1
m~send
s1:
c = m~errorCondition
say c~class items(c)
s = ''; do i over c~allIndexes~sort; if i \== 'THROWN' then s = s i; end
say s
say c~hasIndex('CODE') c~hasIndex('NOPE') c~hasIndex('code')
say c~at('CODE') c['CODE'] c~code
say c~isEmpty c~hasItem(c~at('CODE')) c~index('42.3')
say c~allItems~items - c~hasIndex('THROWN') c~makeArray~items - c~hasIndex('THROWN') c~supplier~class
o = condition('O')
say (o == c) items(o)
o['NEW'] = 1
say items(o) items(c) (m~errorCondition == c)
signal on syntax name s2
r = m~result
s2:
c3 = m~errorCondition
say (c3 == c) (condition('O') == c) c~propagated items(c3) c~position
call on user foo name h1
call raiser
signal on novalue name h2
say zz
h2:
call show 'novalue', condition('O')
signal on syntax name h3
call deep
h3:
c = condition('O')
call show 'propagated' c~propagated, c
do k over c; nop; end
say c~hasIndex(k)
exit
h1:
call show 'call-on', condition('O')
return
show: procedure
  use arg tag, c
  s = ''
  do i over c~allIndexes~sort; if i \== 'THROWN' then s = s i; end
  say tag items(c)':'s
  return
items: return arg(1)~items - arg(1)~hasIndex('THROWN')
raiser:
  raise user foo description 'd1' additional (.array~of(1,2)) return
::routine deep
  signal on syntax name inner
  x = 1/0
inner:
  raise propagate
::class t
::method boom
  return 1/0
