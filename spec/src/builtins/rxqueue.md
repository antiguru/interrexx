---
element: RXQUEUE
kind: builtin
syntax: "RXQUEUE(option [, queuename])"
rexxref: "Functions > Built-in Functions > RXQUEUE"
---

# RXQUEUE

RXQUEUE creates, deletes and asks after the external data queues, and names
or changes the current one. A queue is a list of lines kept outside the
program, in a service that every program of a session can reach, where the
QUEUE and PUSH instructions put lines and the PULL and PARSE PULL
instructions take them out. A program has one current queue, the one those
instructions use and [QUEUED](queued.md) counts. The queue called `SESSION`
is the current queue when a program starts, it always exists, and it cannot
be deleted.

> **Note.** The examples create a queue of their own under a name no other
> example uses, and delete it when they are done, so they leave nothing
> behind and do not depend on what the `SESSION` queue holds when they start.
> A queue that a program creates and does not delete stays until something
> deletes it, even after the program ends.

## Arguments

r[bif.rxqueue.args]
`option` is a string and is required. RXQUEUE does not run as a String method:
sending the message `RXQUEUE` to a string raises `SYNTAX 97.1`. The
[call rules](r:bif.index.call.count) hold for the number of arguments, with the
fewest and the most depending on the option, which the rules below give:
a call with no argument, or with only omitted places, raises `SYNTAX 40.3`; a
call of at most two arguments whose first is omitted but whose second is
present raises `SYNTAX 40.5`. The checks come in this order. First the
number of arguments is checked against the bounds that hold for every
function: at most two arguments, so that `rxqueue(, 'a', 'b')` raises
`SYNTAX 40.4`, and then an `option`. Then the
[first character of `option`](r:bif.rxqueue.option)
is checked, then the number of arguments against the bounds of the function
it picks, and last [`queuename`](r:bif.rxqueue.name).

```rexx
signal on syntax name a
say rxqueue()
a: say condition('o')~code
signal on syntax name b
say rxqueue(, 'x')
b: say condition('o')~code
signal on syntax name c
say 'Get'~rxqueue
c: say condition('o')~code
signal on syntax name d
say rxqueue('X', 'a', 'b')
d: say condition('o')~code
signal on syntax name e
say rxqueue('X', 'a b')
e: say condition('o')~code
signal on syntax name f
say rxqueue('Get', 'a b')
f: say condition('o')~code
signal on syntax name g
say rxqueue(, 'a b')
g: say condition('o')~code
signal on syntax name h
say rxqueue(, 'a', 'b')
h: say condition('o')~code
signal off syntax
say rxqueue('Get', )
```

```output
40.3
40.5
97.1
40.4
40.904
40.4
40.5
40.4
SESSION
```

r[bif.rxqueue.option]
The first character of `option` picks the function, in either case, and the
characters after it are ignored. The functions are `Create`, `Delete`,
`Exist`, `Get`, `Open` and `Set`, so the first character is one of `C`, `D`,
`E`, `G`, `O` and `S`. Any other first character, and the null string, raises
`SYNTAX 40.904`.

```rexx
say rxqueue('Get') rxqueue('get') rxqueue('G') rxqueue('gXYZ')
say rxqueue('exist', 'SESSION') rxqueue('E', 'SESSION')
signal on syntax name a
say rxqueue('X')
a: say condition('o')~code
signal on syntax name b
say rxqueue('')
b: say condition('o')~code
signal on syntax name c
say rxqueue(1)
c: say condition('o')~code
```

```output
SESSION SESSION SESSION SESSION
1 1
40.904
40.904
40.904
```

r[bif.rxqueue.name]
`queuename` is a string. It is the second argument of every function but
`Get`, and a call to `Get` with it raises `SYNTAX 40.4`. `Create` takes it as an
option: without it, the service picks the name. `Delete`, `Exist`, `Open` and
`Set` require it, and a call without it raises `SYNTAX 40.3`; a call with a
third argument raises `SYNTAX 40.4`. A queue name is a symbol that
[SYMBOL](symbol.md) does not call `BAD`, so it has no blank, no `-` and none
of the other characters that no symbol can have. It is at most 250
characters long. A name that is not valid, the null string among them,
raises `SYNTAX 40.26`, before the function does anything. The name is not
a variable: the program need not have a variable of that name, and a
variable of that name plays no part.

```rexx
signal on syntax name a
say rxqueue('Get', 'x')
a: say condition('o')~code
signal on syntax name b
say rxqueue('Delete')
b: say condition('o')~code
signal on syntax name c
say rxqueue('Exist', 'a', 'b')
c: say condition('o')~code
signal on syntax name d
say rxqueue('Exist', 'a b')
d: say condition('o')~code
signal on syntax name e
say rxqueue('Create', 'a-b')
e: say condition('o')~code
signal on syntax name f
say rxqueue('Set', '')
f: say condition('o')~code
signal on syntax name g
say rxqueue('Create', copies('a', 251))
g: say condition('o')~code
signal off syntax
n = rxqueue('Create', copies('a', 250))
say length(n) rxqueue('Exist', n) rxqueue('Delete', n)
n = rxqueue('Create', '1.x!_?')
say n rxqueue('Delete', n)
```

```output
40.4
40.3
40.4
40.26
40.26
40.26
40.26
250 1 0
1.x!_? 0
```

## The queues

r[bif.rxqueue.case]
The case of a queue name plays no part: the service keeps the name in upper
case, and finds a queue by it. The names that `Get` and `Set` answer are in
upper
case. `Create` answers a name it created as it was given.

```rexx
n = rxqueue('Create', 'spec_rxqueue_Mixed')
say n rxqueue('Exist', 'SPEC_RXQUEUE_MIXED') rxqueue('Exist', 'spec_rxqueue_mixed')
say rxqueue('Set', n) rxqueue('Get')
say rxqueue('Set', 'session') rxqueue('Get')
say rxqueue('Delete', 'Spec_Rxqueue_MIXED') rxqueue('Exist', n)
```

```output
spec_rxqueue_Mixed 1 1
SESSION SPEC_RXQUEUE_MIXED
SPEC_RXQUEUE_MIXED SESSION
0 0
```

r[bif.rxqueue.get]
`Get` answers the name of the current queue, in upper case. It is `SESSION`
until a call to `Set` changes it, and it is the name that call set, whether
or not the queue by that name exists, and after the queue was deleted.

```rexx
say rxqueue('Get')
q = rxqueue('Create', 'spec_rxqueue_get')
call rxqueue 'Set', q
say rxqueue('Get')
call rxqueue 'Delete', q
say rxqueue('Get')
call rxqueue 'Set', 'SESSION'
say rxqueue('Get')
```

```output
SESSION
SPEC_RXQUEUE_GET
SPEC_RXQUEUE_GET
SESSION
```

r[bif.rxqueue.create]
`Create` makes a queue and answers its name. With a `queuename` that no queue
has, the queue gets that name and the answer is the name as it was given. When
a queue of that name exists already, the queue is made under a name that the
service picks, and the answer is that name; unless `queuename` is `SESSION`,
the answer of `Create` is therefore always the name of a new queue. Without
a `queuename` the service picks the name too. The form of a name the service
picks is not specified, but it is a valid queue name, it is not `SESSION` and
no existing queue has it. The name `SESSION` makes no queue, and the answer is
the null string.

```rexx
a = rxqueue('Create', 'spec_rxqueue_create')
b = rxqueue('Create', 'spec_rxqueue_create')
c = rxqueue('Create', 'SPEC_RXQUEUE_CREATE')
d = rxqueue('Create')
say a (b \== a) (c \== a) (c \== b) (d \== a) (d \== b) (d \== c)
say rxqueue('Exist', a) rxqueue('Exist', b) rxqueue('Exist', c) rxqueue('Exist', d)
say rxqueue('Delete', a) rxqueue('Delete', b) rxqueue('Delete', c) rxqueue('Delete', d)
say '[' || rxqueue('Create', 'session') || ']' rxqueue('Get')
```

```output
spec_rxqueue_create 1 1 1 1 1 1
1 1 1 1
0 0 0 0
[] SESSION
```

r[bif.rxqueue.exist]
`Exist` answers `1` when a queue of the name exists and `0` when it does not.
The queue `SESSION` always exists.

```rexx
q = rxqueue('Create', 'spec_rxqueue_exist')
say rxqueue('Exist', q) rxqueue('Exist', 'SESSION')
call rxqueue 'Delete', q
say rxqueue('Exist', q) rxqueue('Exist', 'spec_rxqueue_exist_none')
```

```output
1 1
0 0
```

r[bif.rxqueue.open]
`Open` makes sure a queue of the name exists: it creates the queue when there
is none and leaves an existing one as it is, with its lines. It answers `0`
in both cases. Unlike `Create`, it never makes a queue under another name.

```rexx
say rxqueue('Exist', 'spec_rxqueue_open')
say rxqueue('Open', 'spec_rxqueue_open') rxqueue('Exist', 'spec_rxqueue_open')
call rxqueue 'Set', 'spec_rxqueue_open'
queue 'kept'
call rxqueue 'Set', 'SESSION'
say rxqueue('Open', 'Spec_Rxqueue_Open')
call rxqueue 'Set', 'spec_rxqueue_open'
say queued()
call rxqueue 'Set', 'SESSION'
say rxqueue('Delete', 'spec_rxqueue_open') rxqueue('Open', 'SESSION')
```

```output
0
0 1
0
1
0 0
```

r[bif.rxqueue.delete]
`Delete` removes a queue and its lines, and answers a number. The answer is `0`
when the queue was deleted, `9` when no queue has the name and `5` for the
queue `SESSION`, which stays. The answers for a queue in use by a program
that waits on it, and for a failure of the service, are not specified. A
queue that is the current queue can be deleted: it stays the current queue
by name, and a program that puts a line in it then fails, as [the rule on
`Set`](r:bif.rxqueue.set) says.

```rexx
q = rxqueue('Create', 'spec_rxqueue_delete')
say rxqueue('Delete', q) rxqueue('Delete', q) rxqueue('Delete', 'SESSION')
say rxqueue('Delete', 'spec_rxqueue_delete_none') rxqueue('Exist', 'SESSION')
```

```output
0 9 5
9 1
```

r[bif.rxqueue.set]
`Set` makes the queue of the name the current queue, and answers the name of
the queue that was current before, in upper case. It does not look for the
queue: a name that no queue has is accepted, and `Get` answers it. A line put
into a queue that does not exist, by QUEUE or PUSH, raises `SYNTAX 48.1`, and
[QUEUED](queued.md) answers `0` for it. After `Set`, the QUEUE and PUSH
instructions put lines into the new current queue, and the PULL and PARSE PULL
instructions take them out of it.

```rexx
q = rxqueue('Create', 'spec_rxqueue_set')
say rxqueue('Set', q) rxqueue('Set', 'spec_rxqueue_set_none') rxqueue('Set', q)
queue 'second'
push 'first'
parse pull a
parse pull b
say a b queued()
call rxqueue 'Set', 'spec_rxqueue_set_none'
signal on syntax name e
queue 'lost'
say 'not reached'
e: say condition('o')~code queued()
call rxqueue 'Set', 'SESSION'
call rxqueue 'Delete', q
```

```output
SESSION SPEC_RXQUEUE_SET SPEC_RXQUEUE_SET_NONE
first second 0
48.1 0
```
