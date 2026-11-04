t = .methodtester~new
sources = .array~of("raise user boom description 'd1' additional ('a1') return 'r1'", -
  "raise user boom description 'd2'; return 'after'", -
  "signal on user boom; raise user boom description 'd3'; return 'x'; boom: raise propagate", -
  "raise user boom description 'd4' exit 'e4'", -
  "call inner; return 'r5'; inner: raise user boom description 'd5'")
signal on user boom name again
do source over sources
  report = t~TestCallRoutineCondition(.routine~new("r", source), .array~new)
  say 'condition:' report[1] report[2] report[3] report[4] report[5]
end
signal off user boom
r = .context~package~findRoutine("ONEINTARG")
c = t~TestCallRoutineConditionObject(r, .array~of("abc"))
say 'native:' c~condition c~code c~stackFrames~items
do f over c~stackFrames
  say '  frame:' f~type (f~executable == r) (f~context == .nil)
end
r = .routine~new("rr", .array~of("nop", "x = 1 + 'a'"))
c = t~TestCallRoutineConditionObject(r, .array~new)
say 'rexx:' c~condition c~code c~position c~program c~stackFrames~items
do f over c~stackFrames
  say '  frame:' f~type f~line (f~context == .nil)
end
x = 'mine'
r = .context~package~findRoutine("CALLERVARIABLES")
report = t~TestCallRoutine(r, .array~of("X"))
say 'variables:' report[1] report[2] report[3] report[4] report[5] report[6] report[7] x
signal on syntax
say 1 + 'b'
again:
say 'raised again in the caller'
exit
syntax:
say 'syntax:' condition('o')~position condition('o')~stackFrames~items
exit
::requires 'METHODPackage.cls'
::routine oneIntArg EXTERNAL "LIBRARY orxfunction TestOneIntArg"
::routine callerVariables EXTERNAL "LIBRARY orxfunction TestCallerVariables"
