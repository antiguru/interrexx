---
element: TRACE
kind: builtin
syntax: "TRACE([option])"
ansi: "9.5.9"
rexxref: "Functions > Built-in Functions > TRACE"
---

# TRACE

TRACE answers the trace setting in effect where it is called, and when it is
given an option it changes the setting for the code that follows. The answer
is the setting from before the call. This page specifies the setting and how a
call changes it; what the interpreter writes while a setting is in effect is
not part of it.

## Arguments

r[bif.trace.args]
TRACE takes one optional argument, `option`, a [string](r:conv.string.null).
The argument count is checked by the [builtin call
rules](r:bif.index.call.count): none is required and one is the most, so a
call with two raises `SYNTAX 40.4`. A call with no argument, or with only an
[omitted](r:conv.arg.omitted) one, changes nothing and only answers the
setting. TRACE does not run as a String method: sending the message `TRACE`
to a string raises `SYNTAX 97.1`.

```rexx
call show
exit
show: procedure
say trace() trace(,) trace()
signal on syntax name a
say trace('N', 'O')
a: say condition('o')~code
signal on syntax name b
say 'N'~trace
b: say condition('o')~code
```

```output
N N N
40.4
97.1
```

## The setting

r[bif.trace.setting]
A setting is one of nine letters: `A`, `C`, `E`, `F`, `I`, `L`, `N`, `O` and
`R`. The answer of a call is the setting in effect before the call, as one
uppercase letter, with a question mark before it when interactive tracing is
on, as [its rule](r:bif.trace.prefix) says. A program starts with `N`.

```rexx
call show
exit
show: procedure
say trace()
say trace('R') trace('L')
say trace('C') trace()
call trace 'O'
```

```output
N
N R
L C
```

r[bif.trace.letter]
Of `option` after its prefix, only the first character matters. It selects the
setting by the letter it is, in either case, and the characters after it play
no part, a question mark among them, so the name of a setting, or a word that
starts with its letter, does as well. When `option` is the null string, the
setting is `N`; an option of only question marks is the case of [the prefix
rule](r:bif.trace.prefix).

```rexx
call show
exit
show: procedure
say trace('Alpha') trace('c') trace('Ox') trace('r') trace('n5')
say trace('Results') trace('')
say trace('F') trace('?') trace('')
say trace()
```

```output
N A C O R
N R
N F ?F
N
```

```rexx
call show
exit
show: procedure
say trace('N?') trace('a?') trace('?N?') trace()
say trace('N??') trace('??N?') trace()
```

```output defect=#80
N N A ?N
?N N N
```

r[bif.trace.prefix]
Interactive tracing is on or off beside the letter, and `option` can change it
with question marks before the letter. When there is a letter, the tracing is
on afterwards if the question marks are an odd number, and off if they are an
even number, none included; a call that gives a letter without a question mark
therefore turns it off. The letter `O` always turns it off. When
`option` is only question marks, the letter stays and each question mark flips
whether tracing is on. Where the setting is `O`, only question marks leave it
`O`.

```rexx
call show
exit
show: procedure
say trace('?N') trace('?N') trace('??N') trace('???N') trace('N') trace()
say trace('?') trace('?') trace('??') trace('???') trace('?O') trace('?')
say trace()
```

```output
N ?N ?N N ?N N
N ?N N N ?N O
O
```

r[bif.trace.instruction]
While interactive tracing is on, the TRACE instruction changes nothing; the
TRACE function does, and a call that turns interactive tracing off lets the
instruction work again. Without interactive tracing, the instruction and the
function change the same setting.

```rexx
call show
exit
show: procedure
trace results
say trace() trace('?A')
trace normal
say trace()
say trace('N') trace()
trace results
say trace()
trace off
say trace()
```

```output
R R
?A
?A N
R
O
```

## Scope

r[bif.trace.scope]
The setting belongs to the code that is running, not to the program. An
internal routine, one that the program calls by its label, starts with the
caller's setting, and the caller has its own back when the routine returns,
whatever the routine set. A method, and a `::ROUTINE`, starts with `N`
whatever the caller's setting is, and its setting is lost on return as well.
Code run by INTERPRET shares the setting of the code that runs it.

```rexx
call trace 'E'
call plain
say 'caller' trace()
call hidden
say 'caller' trace()
say .o~new~m 'caller' trace()
say rr() 'caller' trace()
interpret 'x = trace("F")'
say 'interpret' trace()
call trace 'O'
exit
plain:
say 'plain' trace()
call trace 'R'
say 'plain' trace()
return
hidden: procedure
say 'hidden' trace()
call trace 'I'
say 'hidden' trace()
call trace 'O'
return
::class o
::method m
x = trace()
call trace 'F'
return x trace()
::routine rr
x = trace()
call trace 'F'
return x trace()
```

```output
plain E
plain R
caller E
hidden E
hidden I
caller E
N F caller E
N F caller E
interpret F
```

## Errors

r[bif.trace.error]
When `option` is not valid, TRACE raises `SYNTAX 24.1`, and the setting is as
it was. An option is valid when it is the null string, or when, after any
question marks at its start, its first character is a letter of the
[setting](r:bif.trace.setting) in either case. Any other first character is
invalid, among them a blank, a digit, a sign, another letter, a character that
is not a letter in ASCII, a null character and a further question mark after a
blank. An option that is only question marks is valid. An object other than a
string is replaced by a string, as [a string argument](r:conv.string.object)
says.

```rexx
call trace 'A'
call trace 'O'
call show
exit
show: procedure
call trace 'R'
say try(' n') try('x') try('5') try('-1') try('=') try('?x') try('? n')
say try('é') try('?' || '00'x) try(.nil) try('N' || '00'x) try(.array~new)
say trace()
call trace 'O'
exit
try: procedure
signal on syntax name e
x = trace(arg(1))
call trace 'R'
return 'ok'
e: return condition('o')~code
```

```output
24.1 24.1 24.1 24.1 24.1 24.1 24.1
24.1 24.1 24.1 ok ok
R
```

> **Note.** The ANSI draft (9.5.9) flips the interactive tracing once for each
> question mark in front of a letter, from whatever it was; ooRexx sets it from
> whether the count is odd, and the rule follows ooRexx. The reference manual
> allows a number in the TRACE instruction and not in the function, which
> matches the rule above.
