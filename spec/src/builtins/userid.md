---
element: USERID
kind: builtin
syntax: "USERID()"
rexxref: "Functions > Built-in Functions > USERID"
---

# USERID

USERID answers the user name of the account that the program runs as. It
takes no argument and changes nothing.

The name is the login name of the account, as the system's account database
records it for the user the process runs as. On a Unix-like system that is the
name the `id -un` command prints. Which name that is depends on whoever runs
the program, so no rule fixes a value.

> **Note.** The examples state the answer through a comparison with the name
> that the `id` command prints, and through its independence of the
> program's environment. They fix no name.

## Arguments

r[bif.userid.args]
USERID takes no argument. A call with an argument raises `SYNTAX 40.4`, by the
[call rules](r:bif.index.call.too-many). A trailing omitted place adds nothing
to the count, so `USERID(,)` is a call without an argument. USERID does not
run as a [String method](r:class.string.builtins): sending the message
`USERID` to a string raises `SYNTAX 97.1`.

```rexx
signal on syntax name a
say userid(1)
a: say condition('o')~code
signal on syntax name b
say userid(, 'x')
b: say condition('o')~code
signal on syntax name c
say 'x'~userid
c: say condition('o')~code
signal off syntax
say userid(,) == userid()
```

```output
40.4
40.4
97.1
1
```

## The result

r[bif.userid.name]
The answer is the login name of the account the process runs as, which is
what the `id -un` command prints. It has no blanks around it, and it is
not the null string. For an account that the system's account database does
not hold, such as one that only a network directory knows, the answer is not
specified.

```rexx
address system 'id -un' with output stem name.
say userid() == name.1
say (userid() == strip(userid())) (userid() \== '')
```

```output
1
1 1
```

r[bif.userid.environment]
For an account the system's account database holds, USERID does not read the
environment: setting the `USER` or `LOGNAME` environment variable, or removing
either of them, does not change the answer.

```rexx
before = userid()
oldUser = value('USER', , 'ENVIRONMENT')
oldLog = value('LOGNAME', , 'ENVIRONMENT')
call value 'USER', 'spec_userid_other', 'ENVIRONMENT'
call value 'LOGNAME', 'spec_userid_other', 'ENVIRONMENT'
address system 'test -z "${USER+set}${LOGNAME+set}"'
say rc (userid() == before)
call value 'USER', .nil, 'ENVIRONMENT'
call value 'LOGNAME', .nil, 'ENVIRONMENT'
address system 'test -z "${USER+set}${LOGNAME+set}"'
say rc (userid() == before)
if oldUser == '' then call value 'USER', .nil, 'ENVIRONMENT'
else call value 'USER', oldUser, 'ENVIRONMENT'
if oldLog == '' then call value 'LOGNAME', .nil, 'ENVIRONMENT'
else call value 'LOGNAME', oldLog, 'ENVIRONMENT'
say (value('USER', , 'ENVIRONMENT') == oldUser) (value('LOGNAME', , 'ENVIRONMENT') == oldLog)
```

```output
1 1
0 1
1 1
```
