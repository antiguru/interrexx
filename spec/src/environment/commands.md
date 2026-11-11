---
element: "Command environments"
kind: concept
rexxref: "General Concepts > Commands to External Environments > Environment; Useful Services > The RXQUEUE Filter"
deviations: [11, 93, 146, 147, 148, 149]
---

# Command environments

A command that a program sends goes to an environment, which runs it and
answers with a return code. [The commands page](r:instr.command.send)
specifies what is sent and how `RC` and `.RS` are set, [the ADDRESS
instruction](../instructions/address.md) chooses the environment and
redirects its streams, and [the conditions chapter](../conditions/error.md)
specifies ERROR and FAILURE. This page specifies the environments that the
interpreter has on Linux and what they do with a command: which program runs
it and with what, the return codes they answer, what the command starts with,
the commands that the interpreter does itself, and the `rxqueue` filter
through which a command puts lines on the program's queue.
[`CommandHandler`](../classes/commandhandler.md) and
[`NativeCommandHandler`](../classes/nativecommandhandler.md) are the classes
of the environments, and a handler written in Rexx gets
[`CommandContext`](../classes/commandcontext.md),
[`CommandInput`](../classes/commandinput.md) and
[`CommandOutput`](../classes/commandoutput.md) objects for a command that it
runs.

The examples start commands that every Linux host has: the shell's own
commands, `echo`, `printf`, `cat` and `cd`. An example that changes the
current directory, `HOME`, `PATH` or the queue puts it back.

## The environments

r[env.commands.names]
A program has eleven command environments, which
[`CommandHandler~environments`](r:class.commandhandler.environments) lists
under these names: `SYSTEM`, `COMMAND`, `SH`, `BASH`, `BSH`, `KSH`, `CSH`,
`TCSH`, `ZSH`, `PATH` and the null string. They are all
[`NativeCommandHandler`](r:class.nativecommandhandler.creation) objects. The
case of a name plays no part ([the setting rule](r:instr.address.with.set)).
A command sent to any other name raises FAILURE with the return code `30`
([the status rule](r:instr.command.status)), and sets nothing that its
redirections name.

```rexx
do n over .commandhandler~environments~allIndexes~sort
  say '['n']' .commandhandler~handler(n)~kind
end
signal on failure name f
address value 'a4zq_none'
'echo not run'
f: say condition('c') rc .rs
```

```output
[] REDIRECTING
[BASH] REDIRECTING
[BSH] REDIRECTING
[COMMAND] REDIRECTING
[CSH] REDIRECTING
[KSH] REDIRECTING
[PATH] REDIRECTING
[SH] REDIRECTING
[SYSTEM] REDIRECTING
[TCSH] REDIRECTING
[ZSH] REDIRECTING
FAILURE 30 -1
```

r[env.commands.initial]
A program starts with the environment `sh`, written in lower case, whose
name is what the function `ADDRESS()` answers until an ADDRESS instruction
names another one, which then answers its name as that instruction gave it:
in upper case for a symbol and as written for a string ([the name
rule](r:bif.address.name)). A command sent before any ADDRESS instruction is
run as `SYSTEM` runs it, and the instruction `ADDRESS` with no operand
brings the starting environment back ([the initial
rule](r:bif.address.initial)).

```rexx
say address()
address system
say address()
address
say address()
'echo "$0"'
address 'MixEnv'
say address()
```

```output
sh
SYSTEM
sh
/bin/sh
MixEnv
```

## Running a command

r[env.commands.shell]
The environments `SYSTEM`, `COMMAND`, `SH` and the null string run a command
with the program `/bin/sh`, given the two arguments `-c` and the command
string, so that the string is one argument and the shell reads it. `BASH`
does the same with `/bin/bash`, and `BSH`, `KSH`, `CSH`, `TCSH` and `ZSH`
with the program of that name in `/bin`, in lower case. The interpreter waits
for the shell to end and answers its exit status as the return code ([the
return code rule](r:env.commands.rc)). A shell that the host does not have
cannot be run, and the command raises FAILURE with the return code `127`
([the failure rule](r:cond.error.failure)).

```rexx
do n over .array~of('system', 'command', 'sh', '')
  address value n
  'echo "$0"'
end
```

```output
/bin/sh
/bin/sh
/bin/sh
/bin/sh
```

r[env.commands.shell.string]
What the shell does with the string is the shell's business, not the
interpreter's: it reads blanks, quotes, `;`, `|`, `&&`, variables, `$(( ))`,
redirections and a line feed that separates two commands as it does in a
script, and the return code is that of the last command it ran. The
variables of the Rexx program are not variables of the shell, which sees
only the environment variables. The string is not changed, and a string that
holds nothing but blanks runs nothing and answers 0. Which characters the
string may hold beyond those that a string argument of a program can hold is
the host's business, except that [a NUL](r:env.commands.deviation.146) is not
run.

```rexx
address system
x = 'rexx'
'echo "a   b" | cat; x=shell; echo $x $((6 * 7)); printf "[%s]" 1 2 3; echo'
'echo l1' || '0a'x || 'echo l2'
'echo ${x:-unset}'
'   '
say 'blank' rc
```

```output
a   b
shell 42
[1][2][3]
l1
l2
unset
blank 0
```

r[env.commands.deviation.146]
A command string that holds a NUL character is not run in any native
environment: the command raises FAILURE with the return code `127`, and
nothing of the string runs. This is [Deviation
146](../deviations.md#146-a-command-string-with-a-nul-character): ooRexx cuts
the string at the NUL and runs the part before it, so that there the shell
gets only `echo ran; echo "a` from the first command of the example, refuses
it as a syntax error and answers 2 (ERROR, nothing printed), and the second
command runs `printf ran`.

```rexx
signal on failure name f
address system 'echo ran; echo "a' || '00'x || 'b"'
f: say condition('c') rc
signal on failure name g
address path 'printf ran' || '00'x || 'x'
g: say condition('c') rc
```

```output
FAILURE 127
FAILURE 127
```

r[env.commands.path]
The environment `PATH` runs no shell. It cuts the command string into words
as [the words rule](r:env.commands.path.words) says, looks for the program
named by the first word and runs it with the other words as its arguments.
A first word that holds a slash names the program itself; any other is
searched for in the directories of the environment variable `PATH`, as the
variable stands when the command is sent, so that a change by VALUE is seen.
An empty entry of `PATH`, and a `PATH` that is the null string, stand for the
current directory. When `PATH` has no value, the host's default list of
system directories is searched, which does not hold the current directory.
A file that cannot be run, or a directory, is passed over and the search goes
on. A string with no word, a program that is not found, and a program named
with a slash that cannot be run raise FAILURE with the return code `127`. The
interpreter answers the exit status of the program as the return code, the
same way as for a shell.

```rexx
e = 'ENVIRONMENT'
address path
'echo one two'
say rc
old = value('PATH', '/a4zq_nonexistent_dir', e)
signal on failure name f
'echo not found'
f: say condition('c') rc
call off failure
'/bin/sh -c "echo by name; exit 4"'
say rc
call value 'PATH', old, e
'echo found again'
say rc
call off failure
signal on failure name g
'   '
g: say condition('c') rc
```

```output
one two
0
FAILURE 127
by name
4
found again
0
FAILURE 127
```

```rexx
e = 'ENVIRONMENT'
f = 'a4zq_prog'
call lineout f, '#!/bin/sh'
call lineout f, 'echo found'
call lineout f
address system 'chmod +x' f
signal on failure name nf
address path
old = value('PATH', '', e)
f
call value 'PATH', ':/a4zq_none', e
f
call value 'PATH', .nil, e
'printf "default\n"'
f
nf: say condition('c') rc
call value 'PATH', old, e
address system 'chmod -x' f
signal on failure name nx
'./'f
nx: say condition('c') rc
address system 'rm' f
```

```output
found
found
default
FAILURE 127
FAILURE 127
```

r[env.commands.deviation.147]
An executable file that the search of `PATH` finds and that has no `#!` line
is run by `PATH` with the program `/bin/sh`, which reads it as a script, given
the file's path and the other words of the command as its arguments, as a
shell runs such a file. The same file named with a slash is not run
this way: the command raises FAILURE with the return code `127`. This is
[Deviation
147](../deviations.md#147-a-file-without-a--line-that-a-path-search-finds):
ooRexx raises FAILURE with the return code `127` for the file that the search
finds as well, and runs nothing.

```rexx
e = 'ENVIRONMENT'
f = 'a4zq_noshebang'
call lineout f, 'echo "ran with" $#'
call lineout f
address system 'chmod +x' f
old = value('PATH', directory()':'value('PATH', , e), e)
address path
f 'x' '"y  z"'
say rc
signal on failure name nf
'./'f
nf: say condition('c') rc
call value 'PATH', old, e
address system 'rm' f
```

```output
ran with 2
0
FAILURE 127
```

r[env.commands.path.words]
The words of a command for `PATH` are separated by blanks and tabs, and any
number of them separate two words. A word that begins with a double quote
ends at the first double quote that is followed by a blank, a tab or the end
of the string, and the two quotes are not part of it; it may hold blanks, any
other double quote, and line feeds. A word of this kind that is never closed
ends with the string. Every other character belongs to a word as it is, a
single quote, a backslash, a dollar sign, a tilde, a star, `;`, `|`, `>` and
a line feed among them, so that nothing is expanded and no redirection or
pipe exists. A double quote inside a word that does not begin with one is an
ordinary character.

```rexx
address path
q = "'"
'printf [%s] "a  b" c\ d' q'e f'q 'x;y $HOME * ~ >z'
say
'printf [%s] a"b c"d "" "x"y "p q"r" s" "t'
say
'printf [%s] "u' || '0a'x || 'v" w' || '0a'x || 'x'
say
'printf [%s]' || '09'x || 'a' || '09'x || '09'x || 'b'
say
```

```output
[a  b][c\][d]['e][f'][x;y][$HOME][*][~][>z]
[a"b][c"d][][x"y "p q"r][s"][t]
[u
v][w
x]
[a][b]
```

## The return code

r[env.commands.rc]
The return code of a command that a shell or a program ran to its end is its
exit status, a whole number from 0 to 255: an `exit` with another number
gives the number modulo 256. It is set in `RC` as [the return code
rule](r:instr.command.rc) says, and a nonzero one raises
[ERROR](r:cond.error.raise), `127` raising
[FAILURE](r:cond.error.failure). A shell answers the status of the command it
ran last, so a command that the shell could not find is the `127` of the
shell, and which codes other than 127 a shell uses for a command that it
cannot run is the shell's business.

```rexx
address system
do c over .array~of('exit 0', 'exit 7', 'exit 255', 'exit 256', 'exit 300',,
    'true', 'false', 'test -d /a4zq_none', '(exit 4)', 'a4zq_none_cmd')
  c
  say c '->' rc .rs
end
```

```output
exit 0 -> 0 0
exit 7 -> 7 1
exit 255 -> 255 1
exit 256 -> 0 0
exit 300 -> 44 1
true -> 0 0
false -> 1 1
test -d /a4zq_none -> 1 1
(exit 4) -> 4 1
a4zq_none_cmd -> 127 -1
```

r[env.commands.rc.signal]
A shell or a program that a signal ended has no exit status, and its return
code is the number of the signal with a minus sign, `-9` for the signal
`KILL`, `-15` for `TERM` and `-1` for `HUP`. The command raises
[ERROR](r:cond.error.raise), and `.RS` is `1`. A shell that runs a program
which a signal ended answers the status `128` plus the signal number for it,
as a shell does, and that is an exit status. This is the same for all
environments.

```rexx
address system with error stem e.
do c over .array~of('kill -s KILL $$', 'kill -s TERM $$', 'kill -s HUP $$',,
    'sh -c "kill -s KILL \$\$"', 'sh -c "kill -s TERM \$\$"')
  c
  say c '->' rc .rs
end
address path with error stem e.
'sh -c "kill -s TERM $$"'
say 'path' rc .rs
```

```output
kill -s KILL $$ -> -9 1
kill -s TERM $$ -> -15 1
kill -s HUP $$ -> -1 1
sh -c "kill -s KILL \$\$" -> 137 1
sh -c "kill -s TERM \$\$" -> 143 1
path -15 1
```

r[env.commands.rc.deviation.11]
A command starts with `SIGPIPE` at its default action, so a shell that
raises `SIGPIPE` is ended by it, with the return code `-13` and ERROR,
[as the rule above](r:env.commands.rc.signal) says for any signal. This is
[Deviation 11](../deviations.md#11-sigpipe-in-a-commands-child): ooRexx starts
the command with `SIGPIPE` ignored, and the shell that raises it goes on and
answers 0.

```rexx
address system
'kill -s PIPE $$'
say rc .rs
```

```output
-13 1
```

> **Note.** [Deviation 10](../deviations.md#10-signals-halt-and-wake) says
> what the return code is when a halt interrupts the wait for a command, and
> [Deviation 91](../deviations.md#91-a-command-whose-child-was-already-reaped)
> what it is for a command whose process was gone before the wait. A command
> that a program sends to an environment that does not exist answers `30`, as
> [the first rule](r:env.commands.names) says.

## What a command starts with

r[env.commands.start]
A command starts in the current directory of the program as
[DIRECTORY](r:bif.directory.effect) leaves it, with the environment
variables of the program as [VALUE](r:bif.value.os.command) leaves them, and
with the standard input, output and error of the program, unless a
redirection of [the ADDRESS instruction](r:instr.address.with.normal)
replaces them. What the command itself changes, its current
directory, its variables and what it exports, ends with it and is not seen by
the program: [the commands that the interpreter does
itself](r:env.commands.inprocess) are the exception.

```rexx
e = 'ENVIRONMENT'
address system
call directory '/'
'pwd'
call directory '/usr'
'pwd'
call value 'A4ZQ_V', 'set by rexx', e
'echo "[$A4ZQ_V]"'
'A4ZQ_V=changed; export A4ZQ_V; echo "[$A4ZQ_V]"'
say '['value('A4ZQ_V', , e)']'
'cd /; pwd'
say directory()
call value 'A4ZQ_V', .nil, e
call directory '/'
```

```output
/
/usr
[set by rexx]
[changed]
[set by rexx]
/
/usr
```

r[env.commands.deviation.148]
A command has the standard input, output and error of the program as its
descriptors 0, 1 and 2. No descriptor that the interpreter opens for its own
work is open in it. One that the interpreter was started with is passed on as
the host passes it, so that the example, whose second interpreter is started
with none, lists only these three and the `3` that `ls` opens to list them.
This is [Deviation
148](../deviations.md#148-the-descriptors-that-a-command-inherits): ooRexx
leaves two descriptors of its own open in the command, so that the example
lists two more there.

```rexx
f = 'a4zq_fd.rex'
call lineout f, "address system 'echo $(ls /proc/self/fd)'"
call lineout f
address bash
'for d in $(ls /proc/self/fd); do [ "$d" -gt 2 ] && eval "exec $d>&-"; done;',
  'rexx' f
say rc
'rm' f
```

```output
0 1 2 3
0
```

## Commands the interpreter does itself

r[env.commands.inprocess]
A command that the shell would run in a process of its own, and whose effect
would end with that process, is run by the interpreter in the program instead,
so that its effect is the program's. These are the commands `cd`, `export`,
`set` and `unset`, in lower case, when all of the following hold. The string
begins with the word, directly followed by a blank and an argument, or, for
`cd`, by nothing or by blanks only. It has none of the characters `;`, `&`,
`|`, `<` and `>`. It is sent as a command clause or by an ADDRESS instruction
with a command to a native environment, any of those that [the first
rule](r:env.commands.names) names, `PATH` included. Which shell that
environment runs, and whether the host has that shell, plays no part. The
environment may also be reached under another name it was registered with, or
by a handler that forwards the command with
[`command`](r:class.nativecommandhandler.command). No `WITH` applies to it:
its clause has none, and no earlier ADDRESS instruction with `WITH` and no
command has given the environment one, even one that names only `NORMAL`. Any
other string is run as the environment runs every command, by a shell or, for
`PATH`, as a program that is searched for, in a process whose effect on the
program ends with it, so that a leading blank, a tab for the blank, a capital
letter, or a `;` sends the string that way. A command that the interpreter
does itself writes nothing, and its return code is set as the rules of the
four commands say.

```rexx
call directory '/'
address system
'cd /usr'
say directory()
call directory '/'
'cd /usr; true'
say directory()
' cd /usr'
say directory()
'cd' || '09'x || '/usr'
say directory()
'CD /usr'
say rc directory()
address sh 'cd /usr'
say directory()
call directory '/'
address system with output stem o.
'cd /usr'
say rc directory() o.0
address system with output normal
'cd /usr'
say rc directory()
address bash 'cd /usr'
say directory()
call directory '/'
address ksh 'cd /usr'
say directory()
call directory '/'
x = .commandhandler~register('a4zq_alias', .commandhandler~handler('system'))
address a4zq_alias 'cd /usr'
say directory()
x = .commandhandler~deregister('a4zq_alias')
call directory '/'
```

```output
/usr
/
/
/
127 /
/usr
0 / 0
0 /
/usr
/usr
/usr
```

```rexx
e = 'ENVIRONMENT'
call directory '/'
address path 'cd /usr'
say rc directory()
address path 'export A4ZQ_P=1'
say rc '['value('A4ZQ_P', , e)']'
address path 'unset A4ZQ_P'
say rc '['value('A4ZQ_P', , e)']'
call directory '/'
```

```output defect=#219
0 /usr
0 [1]
0 []
```

r[env.commands.cd]
`cd` makes the directory its argument names the current directory of the
program, as [DIRECTORY](r:bif.directory.effect) does, and answers 0. The
argument is the rest of the string after the blanks that follow `cd`; with no
argument the directory is the value of the environment variable `HOME`, read
as any other argument is: unlike [DIRECTORY with `~`](r:bif.directory.home), a
value that does not start with a slash is taken from the current directory.
When the argument as written begins with `~`, the tilde is read before
anything else. A `~` alone, or followed by a slash or a blank, stands for the
value of `HOME`, which is then read as the rest of the argument is, its double
quotes and backslashes dropped, but not read again for a `~`. After a slash
the rest of the argument follows as usual; after a blank the rest is not read.
A `~` followed by a name, which runs to the first slash or to the end of the
argument, stands for the home directory of the user of that name in the host's
user database. When there is no such user, the command is not done by the
interpreter: the string goes to the shell, as any other string does. A `~`
that is not the first character of the argument as written, and one that a
quote or backslash comes before, is an ordinary character. In the rest of the
argument every double quote is dropped, even one that nothing closes, and so
is a backslash, the character after it being kept. Nothing else is read: a
blank at the end is part of the name, as is a `-`, a single quote, a dollar
sign or a star. A name that does not start with a slash is taken from the
current directory. When `HOME` is the null string, `cd` with no argument fails
as below, and a `~` that stands for `HOME` stands for nothing, so that `cd ~`
and `cd ~/` make `/` current and `cd ~/x` makes `/x` current. When the
directory cannot be made current, the command answers the number that the host
gives the error, as Linux numbers them: `2` when there is no such directory,
`20` when a part of the name is not a directory, `13` when permission is
refused. It raises [ERROR](r:cond.error.raise) as every nonzero return code
does, and leaves the current directory as it was.

```rexx
e = 'ENVIRONMENT'
old = value('HOME', '/usr', e)
call directory '/'
address system
do c over .array~of('cd', 'cd /', 'cd ~', 'cd ~/..', 'cd ~ /etc', 'cd  usr',,
    'cd "/usr"', 'cd /u"s"r', 'cd \/usr', 'cd /usr/', 'cd /usr ',,
    "cd '/usr'", 'cd $HOME', 'cd -', 'cd "~"', 'cd /a4zq_none',,
    'cd /dev/null', 'cd ""', 'cd /usr lib')
  call directory '/'
  c
  say '['c']' rc .rs directory()
end
call directory '/'
'cd ~a4zq_nouser'
say directory()
address system 'cd ~root 2>/dev/null && pwd -P' with output stem h.
'cd ~root'
say ((rc = 0) == (h.0 = 1)) (rc \= 0 | directory() == h.1)
call directory '/'
call value 'HOME', '', e
'cd ~'
say rc directory()
call directory '/'
call value 'HOME', old, e
```

```output
[cd] 0 0 /usr
[cd /] 0 0 /
[cd ~] 0 0 /usr
[cd ~/..] 0 0 /
[cd ~ /etc] 0 0 /usr
[cd  usr] 0 0 /usr
[cd "/usr"] 0 0 /usr
[cd /u"s"r] 0 0 /usr
[cd \/usr] 0 0 /usr
[cd /usr/] 0 0 /usr
[cd /usr ] 2 1 /
[cd '/usr'] 2 1 /
[cd $HOME] 2 1 /
[cd -] 2 1 /
[cd "~"] 2 1 /
[cd /a4zq_none] 2 1 /
[cd /dev/null] 20 1 /
[cd ""] 2 1 /
[cd /usr lib] 2 1 /
/
1 1
0 /
```

r[env.commands.export]
`export` and `set`, given an argument, set an environment variable of the
program, as VALUE does ([the variables rule](r:bif.value.os)), and answer 0.
The name is the text from the blanks that follow the word to the first `=`,
and the value is all that follows that `=`, to the end of the string, blanks
and quotes included. A string without `=` sets nothing, and neither does one
whose name is empty, `export =3` for one, which answers 0 ([the deviation
rule](r:env.commands.deviation.149)). Within the value each `$` starts a
reference: the characters after it up to the next `/`, `:` or `$`, or the end
of the string, are the name of an environment variable, and the reference is
replaced by the value of that variable, or by nothing when there is none; the
character that ended the name stays. Neither the name nor the reference
follows the rules of the shell, so that `${NAME}` is a reference to a variable
whose name has braces in it. A `~` in the value is an ordinary character.
`export` and `set` with no argument are run by the shell.

```rexx
e = 'ENVIRONMENT'
address system
call value 'A4ZQ_H', 'hval', e
call t 'export A4ZQ_X1=1', 'A4ZQ_X1'
call t 'set A4ZQ_X2=2', 'A4ZQ_X2'
call t 'export  A4ZQ_X3=a  b ', 'A4ZQ_X3'
call t 'export A4ZQ_X4="q r"', 'A4ZQ_X4'
call t 'export A4ZQ_X5=7 A4ZQ_Y5=8', 'A4ZQ_X5 A4ZQ_Y5'
call t 'export A4ZQ_X6', 'A4ZQ_X6'
call t 'export A4ZQ_X8=$A4ZQ_H/s:t', 'A4ZQ_X8'
call t 'export A4ZQ_X9=<$A4ZQ_H$A4ZQ_H>', 'A4ZQ_X9'
call t 'export A4ZQ_X10=${A4ZQ_H}', 'A4ZQ_X10'
call t 'export A4ZQ_X11=u$A4ZQ_nope', 'A4ZQ_X11'
call t 'export A4ZQ_X12=~/x', 'A4ZQ_X12'
call t 'export A4ZQ_X13=1;', 'A4ZQ_X13'
call t 'export A4ZQ_X14=', 'A4ZQ_X14'
'test "${A4ZQ_X14+set}" = set'
say 'set but null' rc
exit
t: procedure expose e
  parse arg cmd, names
  address system
  cmd
  s = ''
  do n over names~makeArray(' ')
    s = s '['value(n, , e)']'
  end
  say '['cmd']' rc s
  return
```

```output
[export A4ZQ_X1=1] 0  [1]
[set A4ZQ_X2=2] 0  [2]
[export  A4ZQ_X3=a  b ] 0  [a  b ]
[export A4ZQ_X4="q r"] 0  ["q r"]
[export A4ZQ_X5=7 A4ZQ_Y5=8] 0  [7 A4ZQ_Y5=8] []
[export A4ZQ_X6] 0  []
[export A4ZQ_X8=$A4ZQ_H/s:t] 0  [hval/s:t]
[export A4ZQ_X9=<$A4ZQ_H$A4ZQ_H>] 2  []
[export A4ZQ_X10=${A4ZQ_H}] 0  []
[export A4ZQ_X11=u$A4ZQ_nope] 0  [u]
[export A4ZQ_X12=~/x] 0  [~/x]
[export A4ZQ_X13=1;] 0  []
[export A4ZQ_X14=] 0  []
set but null 0
```

r[env.commands.unset]
`unset`, given an argument, removes the environment variable that the rest of
the string after the blanks names, and answers 0. The whole rest is the name,
so a blank in it or at its end is part of it, and a name that no variable has
removes nothing. `unset` with no argument is run by the shell.

```rexx
e = 'ENVIRONMENT'
address system
do n over .array~of('A4ZQ_U1', 'A4ZQ_U2', 'A4ZQ_U3', 'A4ZQ_U4', 'A4ZQ_W4',,
    'A4ZQ_U5', 'A4ZQ_U6')
  call value n, 'set', e
end
call t 'unset A4ZQ_U1', 'A4ZQ_U1'
call t 'unset  A4ZQ_U2', 'A4ZQ_U2'
call t 'unset A4ZQ_U3 ', 'A4ZQ_U3'
call t 'unset A4ZQ_U4 A4ZQ_W4', 'A4ZQ_U4 A4ZQ_W4'
call t 'unset A4ZQ_nope', 'A4ZQ_U5'
call t 'unset A4ZQ_U6;', 'A4ZQ_U6'
exit
t: procedure expose e
  parse arg cmd, names
  address system
  cmd
  s = ''
  do n over names~makeArray(' ')
    s = s '['value(n, , e)']'
  end
  say '['cmd']' rc s
  return
```

```output
[unset A4ZQ_U1] 0  []
[unset  A4ZQ_U2] 0  []
[unset A4ZQ_U3 ] 0  [set]
[unset A4ZQ_U4 A4ZQ_W4] 0  [set] [set]
[unset A4ZQ_nope] 0  [set]
[unset A4ZQ_U6;] 0  [set]
```

r[env.commands.deviation.149]
Every value that VALUE sets in the `ENVIRONMENT` is the value of the variable
afterwards and is what a command sees, whatever `export` and `unset` the
interpreter did before, the one that gives a variable the null string and
the repeated `unset` of a name included. An `export` with an empty name
changes nothing and answers 0. This is [Deviation
149](../deviations.md#149-environment-variables-set-after-export-and-unset):
ooRexx loses some of these sets, which ones depending on the in-process
`export` and `unset` commands that the program ran before for any variable, so
that the second line of the example is empty there. And `export =3` stores an
entry whose name is empty, which a `$` with no name after it then refers to,
so that the last line is `[a3]` there.

```rexx
e = 'ENVIRONMENT'
address system
call value 'A4ZQ_U', 'u', e
'export A4ZQ_U='
call value 'A4ZQ_U', 'u5', e
say '['value('A4ZQ_U', , e)']'
call value 'A4ZQ_Z', 'w', e
'unset A4ZQ_Z'
call value 'A4ZQ_Z', 'w', e
'unset A4ZQ_Z'
call value 'A4ZQ_Z', 'w2', e
say '['value('A4ZQ_Z', , e)']'
'export =3'
say rc
'export A4ZQ_R=a$'
say '['value('A4ZQ_R', , e)']'
```

```output
[u5]
[w2]
0
[a]
```

## Redirection

r[env.commands.redirect]
The redirections of [the ADDRESS instruction](r:instr.address.with.syntax)
apply to the commands of every environment that runs a command: the shells
and `PATH` give the command the streams that the redirection names for its
standard input, output and error. A command that fails still fills the stems
and streams that it wrote: the output and error of a shell that answered 127,
of a program that exited with a nonzero status and of one that a signal ended
are all there, and so are the empty ones, as stems with the count 0. A
command that is sent to an environment that does not exist is not run, and
its redirections are left as they were.

```rexx
address system
do c over .array~of('echo out; echo err >&2; exit 3', 'exit 127', ,
    'echo o; kill -s KILL $$')
  address system c with output stem o. error stem e.
  say rc o.0 e.0 o.1 e.1
end
o.0 = 'kept'
e.0 = 'kept'
address a4zq_none 'echo x' with output stem o. error stem e.
say rc o.0 e.0
address path 'printf %s-%s a b' with output stem o. error stem e.
say rc o.0 e.0 o.1
address path 'a4zq_none_cmd' with output stem o. error stem e.
say rc o.0 e.0
```

```output
3 1 1 out err
127 0 0 O.1 E.1
-9 1 0 o E.1
30 kept kept
0 1 0 a-b
127 0 0
```

## The queue and the filter

r[env.commands.rxqueue.filter]
`rxqueue` is a program of the interpreter's installation that a command finds
by its name, as it finds any program. It reads its standard input and puts the
lines on a queue of the program that ran the command, which makes it the way
for a command to hand lines to the program: a command whose output is piped
into `rxqueue` leaves its lines in the queue when it ends. That queue is the
program's `SESSION` queue, the one that PUSH, QUEUE, QUEUED and PULL work on
until RXQUEUE sets another ([the get rule](r:bif.rxqueue.get)), and the lines
are put on it as the rules of its options say. A line of the input ends at a
line feed, or at a carriage return followed by a line feed; a carriage return
that no line feed follows also ends a line, and the character after it begins
the next line and belongs to it whatever it is, so that a second carriage
return directly after the first is text of the next line, not the end of an
empty one; text after the last line end is a line, an empty line is a line,
and input with no character puts none. The text is not changed, so that case,
blanks, tabs and control characters stay as they are; only PULL changes the
case, as [its rule](r:instr.pull.source) says. A line is cut after its first
65472 characters, and the rest of it up to its end is lost. The filter ends
with the status 0 when it did its work, and the status of a pipeline that ends
in it is its status, as for any pipeline of a shell. The other statuses are in
the rules below. The text that it writes when it fails goes to its standard
output, and this book does not specify it.

```rexx
address system
queue 'first'
'printf "second\n\n  Third  \r\nfourth" | rxqueue'
push 'zero'
say queued()
do queued()
  parse pull l
  say '['l']'
end
'printf "%70000s\nnext\n" x | rxqueue'
say queued()
parse pull l
say length(l) verify(l, ' ')
parse pull l
say l
'printf "" | rxqueue'
say rc queued()
```

```output
6
[zero]
[first]
[second]
[]
[  Third  ]
[fourth]
2
65472 0
next
0 0
```

```rexx
address system
'printf "a\rb\r\rc\r\r\nd\re" | rxqueue'
do queued()
  parse pull l
  say '['l~changeStr('0d'x, '<CR>')']'
end
```

```output
[a]
[b]
[<CR>c]
[<CR>]
[d]
[e]
```

r[env.commands.rxqueue.options]
`rxqueue` takes at most one option and at most one queue name, in either
order. An option is `/` or `-` directly followed by `FIFO`, `LIFO` or `CLEAR`,
in any letter case, and is the whole word; an abbreviation is not an option.
The option `/FIFO`, which is the default, puts each line at the end of the
queue, so that the lines come out in the order of the input after what the
queue held. `/LIFO` puts each line at the front as it is read, so that the
lines come out in the reverse order of the input and before what the queue
held. `/CLEAR` removes every line of the queue, reads no input and puts
nothing on the queue. A word that begins with `/` or `-` and is not one of
these options, a second option, and a second name are errors in the call:
`rxqueue` writes a usage text, puts nothing on the queue and answers the
status 0.

```rexx
address system
call rxqueue 'create', 'a4zq_opt'
q = .rexxqueue~new('a4zq_opt')
call t 'printf "a\nb\n" | rxqueue a4zq_opt'
call t 'printf "a\nb\n" | rxqueue a4zq_opt /fifo'
call t 'printf "a\nb\n" | rxqueue -LIFO A4ZQ_OPT'
call t 'printf "a\nb\n" | rxqueue /lifo a4zq_opt'
call t 'printf "a\nb\n" | rxqueue a4zq_opt /clear'
call t 'rxqueue /Clear a4zq_opt </dev/null'
call t 'printf "a\nb\n" | rxqueue /li'
call t 'printf "a\nb\n" | rxqueue a4zq_opt /li'
call t 'printf "a\nb\n" | rxqueue a4zq_opt /lifo /fifo'
call t 'printf "a\nb\n" | rxqueue a4zq_opt SESSION'
call t 'printf "a\nb\n" | rxqueue a4zq_opt lifo'
call t 'printf "a\nb\n" | rxqueue'
say 'session' queued()
do queued()
  parse pull
end
call rxqueue 'delete', 'a4zq_opt'
exit
t:
  parse arg cmd
  q~empty
  q~queue('x')
  q~queue('y')
  cmd '>/dev/null'
  say right(rc, 1) '['q~makeArray~makeString('l', ' ')']' cmd~substr(cmd~lastPos('|') + 1)~strip
  return
```

```output
0 [x y a b] rxqueue a4zq_opt
0 [x y a b] rxqueue a4zq_opt /fifo
0 [b a x y] rxqueue -LIFO A4ZQ_OPT
0 [b a x y] rxqueue /lifo a4zq_opt
0 [] rxqueue a4zq_opt /clear
0 [] rxqueue /Clear a4zq_opt </dev/null
0 [x y] rxqueue /li
0 [x y] rxqueue a4zq_opt /li
0 [x y] rxqueue a4zq_opt /lifo /fifo
0 [x y] rxqueue a4zq_opt SESSION
0 [x y] rxqueue a4zq_opt lifo
0 [x y] rxqueue
session 2
```

r[env.commands.rxqueue.name]
The queue that `rxqueue` works on is the queue that its name word names, and
the name need not be written in the case that the queue was created with
([the case rule](r:bif.rxqueue.case)). When there is no name word, it is the
queue that the environment variable `RXQUEUE` names, when the variable has a
value, and the `SESSION` queue otherwise. A name word wins over the
variable. The queue must exist, and a name that is not one of a queue is
refused ([the name rule](r:bif.rxqueue.name)): `rxqueue` writes a message,
puts nothing on any queue and answers the status 9 when the queue does not
exist and 5 when the name is not valid; the null string, a value of blanks
only, and a value with a leading or trailing blank as the value of `RXQUEUE`
count as the second. The function RXQUEUE with `SET` does not set the
variable, and a program's current queue plays no part in which queue the
filter uses.

```rexx
address system
e = 'ENVIRONMENT'
call rxqueue 'create', 'a4zq_nm'
call rxqueue 'create', 'a4zq_other'
n = .rexxqueue~new('a4zq_nm')
o = .rexxqueue~new('a4zq_other')
call t 'echo 1 | rxqueue a4zq_nm'
call t 'echo 2 | rxqueue'
call value 'RXQUEUE', 'A4ZQ_NM', e
call t 'echo 3 | rxqueue'
call t 'echo 4 | rxqueue a4zq_other'
call t 'echo 5 | rxqueue session'
call value 'RXQUEUE', 'a4zq_none', e
call t 'echo 6 | rxqueue'
call value 'RXQUEUE', '', e
call t 'echo 7 | rxqueue'
call value 'RXQUEUE', '  ', e
call t 'echo 7a | rxqueue'
call value 'RXQUEUE', ' a4zq_nm', e
call t 'echo 7b | rxqueue'
call value 'RXQUEUE', 'a4zq_nm ', e
call t 'echo 7c | rxqueue'
call t 'echo 8 | rxqueue "a b"'
call t 'echo 9 | rxqueue a-b'
call t 'echo 10 | rxqueue /clear a4zq_none'
call value 'RXQUEUE', .nil, e
call t 'echo 11 | rxqueue'
do queued()
  parse pull
end
call rxqueue 'set', 'a4zq_other'
'echo 12 | rxqueue >/dev/null'
say 'other' o~queued
say rxqueue('set', 'SESSION') queued()
do queued()
  parse pull
end
call rxqueue 'delete', 'a4zq_nm'
call rxqueue 'delete', 'a4zq_other'
exit
t:
  parse arg cmd
  n~empty
  o~empty
  do queued()
    parse pull
  end
  cmd '>/dev/null'
  say right(rc, 1) 'nm['n~makeArray~makeString('l', ' ')']',
    'other['o~makeArray~makeString('l', ' ')']' 'session['queued()']',
    cmd~substr(cmd~lastPos('|') + 1)~strip
  return
```

```output
0 nm[1] other[] session[0] rxqueue a4zq_nm
0 nm[] other[] session[1] rxqueue
0 nm[3] other[] session[0] rxqueue
0 nm[] other[4] session[0] rxqueue a4zq_other
0 nm[] other[] session[1] rxqueue session
9 nm[] other[] session[0] rxqueue
5 nm[] other[] session[0] rxqueue
5 nm[] other[] session[0] rxqueue
5 nm[] other[] session[0] rxqueue
5 nm[] other[] session[0] rxqueue
5 nm[] other[] session[0] rxqueue "a b"
5 nm[] other[] session[0] rxqueue a-b
9 nm[] other[] session[0] rxqueue /clear a4zq_none
0 nm[] other[] session[1] rxqueue
other 0
A4ZQ_OTHER 1
```

r[env.commands.rxqueue.session]
A program starts with the environment variable `RXQUEUESESSION` set, before
its first clause, to `0x` followed by lower-case hexadecimal digits. The
value names the program's `SESSION` queue for `rxqueue` and for a Rexx
program that a command starts. A command inherits it with the other
variables, as [the start rule](r:env.commands.start) says, and that is how
the filter and [a child program](r:env.commands.rxqueue.child) reach the
queue. The interpreter sets it once, at the start. A value that the program
was started with is used as the session when it begins with hexadecimal
digits, with or without `0x` before them, as for [a child
program](r:env.commands.rxqueue.child), and is replaced otherwise. After that
it is a variable of the program like any other, and a value that the program
gives it is the value that a command sees. The digits depend on the run.

```rexx
v = value('RXQUEUESESSION', , 'ENVIRONMENT')
say left(v, 2) (length(v) > 2) verify(substr(v, 3), '0123456789abcdef')
```

```output defect=#220
0x 1 0
```

```rexx
call value 'RXQUEUESESSION', 'a4r', 'ENVIRONMENT'
address system 'echo "$RXQUEUESESSION"' with output stem o.
say o.1 value('RXQUEUESESSION', , 'ENVIRONMENT')
```

```output defect=#220
a4r a4r
```

r[env.commands.rxqueue.child]
A Rexx program that a command starts, as the command `rexx` does, has the
queue `SESSION` of the program that ran the command as its own, and it is its
current queue when it starts, whatever the current queue of the parent is. The
lines that the parent put on it are there for the child to pull, and what the
child leaves on it is there for the parent when the command ends.

```rexx
address system
queue 'parent'
'rexx -e "parse pull a; say a; queue ''child''"'
parse pull b
say b
call rxqueue 'create', 'a4zq_ch'
call rxqueue 'set', 'a4zq_ch'
'rexx -e "say rxqueue(''get'')"'
call rxqueue 'set', 'SESSION'
call rxqueue 'delete', 'a4zq_ch'
```

```output
parent
child
SESSION
```

r[env.commands.rxqueue.deviation.93]
A `rxqueue` that does not run for a program, so that it has no session queue
of a program to use, and that is to work on the `SESSION` queue, whether by
default or by name, writes a message and answers the status 9, and puts
nothing anywhere. This is [Deviation
93](../deviations.md#93-rxqueue-with-no-session): ooRexx answers the status 0
and puts the lines on a queue that no program reaches. A command leaves that
state when it takes away the variable `RXQUEUESESSION` ([the session
rule](r:env.commands.rxqueue.session)), which the example does for the shell
that runs it.

```rexx
address system
'unset RXQUEUESESSION; echo line | rxqueue >/dev/null; echo "[$?]"'
'unset RXQUEUESESSION; echo line | rxqueue session >/dev/null; echo "[$?]"'
'unset RXQUEUESESSION; echo line | rxqueue /clear >/dev/null; echo "[$?]"'
say queued()
```

```output
[9]
[9]
[9]
0
```
