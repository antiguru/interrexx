---
element: "SysStat"
kind: chapter
syntax: "SysMkdirUnix(dir, mode); SysRmdirUnix(dir); SysUmask(mask); SysChmod(file, mode); SysChown(file, uid, gid); SysLchown(file, uid, gid); SysStat(file, option); SysAccess(file, mode); SysEuidaccess(file, mode); SysSymlink(target, link); SysLink(file, link); SysUnlink(file); SysGetdirlist(dir); SysClose(fd); SysChroot(dir)"
rexxref: "Unix extensions > File and File System Functions"
---

# Files and directories

The functions on this page make and remove directories and links, change and
read the permissions, owner and times of a file, and list a directory. What
every function of [the rxunixsys library](index.md) shares holds here too:
[the answers](r:rxunixsys.index.answer) are `0` or `-1`, and [the error
number](r:rxunixsys.index.errno) tells why a call failed. A name that is not
absolute is a name in the current directory of the program, which
[DIRECTORY](r:bif.directory.set) changes, not the one of the process. A name is
not changed by the call: the name of a symbolic link is that of the link, and
what the link names is read only where a rule says so.

## Directories

r[rxunixsys.files.mkdir]
`SysMkdirUnix(dir, mode)` makes the directory `dir` and answers `0`, or answers
`-1` where it cannot: the error is `17` for a name that exists, whatever it is, and
`2` for a name whose directory does not exist. `mode` is a whole number that is
the permissions of the new directory as the decimal value of the bits that
`SysChmod` writes as `rwxrwxrwx`, so that `493` is `rwxr-xr-x`. The permissions
that the new directory gets are `mode` less the bits of [the
mask](r:rxunixsys.files.umask). `SysRmdirUnix(dir)` removes the directory `dir`,
which must be empty, and answers `0`, or `-1` with the error `2` for a name that
does not exist, `20` for one that is not a directory and `39` for a directory that
is not empty.

```rexx
call SysUmask 18
say SysMkdirUnix('c1zqd1', 493) SysStat('c1zqd1', 'p')
say SysMkdirUnix('c1zqd2', 511) SysStat('c1zqd2', 'p')
say SysMkdirUnix('c1zqd3', 448) SysStat('c1zqd3', 'p')
call SysUmask 0
say SysMkdirUnix('c1zqd4', 511) SysStat('c1zqd4', 'p')
call SysUmask 18
say SysMkdirUnix('c1zqd1', 493) SysGeterrno()
say SysMkdirUnix('c1zqd1/c1zqd5/c1zqd6', 493) SysGeterrno()
say SysMkdirUnix('c1zqd1/c1zqd5', 493) SysMkdirUnix('c1zqd1/c1zqd5/c1zqd6', 493)
say SysRmdirUnix('c1zqd1') SysGeterrno()
call lineout 'c1zqf1', 'x'
call lineout 'c1zqf1'
say SysMkdirUnix('c1zqf1', 493) SysGeterrno()
say SysRmdirUnix('c1zqf1') SysGeterrno()
call SysFileDelete 'c1zqf1'
say SysRmdirUnix('c1zqnone') SysGeterrno()
say SysRmdirUnix('c1zqd1/c1zqd5/c1zqd6') SysRmdirUnix('c1zqd1/c1zqd5') SysRmdirUnix('c1zqd1')
say SysRmdirUnix('c1zqd2') SysRmdirUnix('c1zqd3') SysRmdirUnix('c1zqd4')
::requires 'rxunixsys' library
```

```output
0 drwxr-xr-x
0 drwxr-xr-x
0 drwx------
0 drwxrwxrwx
-1 17
-1 2
0 0
-1 39
-1 17
-1 20
-1 2
0 0 0
0 0 0
```

r[rxunixsys.files.umask]
`SysUmask(mask)` sets the mask of the permission bits that a file or directory
made by the process does not get, and answers the mask that was in force before. The mask
is a whole number, and only its low nine bits count: `18` is `022`, which holds back
the write bits of the group and of others, and `512` is `0`. The mask belongs to
the process, so that a program restores the mask it found.

```rexx
old = SysUmask(18)
say SysUmask(63) SysUmask(0) SysUmask(512)
say SysUmask(old) (SysUmask(old) == old)
::requires 'rxunixsys' library
```

```output
18 63 0
0 1
```

r[rxunixsys.files.getdirlist]
`SysGetdirlist(dir)` answers an `Array` of the names of the entries of the
directory `dir`: every entry, `.` and `..` included and files that start with a
dot too, each once and in an order that is not specified. The array is empty where
the directory cannot be listed, and the error is then `2` for a name that does not
exist and `20` for one that is not a directory.

```rexx
call SysMkdirUnix 'c1zqd1', 493
call lineout 'c1zqd1/b', 'x'
call lineout 'c1zqd1/b'
call lineout 'c1zqd1/.a', 'x'
call lineout 'c1zqd1/.a'
call SysMkdirUnix 'c1zqd1/c', 493
list = SysGetdirlist('c1zqd1')
say list~class~id list~items
sorted = list~sort
do name over sorted~allItems
  say name
end
say SysGetdirlist('c1zqd1/c')~items SysGetdirlist('c1zqd1/b')~items SysGeterrno()
say SysGetdirlist('c1zqnone')~items SysGeterrno()
call SysRmdirUnix 'c1zqd1/c'
call SysFileDelete 'c1zqd1/b'
call SysFileDelete 'c1zqd1/.a'
call SysRmdirUnix 'c1zqd1'
::requires 'rxunixsys' library
```

```output
Array 5
.
..
.a
b
c
2 0 20
0 2
```

## Permissions

r[rxunixsys.files.chmod]
`SysChmod(file, mode)` sets the permissions of the file `file`, following a
symbolic link, and answers `0`, or `-1` with the error `2` for a name that does
not exist. `mode` is a string of nine characters, three for the owner, three for
the group and three for others, each triple of `r`, `w` and `x` in this order, with
`-` for a bit that is not set. The third character of a triple may also be `s`
(set the bit and the bit of the special permission), `S` (the special permission
alone) or, in the last triple, `t` and `T` in the same way: the set-user-ID, set-group-ID and sticky bits.
A string of another length, a character that is not allowed in its place, and
the null string raise `SYNTAX 40.1`; so does a number, as `493` or `0755`, which
is not a mode here.

```rexx
call lineout 'c1zqf1', 'x'
call lineout 'c1zqf1'
call try 'rwxrwxrwx'
call try '---------'
call try 'rw-r-----'
call try 'r-xr-xr-x'
call try 'rwsrwsrwt'
call try 'rwSrwSrwT'
call try 'rws------'
call try '--------t'
call try 'RWXRWXRWX'
call try 'rw-r--r-- '
call try '-rwxr-xr-x'
call try 'rwxrwxrw'
call try 'rwxrwxrwa'
call try 'rwxrwxrwX'
call try '755'
call try ''
say SysChmod('c1zqnone', 'rw-rw-rw-') SysGeterrno()
call SysChmod 'c1zqf1', 'rw-rw-rw-'
call SysFileDelete 'c1zqf1'
exit
try:
  signal on syntax name e
  say '['arg(1)']' SysChmod('c1zqf1', arg(1)) SysStat('c1zqf1', 'p')
  return
e: say '['arg(1)']' condition('o')~code
  return
::requires 'rxunixsys' library
```

```output
[rwxrwxrwx] 0 -rwxrwxrwx
[---------] 0 ----------
[rw-r-----] 0 -rw-r-----
[r-xr-xr-x] 0 -r-xr-xr-x
[rwsrwsrwt] 0 -rwsrwsrwt
[rwSrwSrwT] 0 -rwSrwSrwT
[rws------] 0 -rws------
[--------t] 0 ---------t
[RWXRWXRWX] 40.1
[rw-r--r-- ] 40.1
[-rwxr-xr-x] 40.1
[rwxrwxrw] 40.1
[rwxrwxrwa] 40.1
[rwxrwxrwX] 40.1
[755] 40.1
[] 40.1
-1 2
```

r[rxunixsys.files.access]
`SysAccess(file, mode)` asks whether the process, with its real user and group,
may use the file `file` as `mode` says, and `SysEuidaccess(file, mode)` asks the
same of the effective user and group. `mode` is the sum of `4` for reading, `2`
for writing and `1` for executing, or `0` for asking only whether the file
exists. Both answer `0` where the access is allowed and `-1` where it is not,
with the error `13` for a refused permission and `2` for a name that does not
exist. SysAccess answers `-1` with the error `22` for a mode that has another
bit.

```rexx
call lineout 'c1zqf1', 'x'
call lineout 'c1zqf1'
call SysChmod 'c1zqf1', 'rw-rw-rw-'
say SysAccess('c1zqf1', 0) SysAccess('c1zqf1', 4) SysAccess('c1zqf1', 2) SysAccess('c1zqf1', 6)
say SysAccess('c1zqf1', 1) SysGeterrno()
say SysAccess('c1zqf1', 7) SysGeterrno()
call SysChmod 'c1zqf1', 'rwxrwxrwx'
say SysAccess('c1zqf1', 1) SysAccess('c1zqf1', 7)
say SysEuidaccess('c1zqf1', 0) SysEuidaccess('c1zqf1', 7)
call SysChmod 'c1zqf1', 'rw-rw-rw-'
say SysEuidaccess('c1zqf1', 1) SysGeterrno()
say SysAccess('c1zqnone', 0) SysGeterrno()
say SysEuidaccess('c1zqnone', 4) SysGeterrno()
say SysAccess('c1zqf1', 8) SysGeterrno()
say SysAccess('c1zqf1', 16) SysGeterrno()
call SysFileDelete 'c1zqf1'
::requires 'rxunixsys' library
```

```output
0 0 0 0
-1 13
-1 13
0 0
0 0
-1 13
-1 2
-1 2
-1 22
-1 22
```

r[rxunixsys.files.euidaccess]
`SysEuidaccess` looks only at the bits `1`, `2` and `4` of its mode, and ignores
every other bit: a mode of `8` asks as `0` does, and `9` asks as `1` does, so that
the call raises no error for them. This differs from `SysAccess`, which answers `-1` with the error `22` for any other bit.

```rexx
call lineout 'c1zqf1', 'x'
call lineout 'c1zqf1'
call SysChmod 'c1zqf1', 'rw-rw-rw-'
say SysEuidaccess('c1zqf1', 8) SysEuidaccess('c1zqf1', 16) SysEuidaccess('c1zqf1', 9) SysEuidaccess('c1zqf1', 6)
say SysAccess('c1zqf1', 8)
call SysFileDelete 'c1zqf1'
::requires 'rxunixsys' library
```

```output defect=#474
0 0 -1 0
-1
```

r[rxunixsys.files.chown]
`SysChown(file, uid, gid)` changes the owner and the group of the file `file`,
following a symbolic link, and `SysLchown(file, uid, gid)` those of the link
itself. Each answers `0` or `-1`. A `uid` or `gid` of `-1` leaves that one as it is, so that
`-1` for both changes nothing and always succeeds on a file that exists; the error is
`2` for a name that does not exist. Giving a file to another user, or to a group that
the process is not a member of, is refused unless the process is privileged, with
the error `1`.

```rexx
call lineout 'c1zqf1', 'x'
call lineout 'c1zqf1'
say SysChown('c1zqf1', -1, -1) SysLchown('c1zqf1', -1, -1)
say SysChown('c1zqf1', SysGeteuid(), SysGetegid()) SysLchown('c1zqf1', SysGeteuid(), SysGetegid())
say (SysStat('c1zqf1', 'u') == SysGeteuid()) (SysStat('c1zqf1', 'g') == SysGetegid())
say SysChown('c1zqnone', -1, -1) SysGeterrno()
say SysLchown('c1zqnone', -1, -1) SysGeterrno()
call SysFileDelete 'c1zqf1'
::requires 'rxunixsys' library
```

```output
0 0
0 0
1 1
-1 2
-1 2
```

## Information

r[rxunixsys.files.stat]
`SysStat(file, option)` answers one item of the information the system holds
about the file `file`, following a symbolic link, as `option` names it: `D` the
number of the device that holds it, `I` its inode number, `N` its number of
links, `U` and `G` the numbers of its owner and group, `R` the number of the
device it is, `S` its size in bytes, `A`, `M` and `C` the times of its last
access, modification and change of status, and `P` its permissions. A number is
written as a whole number. A time is in the local time of the program, written
`YYYY-MM-DD HH:MM:SS`. The permissions are ten characters as the `ls` command
writes them: the type of the file (`-` for a regular file, `d` for a directory, `l`
never, since a link is followed, `c`, `b`, `p` and `s` for the other types) and the
nine characters that `SysChmod` takes, with `s`, `S`, `t` and `T` for the special
permissions. The answer is the null string where the call fails, with the error
`2` for a name that does not exist.

```rexx
call lineout 'c1zqf1', 'hello'
call lineout 'c1zqf1'
call SysChmod 'c1zqf1', 'rw-r-----'
say SysStat('c1zqf1', 's') SysStat('c1zqf1', 'n') SysStat('c1zqf1', 'p')
say (SysStat('c1zqf1', 'u') == SysGeteuid())
say datatype(SysStat('c1zqf1', 'd'), 'W') datatype(SysStat('c1zqf1', 'i'), 'W') SysStat('c1zqf1', 'r')
say SysSetFileDateTime('c1zqf1', '2001-02-03', '04:05:06')
say SysStat('c1zqf1', 'm') SysStat('c1zqf1', 'Modified') SysStat('c1zqf1', 'M')
say length(SysStat('c1zqf1', 'a')) length(SysStat('c1zqf1', 'c')) verify(SysStat('c1zqf1', 'c'), '0123456789-: ')
say substr(SysStat('c1zqf1', 'a'), 5, 1) substr(SysStat('c1zqf1', 'a'), 11, 1) substr(SysStat('c1zqf1', 'a'), 14, 1)
say '['SysStat('c1zqnone', 's')']' SysGeterrno() '['SysStat('c1zqnone', 'p')']' SysGeterrno()
call SysMkdirUnix 'c1zqd1', 493
say left(SysStat('c1zqd1', 'p'), 1) (SysStat('c1zqd1', 'n') >= 2)
call SysRmdirUnix 'c1zqd1'
call SysFileDelete 'c1zqf1'
::requires 'rxunixsys' library
```

```output
6 1 -rw-r-----
1
1 1 0
0
2001-02-03 04:05:06 2001-02-03 04:05:06 2001-02-03 04:05:06
19 19 0
-   :
[] 2 [] 2
d 1
```

## Links

r[rxunixsys.files.symlink]
`SysSymlink(target, link)` makes a symbolic link named `link` that holds the text
`target`, and answers `0`, or `-1` with the error `17` for a `link` that exists.
`target` is stored as written, not resolved and not checked: it need not exist, and
a relative one is read relative to the directory of `link` when the link is used.
`SysLink(file, link)` makes the name `link` a second name of the file `file`, a hard
link, and answers `0`, or `-1` with the error `2` for a `file` that does not exist
and `17` for a `link` that exists; both names then name one file, whose number of
links is `2`. `SysUnlink(file)` removes the name `file`, a link itself when it is one,
and answers `0`, or `-1` with the error `2` for a name that does not exist and
`21` for a directory. The file goes when its last name does.

```rexx
call lineout 'c1zqf1', 'hello'
call lineout 'c1zqf1'
call SysChmod 'c1zqf1', 'rw-r--r--'
say SysSymlink('c1zqf1', 'c1zql1') SysStat('c1zql1', 's') SysStat('c1zql1', 'p')
say SysSymlink('c1zqf1', 'c1zql1') SysGeterrno()
say SysSymlink('c1zqnone', 'c1zql2') '['SysStat('c1zql2', 's')']' SysGeterrno()
say SysLink('c1zqf1', 'c1zqh1') SysStat('c1zqf1', 'n') SysStat('c1zqh1', 'n') SysStat('c1zqh1', 's')
say SysLink('c1zqf1', 'c1zqh1') SysGeterrno()
say SysLink('c1zqnone', 'c1zqh2') SysGeterrno()
say SysUnlink('c1zqh1') SysStat('c1zqf1', 'n')
say SysUnlink('c1zql1') SysUnlink('c1zql2') SysStat('c1zqf1', 's')
say SysUnlink('c1zql1') SysGeterrno()
call SysMkdirUnix 'c1zqd1', 493
say SysUnlink('c1zqd1') SysGeterrno()
call SysRmdirUnix 'c1zqd1'
say SysUnlink('c1zqf1') SysUnlink('c1zqf1') SysGeterrno()
::requires 'rxunixsys' library
```

```output
0 6 -rw-r--r--
-1 17
0 [] 2
0 2 2 6
-1 17
-1 2
0 1
0 0 6
-1 2
-1 21
0 -1 2
```

## Descriptors and the root directory

r[rxunixsys.files.close]
`SysClose(fd)` closes the file descriptor `fd` of the process and answers `0`, or
`-1` with the error `9` for a number that is not an open descriptor.
`SysChroot(dir)` asks for the directory `dir` to become the root directory of the
process and answers `0`, or `-1`: with the error `2` for a name that does not exist,
and `1` where the process may not change its root directory, which is a privilege.

```rexx
say SysClose(999) SysGeterrno()
say SysClose('-1') SysGeterrno()
say SysChroot('c1zqnone') SysGeterrno()
::requires 'rxunixsys' library
```

```output
-1 9
-1 9
-1 2
```
