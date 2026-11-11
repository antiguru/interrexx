---
element: "Functions that exist only on Windows"
kind: concept
rexxref: "Rexx Utilities (RexxUtil); Rexx Utilities (RexxUtil) > List of Rexx Utility Functions"
---

# Functions that exist only on Windows

The Rexx Utilities of Windows include functions that make no sense on a
Unix-like system: they work with drives, volumes, printers, the console
screen, code pages and the attributes of files on Windows file systems.
interrexx does not provide them on a Unix-like system, and this page lists
them and says what a call of one of them does there. Support for them on
Windows is tracked in issue #215.

The Windows-only functions are these thirty-two:

* `RxMessageBox`, `RxWinExec`, `SysBootDrive`, `SysDriveInfo`, `SysDriveMap`,
  `SysFileSystemType`, `SysVolumeLabel`, `SysSystemDirectory`, `SysWinVer`:
  message boxes, programs, drives, volumes and the system;
* `SysCurPos`, `SysCurState`, `SysTextScreenRead`, `SysTextScreenSize`: the
  cursor and the text of the console screen;
* `SysFromUnicode`, `SysToUnicode`, `SysGetLongPathName`,
  `SysGetShortPathName`: code pages and Windows file names;
* `SysIni`, `SysShutdownSystem`, `SysSwitchSession`, `SysWaitNamedPipe`: the
  registry files, the session and named pipes;
* `SysIsFileCompressed`, `SysIsFileEncrypted`, `SysIsFileNotContentIndexed`,
  `SysIsFileOffline`, `SysIsFileSparse`, `SysIsFileTemporary`,
  `SysWinDecryptFile`, `SysWinEncryptFile`: the attributes of a file on a
  Windows file system;
* `SysWinGetPrinters`, `SysWinGetDefaultPrinter`, `SysWinSetDefaultPrinter`:
  printers.

r[util.windows.list]
None of the thirty-two Windows-only functions that the list above names is a
function of the interpreter on a Unix-like system, and a name in the list is
not different from the name of a routine that does not exist. A call of one,
as a function or by `CALL`, with any arguments and in any case, raises `SYNTAX
43.1`, as [the availability rule](r:util.index.windows) says. The arguments of
the call are evaluated first, so an error in one of them is raised in place of
`43.1`. RXFUNCQUERY, in any case of the name, answers `1`, and RXFUNCADD and
RXFUNCDROP answer `1` too ([RXFUNCQUERY](r:bif.rxfuncquery.answer),
[RXFUNCADD](r:bif.rxfuncadd.module), [RXFUNCDROP](r:bif.rxfuncdrop.provided)).
The functions of the chapter that are provided on a Unix-like system, such as
SysCls, SysGetKey, SysGetMessage and SysVersion, are not in the list:
RXFUNCQUERY answers `0` for them.

```rexx
names = 'RxMessageBox RxWinExec SysBootDrive SysCurPos SysCurState SysDriveInfo SysDriveMap',
        'SysFileSystemType SysFromUnicode SysGetLongPathName SysGetShortPathName SysIni',
        'SysIsFileCompressed SysIsFileEncrypted SysIsFileNotContentIndexed SysIsFileOffline',
        'SysIsFileSparse SysIsFileTemporary SysShutdownSystem SysSwitchSession',
        'SysSystemDirectory SysTextScreenRead SysTextScreenSize SysToUnicode SysVolumeLabel',
        'SysWaitNamedPipe SysWinDecryptFile SysWinEncryptFile SysWinGetPrinters',
        'SysWinGetDefaultPrinter SysWinSetDefaultPrinter SysWinVer'
calls = 0; calls2 = 0; queries = 0; adds = 0; drops = 0
do name over names~makeArray(' ')
  calls += tryit(name || '()')
  calls2 += tryit(name || "('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j')")
  calls2 += tryit('call' name)
  queries += (rxfuncquery(name) == 1) & (rxfuncquery(translate(name)) == 1) & (rxfuncquery(lower(name)) == 1)
  adds += (rxfuncadd(name, 'rexxutil', name) == 1)
  drops += (rxfuncdrop(name) == 1)
end
say names~words calls calls2 queries adds drops
say tryit("SysWinVer(1 / 0)", .true)
say tryit("SysWinVer(nosuchroutinec6zq())", .true)
say rxfuncquery('SysCls') rxfuncquery('SysVersion') rxfuncquery('SysGetKey') rxfuncquery('SysGetMessage')
exit
tryit: procedure
  signal on syntax name e
  interpret arg(1)
  return 0
e: if arg(2, 'E') then return condition('o')~code
  return condition('o')~code == 43.1
```

```output
32 32 64 32 32 32
42.3
43.1
0 0 0 0
```
