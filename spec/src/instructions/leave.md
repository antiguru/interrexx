---
element: LEAVE
kind: instruction
syntax: "LEAVE [name]"
ansi: "6.3.2.64, 8.3.12"
rexxref: "Keyword Instructions > LEAVE"
---

# LEAVE

LEAVE ends a block early. Control passes to the clause after the END of the
block that it ends, as if the loop had run out of passes, and the control
variable keeps the value it had. Without a name it ends the innermost
repetitive loop that is running, and with a name it ends the block that has
that name, which can be a block further out. This page specifies which blocks
a LEAVE can end, how a name picks one, what runs and what does not, and the
errors. It describes loops by the terms of [DO](do.md), and
[ITERATE](iterate.md) is the instruction that ends only the current pass.

## What it ends

r[instr.leave.effect]
LEAVE ends the block that it selects and every block that is open inside it.
The clause that runs next is the one after the END of the selected block. No
test of the block runs, neither a UNTIL nor a WHILE nor the end test of a
controlled loop, and the control variable is not stepped, so it keeps the
value that the pass had. A COUNTER variable keeps its value too. The clauses
that follow the LEAVE in the block do not run. An IF, a THEN and an ELSE are
not blocks that LEAVE ends, so a LEAVE in one ends the loop around it.

```rexx
do i = 1 to 3
  if i = 2 then leave
  say 'pass' i
end
say 'after' i
do counter c j = 10 to 20 until j = 12
  leave
end
say c j
do a = 1 to 2
  do b = 1 to 3
    if b = 2 then leave
    say a b
  end
end
say a b
do 2
  if 1 then leave
  say 'skipped'
end
say 'out'
```

```output
pass 1
after 2
1 10
1 1
2 1
3 2
out
```

r[instr.leave.trap]
A CALL trap that is waiting when a LEAVE runs is called at the LEAVE, once the
block has been left. A condition that the routine raises is not answered at
that point again: its routine is called after the next clause that ends, as
[the rule for CALL traps](r:cond.traps.call) puts it for any clause. In the
example the routine `h1` is called after the `RAISE` returns, at the line of
the call, `h2` is called at the LEAVE, and `h3`, which `h2` raised, is called
after the clause that follows the loop.

```rexx
call on user c1 name h1
call on user c2 name h2
call on user c3 name h3
do forever
  zr = raiser()
  leave
end
say 'after' zr
say 'end'
exit
raiser: raise user c1 return 5
h1: say 'h1' sigl; raise user c2 return
h2: say 'h2' sigl; raise user c3 return
h3: say 'h3' sigl; return
```

```output
h1 5
h2 6
after 5
h3 8
end
```

## Which block it selects

r[instr.leave.target]
A LEAVE with no name selects the innermost block that is a repetitive loop and
is running. A simple group, which is a DO with no repetitor, and a SELECT are
not loops, so an unnamed LEAVE passes over them and selects the loop around
them, and raises `SYNTAX 28.1` when it has none. A group or a SELECT whose
label a LEAVE names can be left, as can a loop.

```rexx
do 2
  do
    say 'group'
    leave
  end
  say 'skipped'
end
say 'after loop'
do 2
  select
    when 1 then leave
  end
  say 'skipped'
end
say 'after select'
do label g
  say 'in g'
  leave g
  say 'skipped'
end g
select label s
  when 1 then leave s
end
say 'after labels'
call try 'do; leave; end'
call try 'select; when 1 then leave; end'
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
group
after loop
after select
in g
after labels
do; leave; end => 28.1
select; when 1 then leave; end => 28.1
```

r[instr.leave.name]
`LEAVE name` selects the innermost running block whose
[name](r:instr.do.label) is `name`. The name must be a plain symbol, and the
instruction takes nothing after it: a name that is not a symbol raises
`SYNTAX 20.907` and extra data `21.907`. When no running block has the name,
LEAVE raises `SYNTAX 28.3`.

```rexx
do i = 1 to 3
  do j = 1 to 3
    if j = 2 then leave i
    say i j
  end
end
say 'i' i 'j' j
do label outer
  loop k = 1 to 3
    leave OUTER
  end
  say 'skipped'
end
say 'k' k
do label x 2
  do label x 2
    leave x
  end
  say 'between'
end
say 'out'
call try 'do 2; leave x; end'
call try 'do label a i = 1 to 2; leave i; end'
call try 'do i = 1 to 2; leave "i"; end'
call try 'do i = 1 to 2; leave (i); end'
call try 'do i = 1 to 2; leave i j; end'
call try 'do s.k = 1 to 2; leave s.k; end'
call try 'do s.k = 1 to 2; leave s.j; end'
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
1 1
i 1 j 2
k 1
between
between
out
do 2; leave x; end => 28.3
do label a i = 1 to 2; leave i; end => 28.3
do i = 1 to 2; leave "i"; end => 20.907
do i = 1 to 2; leave (i); end => 20.907
do i = 1 to 2; leave i j; end => 21.907
do s.k = 1 to 2; leave s.k; end => ok
do s.k = 1 to 2; leave s.j; end => 28.3
```

## Where it runs

r[instr.leave.active]
A LEAVE can only select a block that is running in the same routine or program
part where the LEAVE stands. A loop that a routine, a method or an INTERPRET
text was called out of is not running for that call, so a LEAVE in the callee
that has no loop of its own raises `SYNTAX 28.1`, and one that names a loop of
the caller raises `28.3`. [The INTERPRET rule](r:instr.interpret.loop) states
the same for the text. A loop inside the callee works as usual.

```rexx
do 1
  call plain
end
do label q 1
  call named
end
call inner
exit
plain:
  signal on syntax name e1
  leave
e1: say 'plain' condition('o')~code
  return
named:
  signal on syntax name e2
  leave q
e2: say 'named' condition('o')~code
  return
inner:
  do 3
    leave
  end
  say 'inner done'
  return
```

```output
plain 28.1
named 28.3
inner done
```

r[instr.leave.error]
A LEAVE outside any block that it could select raises `SYNTAX 28.1`, and a
named one raises `SYNTAX 28.3`. These are raised when the LEAVE runs, so the
clauses before it run, and a LEAVE that no path reaches raises nothing. The
two faults in the form of the clause, a name that is not a symbol (`SYNTAX
20.907`) and data after the name (`SYNTAX 21.907`, whatever the name is and
whether or not a block has it), are found when the program is read instead.
No clause of the program runs then, even one before the LEAVE or a LEAVE that
no path reaches, and the exit code is 236 or 235, as [the rule for a program
that is read](r:lex.index.read-first) says. In the text of an INTERPRET
the same faults are ordinary SYNTAX conditions, as [the name
rule](r:instr.leave.name) shows.

```rexx
say 'before'
if 0 then leave
say 'unreached leave is no error'
leave
```

```output rc=228
before
unreached leave is no error
```

```rexx
say 'before'
if 0 then leave x y
say 'unreached'
```

```output rc=235
```

```rexx
say 'before'
if 0 then leave 'x'
say 'unreached'
```

```output rc=236
```
