t = .methodtester~new
call show 'haltX', t~TestRaiseConditionX("HALT", "halt description")
call show 'nomX', t~TestRaiseConditionX("NOMETHOD")
call show 'haltS', t~TestRaiseConditionSyntaxX("HALT")
call show 'nomS', t~TestRaiseConditionSyntaxX("NOMETHOD")
call show 'nomS2', t~TestRaiseConditionSyntaxX("NOMETHOD", "FOO", .array~of("receiver"))
call show 'userS', t~TestRaiseConditionSyntaxX("USER MINE")
say 'continue' t~TestGetObjectVariableOrNil("CONTINUE")
call show 'syntaxX', t~TestRaiseConditionX("SYNTAX", "no number")
say 'ret1' t~TestRaiseCondition("USER MINE")
say 'ret2' t~TestRaiseCondition("USER MINE", , , 7)
call on user mine name handler
say 'ret3' t~TestRaiseCondition("USER MINE")
say 'ret4' t~TestRaiseCondition("USER MINE", , , 7)
call off user mine
c0 = rco_syntaxCondition()
call show 'rcoSyntax', t~TestRaiseConditionObjectX(c0)
say 'same' (c~identityHash == c0~identityHash) (rco_syntaxCondition()~position == c0~position)
call show 'rcoErr', t~TestRaiseConditionObjectX(rco_errorCondition())
call show 'rcErr', t~TestRaiseConditionX("ERROR", "a command")
d = .directory~new; d~condition = "user mine"; d~description = "hand made"
call show 'hand1', t~TestRaiseConditionObjectX(d)
d = .directory~new; d~condition = "SYNTAX"; d~code = "40.1"; d~additional = .array~of("HAND")
call show 'hand2', t~TestRaiseConditionObjectX(d)
c0 = rco_haltCondition()
call show 'rcoHalt', t~TestRaiseConditionObjectX(c0)
call show 'rcoHaltS', t~TestRaiseConditionObjectSyntaxX(c0)
c0 = rco_nomethodCondition()
call show 'rcoNom', t~TestRaiseConditionObjectX(c0)
call show 'rcoNomS', t~TestRaiseConditionObjectSyntaxX(c0)
src = .array~of("use arg tester, c", "signal on syntax", "tester~TestRaiseConditionObject(c)", -
  "return .nil", "syntax: return condition('o')", "::options error syntax")
r = .routine~new("optionsErrorSyntax", src)
call show 'opt1', r~call(t, rco_errorCondition())
call show 'opt2', r~call(t, rco_userCondition())
call show 'err1', t~TestRaiseConditionObjectX()
call show 'err2', t~TestRaiseConditionObjectX("not a directory")
call show 'err3', t~TestRaiseConditionObjectX(.directory~new)
d = .directory~new; d~condition = "SYNTAX"
call show 'err4', t~TestRaiseConditionObjectX(d)
d~code = "100.1"
call show 'err5', t~TestRaiseConditionObjectX(d)
d~code = "40.1"; d~additional = "not an array"
call show 'err6', t~TestRaiseConditionObjectX(d)
exit
handler: say '  handler ran'; return
show: procedure expose c
  use arg label, c
  if c == .nil then do; say label 'NIL'; return; end
  add = c~additional
  if add~isA(.array) then add = 'array' add~items add[1]
  say label c~condition '|' c~code '|' c~rc '|' c~description '|' add '|' c~result '|' c~traceback~isA(.list) '|' c~errortext '|' c~message
  return
::requires 'METHODPackage.cls'
::routine rco_syntaxCondition
  signal on syntax
  call rco_raise "SYNTAX"
  syntax:
  return condition('o')
::routine rco_errorCondition
  signal on error
  call rco_raise "ERROR"
  error:
  return condition('o')
::routine rco_userCondition
  signal on user mine
  call rco_raise "USER"
  mine:
  return condition('o')
::routine rco_haltCondition
  signal on halt
  call rco_raise "HALT"
  halt:
  return condition('o')
::routine rco_nomethodCondition
  signal on nomethod
  .object~new~noSuchMethod
  nomethod:
  return condition('o')
::routine rco_raise
  use arg which
  select
    when which = "SYNTAX" then raise syntax 40.1 array ("XYZ") description "the description"
    when which = "ERROR" then raise error 127 description "a command"
    when which = "USER" then raise user mine description "user description" additional "user additional"
    when which = "HALT" then raise halt description "halt description"
  end
