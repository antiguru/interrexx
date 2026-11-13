---
element: "Command handlers"
kind: concept
---

# Command handlers

Native code may add a command environment of its own: a handler that runs the
commands a program sends to that environment's name. What a command is, and
how its return code and conditions reach the program, is stated for every
environment on [the command page](../instructions/command.md) and [the
command environments page](../environment/commands.md); this page adds what
belongs to a handler that native code adds. The examples use [the fixture
library `specfixture`](index.md#the-fixture-libraries), whose `FxAddHandler`
adds the fixture's handler under a name.

> **Note.** The exits that change how an interpreter does its own work, such as
> reading input or writing output, are given by the program that creates the
> interpreter, and a program run by `rexx` has none that native code can add.
> They have no page here.

r[native.handlers.add]
An environment that native code adds is a command environment of the program
from then on: [ADDRESS](../instructions/address.md) reaches it by its name in
any case, and [`CommandHandler`](../classes/commandhandler.md) lists it, in
capitals, as a [`NativeCommandHandler`](r:class.nativecommandhandler.creation)
of the kind `DIRECT`. A command sent to a name that no environment has raises
FAILURE with the return code `30`, as [the names
rule](r:env.commands.names) says.

```rexx
say .commandhandler~environments~hasIndex('N4ZQ')
call fxaddhandler 'n4zq'
say .commandhandler~environments~hasIndex('N4ZQ')
h = .commandhandler~handler('N4ZQ')
say h~class~id h~name h~kind
address N4ZQ 'rc 3'
say rc
::requires 'specfixture' LIBRARY
```

```output
0
1
NativeCommandHandler N4ZQ DIRECT
3
```

r[native.handlers.answer]
The handler's answer is the return code: a whole number as it is, a string
that is not a number as it is too, and no answer as `0`.

```rexx
call fxaddhandler 'N4ZQ'
address n4zq
'rc 0';     say rc
'rc -2';    say rc
'text abc'; say rc
rc = 'old'
'none';     say rc
::requires 'specfixture' LIBRARY
```

```output
0
-2
abc
0
```

r[native.handlers.condition]
A handler may raise ERROR or FAILURE for the command, with a return code. The
command's return code is then that code, the condition's description is the
command, and the condition is raised as for a command of any environment, so
a `CALL ON` handler runs after the clause.

```rexx
call fxaddhandler 'N4ZQ'
address n4zq
call on error name e
call on failure name f
'error 5';   say 'after' rc
'failure 7'; say 'after' rc
exit
e: say 'ERROR' condition('d') rc; return
f: say 'FAILURE' condition('d') rc; return
::requires 'specfixture' LIBRARY
```

```output
ERROR error 5 5
after 5
FAILURE failure 7 7
after 7
```

r[native.handlers.context]
The handler receives the name of the environment as the program names it:
the name of an ADDRESS that writes a symbol is in capitals, and the name of an
`ADDRESS VALUE` is as the expression gives it. It may set the variables of the
program that sent the command.

```rexx
call fxaddhandler 'N4ZQ'
address n4zq 'address'; say rc
address value 'n4zq'
'address'; say rc
'var hello'; say rc cmdvar
::requires 'specfixture' LIBRARY
```

```output
N4ZQ
n4zq
0 hello
```
