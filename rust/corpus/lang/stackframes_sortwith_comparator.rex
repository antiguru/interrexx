/* A comparator's compare that sortWith or stableSortWith runs has its own
   level first in its stackFrames, and the program's level, at the line of
   the sort, last (r[class.stackframe.builtin]). A condition raised in
   compare lists compare's level first too. Whether the sort has a level of
   its own between them is not specified, so only levels of Rexx code, whose
   trace line is numbered (r[class.stackframe.traceline]), are printed. */
test = .array~of(2, 1)
c = .recorder~new
test~sortWith(c)
call show 'sortWith', c~frames, c
test~stableSortWith(c)
call show 'stableSortWith', c~frames, c
signal on syntax
test~sortWith(.failing~new)
exit
syntax:
  do f over condition('o')~stackFrames
    if f~traceLine~word(1)~datatype('W') then
      say 'condition' f~type filespec('name', f~name) f~line f~traceLine
  end
  exit

show: procedure
  use arg tag, frames, c
  f = frames[1]
  say tag f~type f~name f~line (f~target == c),
    (f~executable == .recorder~method('COMPARE')) f~traceLine
  f = frames[frames~items]
  say tag 'last' f~type f~line
  return

::class recorder
::attribute frames
::method compare
  self~frames = .context~stackFrames
  return 0

::class failing
::method compare
  return 1 + 'x'
