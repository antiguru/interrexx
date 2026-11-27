/* A string or number compared with an object whose REQUEST is Rexx code: a
   string's == sends REQUEST('STRING') to the other operand, so that code runs
   under the level of the collection method that compares, which
   .context~stackFrames lists. The item, index or value of a base class does
   not make that level go away: Array~hasItem, Table [] and []=, and Set~put. */
o = .k~new('abc')
say 'hasItem abc:' .array~of(o)~hasItem('abc')
say 'hasItem 7:' .array~of(o)~hasItem(7)
say 'hasItem 1.5:' .array~of(o)~hasItem(1.5)
say 'hasItem array:' .array~of(o)~hasItem(.array~new)
t = .table~new
t[o] = 1
say 'put abc'
t['abc'] = 2
say 'items' t~items
say 'at abc:' t['abc']
s = .set~new
s~put(o)
say 'set put abc'
s~put('abc')
say 'items' s~items
-- A comparison whose REQUEST itself compares under a level of its own.
n = .k~new('abc', .true)
say 'nested hasItem abc:' .array~of(n)~hasItem('abc')
-- A plain string table: no comparison runs Rexx code.
p = .table~new
p['abc'] = 1
p['abc'] = 2
say 'plain items' p~items
exit

::class k
::method init
  expose v nest
  use arg v, nest = .false
::method hashcode
  expose v
  return v~hashcode
::method request
  expose v nest
  call frames 'REQUEST' arg(1)
  if nest then do
    inner = .table~new
    inner[.k~new(v)] = 1
    inner[v] = 2
  end
  return v
::method '=='
  call frames '=='
  return 0
::routine frames
  parse arg what
  s = ''
  do f over .context~stackFrames
    if f~name == .context~package~name then s = s 'PROGRAM'
    else s = s f~name
  end
  say '  ' what 'frames:' s
