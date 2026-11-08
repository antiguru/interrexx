---
element: ITERATE
kind: instruction
syntax: "ITERATE [name]"
ansi: "6.3.2.63, 8.3.11"
rexxref: "Keyword Instructions > ITERATE"
---

# ITERATE

ITERATE ends the current pass of a loop and starts the next one. The rest of
the block does not run for that pass. The loop then does the work that ends a
pass: the step, the tests, and the next pass. Without a name it acts on the
innermost loop that is running, and with a name on the loop that has that
name, which can be one further out. This page specifies what the next pass
begins with, which loops a name can select, and the errors. It describes
loops by the terms of [DO](do.md), and [LEAVE](leave.md) is the instruction
that ends the loop itself.

## What it does

r[instr.iterate.effect]
ITERATE does what the END of the block does. A controlled loop steps its
variable and runs its end tests, a WHILE phrase is tested before the next
pass, and an UNTIL phrase is tested at once, so an ITERATE can end the loop
when the UNTIL holds. A COUNTER variable counts the next pass as usual. If the
selected loop is not the innermost, every block open inside it is closed
first, and the control variables of those blocks keep their values. The
clauses after the ITERATE do not run in the pass that it ends.

```rexx
do i = 1 to 3
  if i = 2 then iterate
  say 'pass' i
end
say 'after' i
do j = 1 to 5 until t(j)
  say 'body' j
  iterate
end
say 'j' j
do counter c k = 1 to 3
  iterate
end
say 'c' c 'k' k
do a = 1 to 2
  do b = 1 to 5
    if b = 2 then iterate a
    say a b
  end
end
say 'a' a 'b' b
exit
t: say 'test' arg(1); return arg(1) = 2
```

```output
pass 1
pass 3
after 4
body 1
test 1
body 2
test 2
j 2
c 3 k 4
1 1
2 1
a 3 b 2
```

## Which loop it selects

r[instr.iterate.target]
An ITERATE with no name selects the innermost running block that is a
repetitive loop, and a simple group and a SELECT are passed over, as [they are
for LEAVE](r:instr.leave.target). With no loop to select it raises
`SYNTAX 28.2`. A name that is the label of a simple group or of a SELECT
raises `SYNTAX 28.5`, since a block that does not repeat has no next pass.

```rexx
do 2
  do
    say 'group'
    iterate
  end
  say 'skipped'
end
do 2
  select
    when 1 then iterate
  end
  say 'skipped'
end
call try 'do; iterate; end'
call try 'select; when 1 then iterate; end'
call try 'do label g; iterate g; end'
call try 'select label s; when 1 then iterate s; end'
call try 'do 2; if 1 then iterate; say "skipped"; end; say "ok"'
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
group
do; iterate; end => 28.2
select; when 1 then iterate; end => 28.2
do label g; iterate g; end => 28.5
select label s; when 1 then iterate s; end => 28.5
ok
do 2; if 1 then iterate; say "skipped"; end; say "ok" => ok
```

r[instr.iterate.name]
`ITERATE name` selects the innermost running loop whose name is `name`. The
names are those of [the LABEL rule](r:instr.do.label). When no running loop
has the name, ITERATE raises `SYNTAX 28.4`. A name that is not a symbol raises
`SYNTAX 20.908`, and data after the name `SYNTAX 21.908`. Those two are
found when the program is read, as [the LEAVE rule](r:instr.leave.error) says
of the same faults: no clause runs, and the exit codes are 236 and 235.

```rexx
do label outer i = 1 to 2
  do j = 1 to 3
    if j = 2 then iterate OUTER
    say i j
  end
end outer
do x = 1 to 2
  do x = 7 to 8
    iterate x
  end
  say 'x' x
end
call try 'do 2; iterate nosuch; end'
call try 'do label a i = 1 to 2; iterate i; end'
call try 'do i = 1 to 2; iterate "i"; end'
call try 'do i = 1 to 2; iterate i j; end'
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
2 1
x 9
do 2; iterate nosuch; end => 28.4
do label a i = 1 to 2; iterate i; end => 28.4
do i = 1 to 2; iterate "i"; end => 20.908
do i = 1 to 2; iterate i j; end => 21.908
```

```rexx
say 'before'
if 0 then iterate x y
say 'unreached'
```

```output rc=235
```

```rexx
say 'before'
if 0 then iterate 'x'
say 'unreached'
```

```output rc=236
```

## Where it runs and its errors

r[instr.iterate.active]
An ITERATE can only select a loop that is running in the same part of the
program where it stands, in the way [LEAVE](r:instr.leave.active) states. A
loop that a routine or an INTERPRET text was called out of is not running for
the callee. An ITERATE with no name there raises `SYNTAX 28.2`, and one with a
name raises `28.4`. The error is raised when the clause runs, so a clause that
no path reaches raises nothing.

```rexx
do 1
  call plain
end
do label q 1
  call named
end
signal on syntax name e3
do 1
  interpret 'iterate'
end
e3: say 'text' condition('o')~code
say 'before'
if 0 then iterate
exit
plain:
  signal on syntax name e1
  iterate
e1: say 'plain' condition('o')~code
  return
named:
  signal on syntax name e2
  iterate q
e2: say 'named' condition('o')~code
  return
```

```output
plain 28.2
named 28.4
text 28.2
before
```
