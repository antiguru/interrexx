-- A condition trapped after a call to FILESPEC, a routine of the REXX package
-- at language level 6.06 (Deviation 179, r[cond.errors.deviation.179]): the
-- context of the native level the argument error comes from, and POSITION and
-- PROGRAM from the caller, the first frame with a package.
call t3
exit

t3: procedure
  signal on syntax
  x = filespec('N')
  return
syntax: call show; return

show:
  c = condition('O')
  say c~code c~position c~hasIndex('POSITION') filespec('n', c~program) c~traceback~items
  do l over c~traceback; if l~word(1)~datatype('W') then say ' ' l; end -- numbered levels only: a built-in one may be absent, r[class.stackframe.builtin]
  do f over c~stackframes
    say ' ' f~type f~name f~line (f~context == .nil) f~arguments~items
  end
  return
