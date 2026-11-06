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
A variable symbol has the value that the variable was last assigned. A
variable that has never been assigned, or that has been dropped, is unset,
and the value of its symbol is the name of the symbol, in upper case. [The
SYMBOL function](r:bif.symbol.result) tells the two states apart: it answers
`VAR` for a variable that has a value and `LIT` for one that has none.
Reading an unset variable is what the NOVALUE condition reports when it is
enabled, and the conditions chapter specifies it.

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

r[expr.variables.stem]
A stem symbol names one variable, and its value is that variable's value, or
the name of the symbol with its period, in upper case, when it is unset.
Assigning to a stem symbol also sets the [default](r:expr.variables.default)
of every compound variable of that stem, and discards the values that they
had until then.

```rexx
a.1 = 'one'
say a. a.1
a. = 'all'
say a. a.1 a.2
```

```output
A. one
all all all
```

r[expr.variables.compound]
The variable that a compound symbol names has a name derived from the symbol.
The symbol is a stem, which ends at its first period, and a tail, which is
the rest. The stem is used as written. The tail is split at its periods into
components, and each component that is a symbol starting with a letter, `!`,
`?` or `_` is replaced by the value of the variable of that name, or by its
own name in upper case when that variable is unset. A component that starts
with a digit is not replaced, nor is an empty component, such as the one
that follows a period at the end of the symbol. The name is the stem and the
components joined by periods.

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
characters: `a.n` and `a.x.y` below are one variable, because `x` and `y` are
unset and stand for `X` and `Y`.

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
```

```output
upper upper lower
three A.03 A.03
dotted dotted
A.1 blanks
```

r[expr.variables.default]
A compound variable that is unset has the value of its stem variable when
that one has a value, and the derived name otherwise. A value assigned to a
stem symbol is in this way the default of every compound variable of the
stem. A compound variable that has been assigned keeps its value until the
stem is assigned again.

```rexx
f. = 'dflt'
f.1 = 'one'
k = 'key'
say f.1 f.2 f.k f.k.k
g.1 = 'one'
say g.1 g.2
```

```output
one dflt dflt dflt
one G.2
```

r[expr.variables.separate]
The simple symbol `a`, the stem symbol `a.` and the compound symbols with the
stem `a.` name different variables, and assigning to the stem symbol
changes the compound variables and the stem and leaves the simple variable
as it was.

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
