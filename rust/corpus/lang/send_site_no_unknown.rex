/* One send site whose message neither the receiver's class nor a class beneath
   it answers, and no UNKNOWN does: NOMETHOD is raised. The same site is
   reached again by a receiver whose own methods include an UNKNOWN, then by a
   plain receiver and after a class UNKNOWN is defined and deleted. Each
   receiver is made after the change it reads. */
b = .plain~new~~ownUnknown
do i = 1 to 8
  select
    when i = 2 | i = 6 then r = b
    otherwise r = .plain~new
  end
  call show r, i
  select
    when i = 4 then .plain~define('UNKNOWN', 'use arg name; return "class unknown" name')
    when i = 5 then .plain~delete('UNKNOWN')
    otherwise nop
  end
end
exit
show: procedure
  use arg r, i
  signal on nomethod name raised
  say i r~gone
  return
raised:
  say i 'nomethod' condition('D')
  return
::class plain
::method ownUnknown
  self~setMethod('UNKNOWN', 'use arg name; return "own unknown" name')
