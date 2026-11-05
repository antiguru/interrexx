call TestAddCommandEnvironment "rc-condition", "condition"
cases = .array~of( -
  .array~of("signal on user foo", "address 'rc-condition' 'USER FOO'", -
    "return 'no condition'", "foo: return condition('C') rc", "::options error syntax"), -
  .array~of("call on user foo", "address 'rc-condition' 'USER FOO'", -
    "return 'after' rc", "foo: say '  handler' condition('C'); return", "::options error syntax"), -
  .array~of("address 'rc-condition' 'USER FOO'", "return rc", "::options error syntax"), -
  .array~of("address 'rc-condition' 'NOTREADY'", "return rc", "::options error syntax"), -
  .array~of("address 'rc-condition' 'USER FOO'", "return rc", "::options failure syntax"), -
  .array~of("address 'rc-condition' 'NOTREADY'", "return rc", "::options notready syntax"), -
  .array~of("address 'rc-condition' 'ERROR'", "return rc", "::options error syntax"), -
  .array~of("address 'rc-condition' 'ERROR'", "return rc", "::options failure syntax"))
do i = 1 to cases~items
  say i run(cases[i])
end
exit
run: procedure
  signal on syntax
  r = .routine~new("case", arg(1))
  return r~call
syntax:
  return 'SYNTAX' condition('O')~code
::requires 'FUNCTIONPackage.cls'
