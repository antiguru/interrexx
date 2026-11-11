---
element: "NativeCommandHandler"
kind: class
rexxref: "Builtin Classes > Utility Classes > NativeCommandHandler Class; Builtin Classes > Utility Classes > NativeCommandHandler Class > command; Builtin Classes > Utility Classes > NativeCommandHandler Class > kind; Builtin Classes > Utility Classes > NativeCommandHandler Class > name"
---

# NativeCommandHandler

Some command environments are not written in Rexx: the environment `SYSTEM`,
which runs commands in the host's shell, the environments that name a shell,
and those that an application that hosts the interpreter adds. A
`NativeCommandHandler` is the object that stands for one of them, so that a
program can list it, register it again under another name, remove it, and send
it a command from a handler written in Rexx. The class inherits
[`CommandHandler`](commandhandler.md), and the objects answer to the same
class methods as the handlers written in Rexx do. [How a native environment
runs a command](r:env.commands.shell), and [what the environments of a host
are](r:env.commands.names), is specified in the environment chapter. The
examples here use the environment `SYSTEM` and the commands `echo` and `exit`,
which every host's shell has.

## The class

r[class.nativecommandhandler.creation]
`NativeCommandHandler` has the instance methods `command`, `kind` and `name`
besides the methods that every class has. It inherits the mixin class
`CommandHandler`, and its superclass is `Object`: its `superClasses` are
`Object` and `CommandHandler`, in that order ([the hierarchy
rule](r:class.index.hierarchy)). A program does not make these objects: `new`
raises `SYNTAX 97.2` whatever its arguments are. The interpreter makes one for
each native environment and answers it from the class methods of
`CommandHandler` ([`handler`](r:class.commandhandler.handler),
[`handlers`](r:class.commandhandler.handlers),
[`environments`](r:class.commandhandler.environments),
[`register`](r:class.commandhandler.register) and
[`deregister`](r:class.commandhandler.deregister)), and a native environment
always answers the same object, so `==` finds it again. The class may have
private methods besides these; they are not part of this specification.

```rexx
s = ''
do c over .nativecommandhandler~superclasses
  s = s c~id
end
say .nativecommandhandler~id .nativecommandhandler~superclasses~items s~strip
say .nativecommandhandler~queryMixinClass,
  .nativecommandhandler~isSubclassOf(.commandhandler)
h = .commandhandler~handler('system')
say h~class~id h~isA(.commandhandler) h~isA(.nativecommandhandler)
say (h == .commandhandler~handler('SYSTEM')),
  (h == .commandhandler~environments['SYSTEM'])
say .nativecommandhandler~hasMethod('HANDLER') h~hasMethod('HANDLER')
call try '.nativecommandhandler~new'
call try '.nativecommandhandler~new("system")'
call try '.nativecommandhandler~new("system", "REDIRECTING")'
exit
try:
  signal on syntax name bad
  interpret 'x =' arg(1)
  say arg(1) 'no error'
  return
bad: say arg(1) condition('o')~code
  return
```

```output
NativeCommandHandler 2 Object CommandHandler
0 1
NativeCommandHandler 1 1
1 1
1 0
.nativecommandhandler~new 97.2
.nativecommandhandler~new("system") 97.2
.nativecommandhandler~new("system", "REDIRECTING") 97.2
```

## What it knows

r[class.nativecommandhandler.name]
`name` answers the name under which the native environment was added, in upper
case: `SYSTEM` for the environment `SYSTEM`. It is a property of the
environment, not of the name that a program uses for it, so it does not change
when the object is registered under another name. The method sends no message.

```rexx
h = .commandhandler~handler('system')
say h~name .commandhandler~handler('SyStEm')~name
say .commandhandler~register('spec_nch_alias', h)~isNil
say .commandhandler~handler('spec_nch_alias')~name h~name
x = .commandhandler~deregister('spec_nch_alias')
```

```output
SYSTEM SYSTEM
1
SYSTEM SYSTEM
```

r[class.nativecommandhandler.kind]
`kind` answers a string that tells how the native environment was added. It is
`REDIRECTING` for the environments of the interpreter itself, `SYSTEM` and the
shells among them, and for any that a host application added as a handler that
supports `ADDRESS ... WITH` redirection ([the redirection
rules](r:instr.address.with.syntax)); `DIRECT` for one a host application
added as a handler that does not support redirection; and `SUBCOM` for a
handler of the older subcommand interface of the host. A program cannot add a
native environment, so the examples can only show `REDIRECTING`. The method
takes no argument (`SYNTAX 88.922`).

```rexx
h = .commandhandler~handler('system')
say h~kind h~kind~class~id
signal on syntax name bad
x = h~kind(1)
bad: say condition('o')~code
```

```output
REDIRECTING String
88.922
```

## Sending a command

r[class.nativecommandhandler.command]
`command(command, caller)` sends `command` to the native environment as if the
code that sent the command that `caller` stands for had sent it: the environment
runs it with the variables of that code and with the redirections of its
`ADDRESS ... WITH`, under the name that
[`name`](r:class.nativecommandhandler.name) answers, and the method answers the
return code of the command. A condition that the environment raises, ERROR for a
return code other than zero or FAILURE for a command that could not be run,
becomes a condition of the command that `caller` stands for, which the code that
sent it traps as it does for any command; the handler that called `command` does
not see it. `caller` is the [`CommandContext`](commandcontext.md) that a handler
gets for the command it is running. Both arguments are required
(`SYNTAX 93.901`) and a third raises `SYNTAX 93.902`, as the command
`echo third` of the example shows. What the method does with another object as
`caller` is not specified. This is how a handler written in Rexx forwards a
command to the native environment that it covers. The example covers `SYSTEM`
with a handler that writes the command and forwards it, runs a command with its
output and error redirected, shows the error for a third argument, and shows
ERROR reaching the code that sent the command.

```rexx
sys = .commandhandler~handler('system')
x = .commandhandler~register('system', .logger~new(sys))
address system 'echo forwarded'
say 'rc' rc
address system 'echo third'
address system 'echo out; echo err >&2; exit 4',
  with output stem o. error stem e.
say 'rc' rc 'o.0' o.0 o.1 'e.0' e.0 e.1
signal on error name caught
address system 'exit 3'
say 'not reached'
caught: say 'trapped' condition('c') rc condition('d')
call off error
x = .commandhandler~deregister('system')
say x~class~id (x == sys)
signal on syntax name bad
y = sys~command('echo x')
bad: say condition('o')~code
::class logger inherit CommandHandler
::method init
  expose below
  use arg below
::method command
  expose below
  use arg command, caller
  say 'log:' command
  if command == 'echo third' then do
    signal on syntax name extra
    return below~command(command, caller, 3)
  end
  return below~command(command, caller)
  extra: say 'extra' condition('o')~code
  return 0
```

```output
log: echo forwarded
forwarded
rc 0
log: echo third
extra 93.902
log: echo out; echo err >&2; exit 4
rc 4 o.0 1 out e.0 1 err
log: exit 3
trapped ERROR 3 exit 3
LOGGER 0
93.901
```

## Covering and removing

r[class.nativecommandhandler.environment]
A native environment is an environment like any other for the class methods of
`CommandHandler`. [`register`](r:class.commandhandler.register) with its name
covers it, so that commands go to the new handler, and answers its
`NativeCommandHandler`; the object answers `register` too, so it can be
registered under another name, where commands sent to that name reach the
environment, and the environment name in their context is the new name.
[`deregister`](r:class.commandhandler.deregister) takes a handler written in
Rexx away and brings the native environment back into use, and when it takes
the native one itself, the name is no longer an environment of this
interpreter instance: a command sent to it raises FAILURE with the return code
`30`. The object that `deregister` answered can be registered again, which
restores the environment. Nothing outside the interpreter instance changes.
The examples cover `SYSTEM` and uncover it, then remove it and restore it.

```rexx
sys = .commandhandler~handler('system')
say .commandhandler~register('spec_nch_other', sys)~isNil
address spec_nch_other 'echo through another name'
say 'rc' rc
x = .commandhandler~deregister('spec_nch_other')
say .commandhandler~register('system', .quiet~new)~class~id
say .commandhandler~handlers('system')~items,
  .commandhandler~handlers('system')[2]~name
address system 'echo hidden'
say 'rc' rc
say .commandhandler~deregister('system')~class~id
address system 'echo visible'
say 'rc' rc
x = .commandhandler~deregister('system')
say x~class~id .commandhandler~handler('system')~isNil
signal on failure name gone
address system 'echo not run'
gone: say condition('c') rc
signal off failure
say .commandhandler~environments~hasIndex('SYSTEM')
say .commandhandler~register('system', sys)~isNil
address system 'echo restored'
say 'rc' rc (.commandhandler~handler('system') == sys)
::class quiet inherit CommandHandler
::method command
  return 5
```

```output
1
through another name
rc 0
NativeCommandHandler
2 SYSTEM
rc 5
QUIET
visible
rc 0
NativeCommandHandler 1
FAILURE 30
0
1
restored
rc 0 1
```

r[class.nativecommandhandler.copy]
`copy` answers a new `NativeCommandHandler` for the same native environment: it
is not the object that `handler` answers, but it has the same `name` and `kind`,
and a command sent to it, directly or through a name it is registered under,
runs in that environment as with the original. In the example the copy is
registered under a name of its own, and a handler written in Rexx sends it a
command with `command`.

```rexx
sys = .commandhandler~handler('system')
k = sys~copy
say (k == sys) k~class~id k~kind k~name
say .commandhandler~register('spec_nch_copy', k)~isNil
address spec_nch_copy 'echo through the name'
say 'rc' rc
x = .commandhandler~deregister('spec_nch_copy')
x = .commandhandler~register('spec_nch_fwd', .forward~new(k))
address spec_nch_fwd 'echo through command; exit 2'
say 'rc' rc
x = .commandhandler~deregister('spec_nch_fwd')
::class forward inherit CommandHandler
::method init
  expose target
  use arg target
::method command
  expose target
  use arg command, caller
  return target~command(command, caller)
```

```output defect=#183
0 NativeCommandHandler REDIRECTING SYSTEM
1
through the name
rc 0
through command
rc 2
```
