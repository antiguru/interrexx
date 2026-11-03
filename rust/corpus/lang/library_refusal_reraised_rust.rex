/* A SYNTAX condition a library call ends with is re-raised in its caller,
   whether the boundary refused an argument (88.921, 88.909) or the argument's
   own MAKESTRING raised (40.1): POSITION, PROGRAM and PACKAGE are the calling
   clause's and PROPAGATED is 1. A routine by name, a ::ROUTINE bound to the
   entry, and two methods of a class in a required package. */
r = .Re~new('a*b')
i = 0
next:
i = i + 1
if i > 6 then exit
signal on syntax name t
select
  when i = 1 then say RxCalcSqrt('abc')
  when i = 2 then say sq('abc')
  when i = 3 then say r~does(.object~new)
  when i = 4 then say r~doparse(.object~new)
  when i = 5 then say RxCalcSqrt(.Raiser~new)
  when i = 6 then say r~does(.Raiser~new)
end
t:
c = condition('O')
pk = c~package
if pk \= .nil then pk = filespec('n', pk~name)
say i c~code 'pos' c~position 'prop' c~propagated 'prog' filespec('n', c~program) 'pkg' pk 'tb' c~traceback~items
signal next

::requires 'rxmath' LIBRARY
::requires 're.cls'
::routine sq external "LIBRARY rxmath RxCalcSqrt"

::class Raiser
::method makestring
  raise syntax 40.1
