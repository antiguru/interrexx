/* An untrapped HALT or NOMETHOD an extension raised is an error once its
   call has returned. The native level
   reports it: by then a method has given up its receiver, so its frame is a
   routine's, and that frame has a package and no line, so the condition has
   no POSITION and the report's major line names the package with no line.
   A method, a method with the NOMETHOD receiver and name, a routine, and
   last an untrapped one from a routine for the report itself. A built-in
   level is not printed: its presence is not specified
   (r[class.stackframe.builtin]), and its trace line has no line number. */
t = .T~new
call show 'method halt', t, 'HALT'
call show 'method nomethod', t, 'NOMETHOD', 'FOO', .array~of('x')
call show 'method nomethod bare', t, 'NOMETHOD'
call show 'routine halt', .nil, 'HALT'
call show 'routine nomethod', .nil, 'NOMETHOD'
call tc 'HALT', 'stop'
say 'not reached'
exit
show: procedure
  use arg label, t, name
  signal on syntax
  select
    when t == .nil then call tc name
    when arg() == 3 then x = t~rc(name)
    otherwise x = t~rc(name, arg(4), arg(5))
  end
  say label 'not raised'
  return
syntax:
  c = condition('o')
  say label c~code '|' c~position '|' filespec('n', c~program)
  do l over c~traceback
    if word(l, 1) \== '*-*' then say '  tb' l
  end
  do f over c~stackframes
    if word(f~traceLine, 1) \== '*-*' then say '  sf' f~type f~name f~line (f~target == .nil) f~executable~class~id
  end
  return
::class T
::method rc external "LIBRARY orxmethod TestRaiseCondition"
::routine tc external "LIBRARY orxfunction TestThrowCondition"
