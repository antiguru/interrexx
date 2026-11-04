t = .methodtester~new
sources = .array~of("raise user boom description 'd' additional ('a')", -
  "raise notready description 'n'", "raise error 5", "raise error 0", "raise error (-3)", -
  "raise error 127", "raise error 'abc'", "raise failure (-1)", -
  "raise user boom array (1, 2)", "raise user boom description (2 + 3)", -
  "raise user boom description (.array~of(1))", "x = 1 + 'a'")
do source over sources
  report = t~TestDecodeCondition(conditionOf(source))
  additional = report[5]
  if additional~isA(.array) then additional = 'array of' additional~items
  say report[1] '|' report[2] '|' report[3] '|' report[4] '|' additional '|' report[6]
end
exit
::requires 'METHODPackage.cls'
::routine conditionOf
  signal on user boom name caught
  signal on error name caught
  signal on failure name caught
  signal on notready name caught
  signal on syntax name caught
  call raiseFrom arg(1)
  return .nil
caught:
  return condition("O")
::routine raiseFrom
  interpret arg(1)
