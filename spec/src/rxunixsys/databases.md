---
element: "SysGetpwnam"
kind: chapter
syntax: "SysGetpwnam(user, option); SysGetpwuid(uid, option); SysGetgrnam(group, option); SysGetgrgid(gid, option); SysGetservbyname(name, protocol, option); SysGetservbyport(port, protocol, option)"
rexxref: "Unix extensions > User and Group Functions; Unix extensions > Name Lookup Functions"
---

# Users, groups and services

The functions on this page look up an entry of the databases of the system: the
users, the groups and the network services. What every function of [the
rxunixsys library](index.md) shares holds here too. Every lookup sets
[the error number](r:rxunixsys.index.errno) to `0` before it looks, so that
`SysGeterrno()` answers `0` after it whether the entry was found or not.

## Users

r[rxunixsys.databases.user]
`SysGetpwnam(user, option)` and `SysGetpwuid(uid, option)` answer one field of the
entry of a user, found by the name `user` or by the number `uid`, as `option` names it,
by its first character in either case: `N` the name, `U` the number, `G` the number of
the primary group, `R` the real name or comment, `D` the home directory, `S` the
login shell and `P` the password, which is the null string. A number is written
as a whole number. The answer is the null string where there is no such user. An
option that starts with another character, the null string, and a `user` that is the
null string raise `SYNTAX 40.1` whether the user exists or not, but `uid` is a [whole
number](r:rxunixsys.index.whole). The user with the number `0` is named `root`
on a system that has the user at all.

```rexx
say SysGetpwnam('root', 'n') SysGetpwnam('root', 'u') SysGetpwnam('root', 'g')
say SysGetpwuid(0, 'n') SysGetpwuid(0, 'Name') SysGetpwuid(0, 'u') SysGetpwuid(0, 'G')
say (SysGetpwnam('root', 'd') <> '') (SysGetpwnam('root', 's') <> '') '['SysGetpwnam('root', 'p')']'
say '['SysGetpwnam('c1zqnouser', 'n')']' SysGeterrno() '['SysGetpwnam('c1zqnouser', 'x')']'
say '['SysGetpwuid(2000000000, 'n')']' SysGeterrno()
call try "SysGetpwnam('root', 'x')"
call try "SysGetpwnam('root', '')"
call try "SysGetpwnam('', 'n')"
call try "SysGetpwuid(0, 'x')"
call try "SysGetpwuid(0, '')"
call try "SysGetpwuid('x', 'n')"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
::requires 'rxunixsys' library
```

```output
root 0 0
root root 0 0
1 1 []
[] 0 []
[] 0
40.1
40.1
40.1
40.1
40.1
88.907
```

## Groups

r[rxunixsys.databases.group]
`SysGetgrnam(group, option)` and `SysGetgrgid(gid, option)` answer one field of
the entry of a group, found by the name `group` or by the number `gid`: `N` the name, `G`
the number, `P` the password, which is the null string, and `M` the members, an
`Array` of the names of the users that the entry lists, which is empty where it
lists none. The group with the number `0` is named `root` on Linux. An option
that starts with another character, the null string, and a `group` that is the
null string raise `SYNTAX 40.1`.

```rexx
say SysGetgrnam('root', 'n') SysGetgrnam('root', 'g') SysGetgrgid(0, 'n') SysGetgrgid(0, 'G')
say '['SysGetgrgid(0, 'p')']' SysGetgrnam('root', 'm')~class~id SysGetgrgid(0, 'M')~class~id
call try "SysGetgrnam('root', 'x')"
call try "SysGetgrnam('root', '')"
call try "SysGetgrnam('', 'n')"
call try "SysGetgrgid(0, 'x')"
call try "SysGetgrgid('x', 'n')"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
::requires 'rxunixsys' library
```

```output
root 0 root 0
[] Array Array
40.1
40.1
40.1
40.1
88.907
```

## Services

r[rxunixsys.databases.service]
`SysGetservbyname(name, protocol, option)` and `SysGetservbyport(port, protocol,
option)` answer one field of the entry of a network service in the database of
services, found by its name or one of its aliases, or by its port number, for a
protocol such as `tcp` or `udp`: `N` the official name, `P` the port and `A` the
aliases, an `Array`. The answer is the null string where there is no such service, whatever
`option` is. A name or a protocol that is the null string, and a port outside `1`
to `65534`, raise `SYNTAX 40.1`; a service that is found and an option that
starts with another character raise it too. The three arguments are all
required.

```rexx
say '['SysGetservbyname('c1zqnosvc', 'tcp', 'p')']' SysGeterrno() '['SysGetservbyname('c1zqnosvc', 'tcp', 'x')']'
say '['SysGetservbyname('c1zqnosvc', 'c1zqnoproto', 'n')']'
say '['SysGetservbyport(65000, 'c1zqnoproto', 'n')']' SysGeterrno()
call try "SysGetservbyname('', 'tcp', 'p')"
call try "SysGetservbyname('c1zqnosvc', '', 'p')"
call try "SysGetservbyname('c1zqnosvc', 'tcp')"
call try "SysGetservbyport(0, 'tcp', 'n')"
call try "SysGetservbyport(65535, 'tcp', 'n')"
call try "SysGetservbyport(-1, 'tcp', 'n')"
call try "SysGetservbyport(80, '', 'n')"
call try "SysGetservbyport(80, 'tcp')"
exit
try:
  signal on syntax name e
  interpret 'say' arg(1)
  return
e: say condition('o')~code
  return
::requires 'rxunixsys' library
```

```output
[] 0 []
[]
[] 0
40.1
40.1
88.901
40.1
40.1
40.1
40.1
88.901
```
