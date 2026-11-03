/* A WHEN whose consequence is an IF with no ELSE, where the IF's condition
   is false. The IF's false path lands on whatever follows the WHEN's branch
   -- the next WHEN, the OTHERWISE, or the SELECT's END -- and that has to
   close the branch and leave the SELECT, not run on into the OTHERWISE or
   the next WHEN. The Rexx Parser's Expressions.cls has this shape:
   `When 1 Then If \logical, \commasFound Then Return expressions[1]`. */
n = 1
select case n
  when 0 then say "case: zero"
  when 1 then if n = 2 then say "case: inner"
  otherwise say "case: other"
end
say "case: after"

select
  when n = 0 then say "plain: zero"
  when n = 1 then if n = 2 then say "plain: inner"
  otherwise say "plain: other"
end
say "plain: after"

/* The next listed WHEN follows the branch. */
select
  when n = 1 then if n = 2 then say "next-when: inner"
  when n = 1 then say "next-when: second"
  otherwise say "next-when: other"
end
say "next-when: after"

/* No OTHERWISE: the branch runs out at the END. */
select
  when n = 1 then if n = 2 then say "no-otherwise: inner"
end
say "no-otherwise: after"

/* Two IFs nested, both false and both ending the branch. */
select
  when n = 1 then if n = 1 then if n = 2 then say "nested: inner"
  otherwise say "nested: other"
end
say "nested: after"

/* Inside a loop, which keeps frames of its own open around the SELECT. */
do i = 1 to 3
  select
    when i = 1 then if i = 2 then say "loop:" i "inner"
    when i = 2 then if i = 2 then say "loop:" i "true"
    otherwise say "loop:" i "other"
  end
end

/* And in a routine, as Expressions.cls has it. */
say "routine:" pick(0) pick(1) pick(2)
exit

pick: procedure
  use arg k
  select case k
    when 0 then return "zero"
    when 1 then if k = 2 then return "inner"
    otherwise nop
  end
  return "fell-through-" || k
