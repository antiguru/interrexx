---
element: SELECT
kind: instruction
syntax: "SELECT [LABEL name] [CASE expression]; WHEN expression [, expression]... [;] THEN instruction ... [OTHERWISE [instructions]] END [name]"
ansi: "6.3.2.17, 8.3.24"
rexxref: "Keyword Instructions > SELECT"
deviations: [39]
---

# SELECT

SELECT chooses one of several alternatives. The block holds a sequence of
WHEN clauses, each with a test and one instruction, and an optional OTHERWISE
part. The tests run in order and the instruction of the first true one runs,
after which control leaves the block at its END. In the second form, SELECT
CASE, the tests compare a value that SELECT computes once. This page
specifies the order of evaluation, what a test must be, the CASE comparison,
the label of the block and the errors. The two-way choice is [IF](if.md).

## What it does

r[instr.select.run]
SELECT evaluates the expressions of its WHEN clauses from the first to the
last, and it stops at the first one that is true. The instruction after the
THEN of that clause runs, and control then passes to the clause after the END
of the SELECT, so the WHEN clauses after it are not evaluated and the
OTHERWISE part does not run. THEN may begin a later clause than the
expression of its WHEN, as [the rule for THEN](r:instr.if.then) says, and the
instruction after each THEN follows [the rules of IF](r:instr.if.branch) for
null clauses and for the implied semicolons after THEN.

```rexx
n = 7
select
  when n < 5 then say 'small'
  when n < 10 then say 'medium'
  when f(n) then say 'never evaluated'
  otherwise say 'large'
end
say 'after'
exit
f: say 'f called'; return 1
```

```output
medium
after
```

## The test of a WHEN

r[instr.select.when]
In a SELECT without CASE, the expression of a WHEN must have the value `0` or
`1`, as [the value of an IF](r:instr.if.value) must, and any other value
raises `SYNTAX 34.2`. A WHEN can hold a list of expressions separated by
commas, and the list is true and is evaluated as [the list of an
IF](r:instr.if.list) is. An expression of the list that is evaluated and
is not `0` or `1` raises `SYNTAX 34.6`. The error is raised when the WHEN is
reached, so a WHEN that comes after a true one is never evaluated and cannot
raise it.

```rexx
call try 'select; when 2 then nop; end'
call try 'select; when 1, 5 then nop; end'
call try 'select; when 0, 5 then nop; otherwise say "short"; end'
call try 'select; when 1, f(1), f(1) then say "all"; end'
call try 'select; when 1 then say "first"; when 2 then nop; end'
call try 'select; when 0 then nop; when "a" then nop; end'
exit
f: say 'f('arg(1)')'; return arg(1)
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
select; when 2 then nop; end => 34.2
select; when 1, 5 then nop; end => 34.6
short
select; when 0, 5 then nop; otherwise say "short"; end => ok
f(1)
f(1)
all
select; when 1, f(1), f(1) then say "all"; end => ok
first
select; when 1 then say "first"; when 2 then nop; end => ok
select; when 0 then nop; when "a" then nop; end => 34.2
```

## OTHERWISE

r[instr.select.otherwise]
The OTHERWISE part runs when no WHEN was true. It holds any number of
instructions up to the END, and it may hold none, in which case a SELECT with
no true WHEN does nothing. A SELECT that has no OTHERWISE part and no true
WHEN raises `SYNTAX 7.3` when the END is reached, which is when the program
runs and not when it is read, so the clauses before it have run. OTHERWISE is
the last part of the block, and a WHEN after it raises `SYNTAX 9.1`, and a
second OTHERWISE raises `9.2`, when the program is read.

```rexx
select
  when 0 then say 'no'
  otherwise
    say 'o1'
    say 'o2'
end
select
  when 0 then say 'no'
  otherwise
end
say 'empty otherwise is fine'
call try 'select; when 0 then say "no"; end'
call try 'select; when 1 then nop; otherwise nop; when 1 then nop; end'
call try 'select; when 0 then nop; otherwise nop; otherwise nop; end'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
o1
o2
empty otherwise is fine
select; when 0 then say "no"; end => 7.3
select; when 1 then nop; otherwise nop; when 1 then nop; end => 9.1
select; when 0 then nop; otherwise nop; otherwise nop; end => 9.2
```

## SELECT CASE

r[instr.select.case]
SELECT CASE evaluates its expression once, before the first WHEN, and each
WHEN then holds one expression or a comma-separated list of them. The WHEN is
true when the strict comparison `==` of the case value with one of its
expressions is true, where the case value is the receiver and the expression
the argument, and the list is evaluated from left to right and stops at the
first true comparison. The comparison is the strict one, so `1` does not equal
`1.0`, a blank counts, and the case of letters counts. The expressions need
not be `0` or `1`, and no value of a WHEN raises `SYNTAX 34.2`, so a case that
matches no WHEN raises `SYNTAX 7.3` as above. When the case value is an
object, the `==` message goes to it, and its answer must be `0` or `1`: any
other answer, such as the text `banana` or the number `2`, raises
`SYNTAX 34.905`.

```rexx
x = 'b'
select case x
  when 'a' then say 'is a'
  when 'c', 'b' then say 'is b or c'
  otherwise say 'other'
end
select case 1
  when 1.0 then say 'numeric equal'
  when 1 then say 'strict equal'
end
select case 'a '
  when 'a' then say 'blank ignored'
  when 'A', 'a ' then say 'blank counts, case counts'
end
select case f(2)
  when f(1), f(2), f(3) then say 'hit'
  otherwise nop
end
select case .probe~new('A')
  when .probe~new('B') then say 'matched B'
  when .probe~new('A') then say 'matched A'
end
call try 'select case 3; when 1, 2 then nop; end'
call try 'select case .bad~new; when 1 then nop; end'
call try 'select case .num~new; when 1 then nop; end'
exit
f: say 'f('arg(1)')'; return arg(1)
try:
  signal on syntax name e
  interpret arg(1)
e: say arg(1) '=>' condition('o')~code
  return
::class probe
::method init
  expose n
  use arg n
::attribute n get
::method '=='
  expose n
  use arg other
  say 'receiver' n 'argument' other~n
  return n = other~n
::class bad
::method '=='
  return 'banana'
::class num
::method '=='
  return 2
```

```output
is b or c
strict equal
blank counts, case counts
f(2)
f(1)
f(2)
hit
receiver A argument B
receiver A argument A
matched A
select case 3; when 1, 2 then nop; end => 7.3
select case .bad~new; when 1 then nop; end => 34.905
select case .num~new; when 1 then nop; end => 34.905
```

r[instr.select.case.syntax]
The expression after CASE is required, and so is the expression of every WHEN
in a SELECT CASE, including each one that follows a comma. A missing case
expression raises `SYNTAX 35.933`, and a missing expression in a WHEN
`35.934`. Both are found when the program is read. A SELECT with a token after
it that is not LABEL or CASE raises `25.923`. The word LABEL has to come
before CASE: after the CASE expression, the word is part of that expression.

```rexx
call try 'select case ; when 1 then nop; end'
call try 'select case 1; when then nop; end'
call try 'select case 1; when 1, then nop; end'
call try 'select case 1; when , 1 then nop; end'
call try 'select 1; when 1 then nop; end'
call try 'select label a case 1; when 1 then say "ok"; end'
call try 'select case 1 label a; when 1 then say "ok"; end'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
select case ; when 1 then nop; end => 35.933
select case 1; when then nop; end => 35.934
select case 1; when 1, then nop; end => 35.934
select case 1; when , 1 then nop; end => 35.934
select 1; when 1 then nop; end => 25.923
ok
select label a case 1; when 1 then say "ok"; end => ok
select case 1 label a; when 1 then say "ok"; end => 7.3
```

## The label of a block

r[instr.select.label]
A LABEL phrase gives the block a name that LEAVE can use, whether or not the
SELECT has a CASE. [LEAVE name](r:instr.leave.name) ends the block from any
depth inside it. An END after the block may repeat the name, which is compared
without regard to case, and a different name raises `SYNTAX 10.4` and any name
on the END of a SELECT that has no LABEL raises `10.7`. Both are found when
the program is read. An unnamed LEAVE or ITERATE does not select a SELECT,
because it is not a loop, as [the LEAVE rule](r:instr.leave.target) says.

```rexx
select label pick
  when 1 then do
    say 'in'
    leave PICK
    say 'skipped'
  end
  otherwise say 'not run'
end pick
say 'after'
call try 'select label a; when 1 then nop; end A'
call try 'select label a; when 1 then nop; end b'
call try 'select; when 1 then nop; end b'
call try 'select; when 1 then nop; end 5'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
in
after
select label a; when 1 then nop; end A => ok
select label a; when 1 then nop; end b => 10.4
select; when 1 then nop; end b => 10.7
select; when 1 then nop; end 5 => 10.7
```

## What the block holds

r[instr.select.body]
Between the SELECT and its OTHERWISE or END, the only clauses are WHEN
clauses with their instruction, and null clauses. A label there raises
`SYNTAX 47.4` when the program is read, as [the label
rule](r:lex.clauses.label) says. The instruction after a THEN
is one instruction, which may be a DO group, an IF or another SELECT, so a
second instruction raises `SYNTAX 7.2` when the program is read. In an IF in
the instruction, an ELSE that follows pairs with that IF, as [the ELSE
rule](r:instr.if.else) says, and in a nested SELECT the OTHERWISE and the END
belong to the nearest SELECT that is still open.

```rexx
select
  when 1 then if 0 then say 'a'
  else say 'if else'
  otherwise say 'outer otherwise'
end
select
  when 1 then
    select
      when 0 then say 'inner'
      otherwise say 'inner otherwise'
    end
  otherwise say 'outer otherwise'
end
select
  when 1 then do
    say 'g1'
    say 'g2'
  end
end
```

```output
if else
inner otherwise
g1
g2
```

## Errors

r[instr.select.syntax]
Errors in the shape of a SELECT are found when the program is read, so a
program that has one runs none of its clauses. A SELECT with no WHEN before
its OTHERWISE or END raises `SYNTAX 7.1`, and another clause where a WHEN is
required raises `7.2`. A WHEN or an OTHERWISE outside a SELECT raises `9.1`
and `9.2`. A WHEN with no THEN raises `18.2`, and an END where the instruction
of a THEN should be raises `10.1`. A SELECT with no END raises `14.2`
and one whose OTHERWISE part has no END `14.901`. A stray END raises `10.1`.

```rexx
say 'before'
select
  when 1 then say 'a'
  say 'second instruction'
end
```

```output rc=249
```

```rexx
call try 'select; end'
call try 'select; otherwise say 1; end'
call try 'select; say 1; end'
call try 'select; when 1 then say "a"; say "b"; end'
call try 'do; when 1 then say "a"; end'
call try 'otherwise say 1'
call try 'select; when 1 say "a"; end'
call try 'select; when 1 then end'
call try 'select; when 1 then say 1'
call try 'select; when 1 then say 1; otherwise say 2'
call try 'select; when 1 then say 1; end; end'
call try 'select; when 1 then say 1; else say 2; end'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
select; end => 7.1
select; otherwise say 1; end => 7.1
select; say 1; end => 7.2
select; when 1 then say "a"; say "b"; end => 7.2
do; when 1 then say "a"; end => 9.1
otherwise say 1 => 9.2
select; when 1 say "a"; end => 18.2
select; when 1 then end => 10.1
select; when 1 then say 1 => 14.2
select; when 1 then say 1; otherwise say 2 => 14.901
select; when 1 then say 1; end; end => 10.1
select; when 1 then say 1; else say 2; end => 8.2
```

## Deviations

r[instr.select.deviation.39]
A label spelled THEN that stands as a clause of its own after the expression
of a WHEN raises `SYNTAX 18.2`, because the WHEN is still waiting for its
THEN. ooRexx raises `35.1` at the colon. When the `then:` follows the
expression in the same clause, both raise `35.1`.

```rexx
call try 'select; when 1 then: say 1; end'
call try 'select; when 1; then: say 1; end'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: say arg(1) '=>' condition('o')~code
  return
```

```output
select; when 1 then: say 1; end => 35.1
select; when 1; then: say 1; end => 18.2
```
