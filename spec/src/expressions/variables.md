---
element: "Variables"
kind: concept
ansi: "7.3, 7.3.1"
rexxref: "General Concepts > Terms, Expressions, and Operators > Terms and Expressions"
---

# Variables

A variable is a named place that holds a string, or any other object. A
program names it with a variable symbol, of which
[the lexical rules](r:lex.symbols.variable) distinguish three forms: the
simple symbol, the stem symbol and the compound symbol. A simple symbol and a
stem symbol name one variable each, and a compound symbol names the variable
whose name the program works out when the symbol is evaluated. This page gives
the value of each form, with the value of one that nothing has been assigned
to, and the way a compound symbol becomes a name.

r[expr.variables.value]
A variable symbol has the value that the variable was last assigned. A simple
or compound variable that has never been assigned, or that has been dropped,
is unset, and the value of its symbol is the name of the symbol, in upper
case, or for a compound symbol its [derived name](r:expr.variables.compound).
[The SYMBOL function](r:bif.symbol.result) tells the two states apart: it
answers `VAR` for a variable that has a value and `LIT` for one that has none.
Reading an unset variable is what the [NOVALUE](r:cond.novalue.raise)
condition reports when it is enabled, and [the conditions
chapter](../conditions/index.md) specifies it. A stem symbol is never unset in
this sense, because its value is a `Stem` in every state, as [the stem
rule](r:expr.variables.stem) says: reading one that was never assigned raises
no NOVALUE, and what SYMBOL answers for a stem symbol, which depends on
whether it has been referred to, is specified with [the
function](r:bif.symbol.stem).

```rexx
say foo symbol('foo')
foo = 'bar'
say foo symbol('foo')
drop foo
say foo symbol('foo')
```

```output
FOO LIT
bar VAR
FOO LIT
```

```rexx
signal on novalue name trap
say symbol('s.')
x = s.
say x symbol('s.')
drop d.
say symbol('d.')
say e.
say e.1
trap: say 'NOVALUE' condition('d')
```

```output
LIT
S. VAR
VAR
E.
NOVALUE E.1
```

r[expr.variables.stem]
A stem symbol names one variable, and the value of the symbol is a `Stem`
object in every state. The string value of that object is the default
assigned to the stem, and the name of the symbol with its period, in upper
case, when none was. Assigning a value that is not a `Stem` to a stem symbol
makes a new `Stem` with that value as its default, which discards the values
that the compound variables of the stem had until then and sets their
[default](r:expr.variables.default). Assigning a `Stem` to a variable keeps
that object, so a stem symbol assigned to another stem symbol shares the
compound variables of one `Stem` with it, and the string value of a shared
`Stem` is the name it was created with. A simple variable that holds a `Stem`
is not changed by a later assignment to the stem symbol it came from.

```rexx
a.1 = 'one'
say a. a.1
a. = 'all'
say a. a.1 a.2 a.~class~id
z. = 'stem'; z.2 = 'two'
y = z.
z. = 'new'
say y[2] z.2 y z.
q.1 = 'one'
r. = q.
r.2 = 'two'
say q.2 r. (r. == q.)
```

```output
A. one
all all all Stem
two new stem new
two Q. 1
```

r[expr.variables.compound]
The variable that a compound symbol names has a name derived from the symbol.
The symbol has a stem and a tail, [as the lexical
rule](r:lex.symbols.variable) splits it. The stem is used as written. The tail
is split at its periods into components, and each component that is a symbol
starting with a letter, `!`, `?` or `_` is replaced by the value of the
variable of that name, or by its own name in upper case when that variable is
unset. A component that starts with a digit is not replaced, nor is an empty
component, such as the one that follows a period at the end of the symbol. The
name is the stem and the components joined by periods.

```rexx
k = 3; z = 'zed'; !q = 'bang'
a.k = 'three'
a.k.z = 'three and zed'
say a.3 a.k a.1 a.3.zed a.k.z a.3.z
say d.k.q
d.k.k = 'kk'
say d.3.3 d.K.3
e.k. = 'trail'
e.!q = 'bang var'
say e.3. e.!q e.bang
```

```output
three three A.1 A.3.ZED three and zed three and zed
D.3.Q
kk kk
trail bang var E.BANG
```

r[expr.variables.tail]
The value that replaces a tail component is used as it is: its case is not
changed, a number is not put in its normal form, and its blanks and periods
stay. Two tails that differ in any character, even only in case or in the
leading zeros of a number, therefore name different variables. A value that
holds periods names the same variable as a tail written with those
characters, but the value is substituted once: it is not split at its
periods and its components are not replaced again. So `a.n` and `a.x.y`
below are different variables when `x` is set, because the tail `x.y` is
`1.Y`, and the same one when `x` and `y` are unset. A value that is not a
string is replaced by a string as [the conversion rule](r:conv.string.object)
says, as [a concatenation operand](r:expr.concatenation.operands) is, so an
object whose `request` with the argument `STRING` answers a string contributes
that answer and not its
`STRING` value.

```rexx
i = 'bar'; j = '03'; n = 'X.Y'; w = ' 1 '
a.bar = 'upper'
a.i = 'lower'
a.3 = 'three'
a.n = 'dotted'
a.w = 'blanks'
say a.BAR a.bar a.i
say a.3 a.j a.03
say a.x.y a.n
say a.1 a.w
drop x y
say a.x.y a.n
x = 1
say a.x.y a.n
b = .both~new
a.b = 'object'
say a.MADE a.b a.TOLD
::class both
::method makestring
  return 'MADE'
::method string
  return 'TOLD'
```

```output
upper upper lower
three A.03 A.03
dotted dotted
A.1 blanks
dotted dotted
A.1.Y dotted
object object A.TOLD
```

r[expr.variables.default]
A compound variable that has never been assigned has the default of its stem
when the stem has one, and the derived name otherwise. A compound variable
that has been assigned keeps its value until the stem is assigned again or
the variable is dropped. A dropped compound variable has the derived name as
its value, which overrides the default of the stem, until it or its stem is
assigned again.

```rexx
f. = 'dflt'
f.1 = 'one'
k = 'key'
say f.1 f.2 f.k f.k.k
g.1 = 'one'
say g.1 g.2
u. = 'dflt'; u.1 = 'one'; u.2 = 'two'
drop u.1
say u.1 symbol('u.1') u.2 symbol('u.3')
u. = 'again'
say u.1 u.2 symbol('u.1')
```

```output
one dflt dflt dflt
one G.2
U.1 LIT two VAR
again again VAR
```

r[expr.variables.separate]
Assigning to the stem symbol `a.` changes its compound variables and leaves
the simple variable `a`, which [is another
variable](r:lex.symbols.variable), as it was.

```rexx
a = 'simple'
a.b = 'compound'
say a a. a.b
a. = 'stem'
say a a. a.b
```

```output
simple A. compound
simple stem stem
```
