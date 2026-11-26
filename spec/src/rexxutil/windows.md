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
them and says what a call of one of them does there. The Windows port is
tracked in issue #215.

The Windows-only functions are these:

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
None of the functions that the list above names is a
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
say tryit("SysWinVer(SysCls('x'))", .true)
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
88.922
0 0 0 0
```

## Windows system paths

r[util.windows.system-paths]
On Windows, `SysBootDrive()` answers the drive letter and colon of the
Windows installation, such as `C:`. `SysSystemDirectory()` answers the full
path of the Windows system directory. Both routines take zero arguments
and answer the null string if the system query fails.

## Windows volume information

r[util.windows.volume]
On Windows, `SysDriveInfo([drive])`, `SysFileSystemType([drive])` and
`SysVolumeLabel([drive])` query a drive letter or UNC share. With the argument
omitted they query the drive of the interpreter's current directory.
`SysDriveInfo` answers the drive identifier, unused bytes, total bytes and
volume label separated by spaces. The other two routines answer the filesystem
name and volume label respectively. They answer the null string if the drive
cannot be queried.

## Windows version

r[util.windows.version]
`SysWinVer()` and `SysVersion()` take no arguments and answer the same operating
system version information on Windows. The result starts with `Windows`, followed
by the operating system version when available.

## Windows drive map

r[util.windows.drive-map]
`SysDriveMap([drive], [option])` lists drive letters from the given letter
through `Z:`, separated by spaces and in ascending order. With both
arguments omitted it lists the assigned drives starting at `C:`. `USED`
lists assigned drives and `FREE` lists unassigned letters. `LOCAL`,
`REMOTE`, `REMOVABLE`, `CDROM` and `RAMDISK` restrict the list to drives
of the chosen type.

## Windows file encryption

r[util.windows.efs]
`SysWinEncryptFile(filename)` and `SysWinDecryptFile(filename)` set and
clear Windows filesystem encryption for the named file. Each takes a file
name and answers `0` on success or a Windows system error code on failure.

## Windows console size

r[util.windows.console-size]
`SysTextScreenSize()` or `SysTextScreenSize('BUFFERSIZE')` answers the
console's buffer size as rows and columns. `WINDOWRECT` answers the visible
rectangle as top, left, bottom and right. `MAXWINDOWSIZE` answers the maximum
window size as rows and columns for the current buffer and display. A
query without a console output buffer answers zero for each field.
`BUFFERSIZE` with rows and columns, or `WINDOWRECT` with all four coordinates,
changes that property and answers `0` on success or a Windows system error
code on failure.

## Windows file name spellings

r[util.windows.file-name-spelling]
`SysGetLongPathName(path)` and `SysGetShortPathName(path)` answer the long
or short Windows spelling of an existing file or directory. A relative name
is interpreted against the interpreter's current directory and answered as a
relative name. An absolute name is answered as an absolute name. A missing
entry or a failed Windows name query answers the null string.

## Windows code pages

r[util.windows.code-pages]
`SysToUnicode(text, [codepage], [flags], stem)` converts the text to
UTF-16 little-endian bytes. `SysFromUnicode(wide, [codepage], [flags],
[replacement], stem)` converts such bytes to the chosen code page. The
code page can be an integer or one of `ACP`, `OEMCP`, `SYMBOL`,
`THREAD_ACP`, `UTF7` or `UTF8`. An omitted or empty code page means
`OEMCP`. Empty input produces an empty result. UTF-7 and UTF-8 ignore
the replacement character.
On success both calls write the converted bytes to `!TEXT` and answer
zero. `SysFromUnicode` also writes `!USEDDEFAULTCHAR`, which is one
when Windows used the replacement character and zero otherwise. On
failure the functions answer a Windows system error code.

## Windows profile files

r[util.windows.ini]
`SysIni(file, section, key[, value])` reads a key or sets it when a value
is given. `DELETE:` as the value deletes a key and as the key deletes a
section. With `ALL:` in place of a section or key, the next argument is
a stem or array that receives the names at that level. A successful
write, deletion or enumeration answers the null string. A missing key
or failed profile operation answers `ERROR:`. Omitting the filename
or passing an empty name uses `WIN.INI`. Windows resolves a relative
profile file against its Windows directory. File names, section and key
names, and values are UTF-8. The key is required for a section-specific
call. An empty key answers `ERROR:`.

## Windows console cursor and text

r[util.windows.console-cursor]
`SysCurPos()` answers the current cursor position as row and column,
starting at zero. With both coordinates given, it returns the previous
position and moves the cursor when the new position is in the buffer. `SysCurState('ON')` and
`SysCurState('OFF')` show or hide the cursor.
`SysTextScreenRead(row, column[, length])` reads screen characters
starting at the specified cell. With no length, it reads to the end of
the screen buffer. A read outside the buffer or with a nonpositive length
answers the null string. The result is UTF-8, so its byte length can
differ from the number of cells read.

## Windows printers

r[util.windows.printers]
`SysWinGetPrinters(stemarray)` fills a stem or array with the installed
printer descriptions. Each description includes the printer, driver and
port names, separated by commas. The call answers zero on success and
one on failure. `SysWinGetDefaultPrinter()` answers the description of
the default printer, or the null string when none is available. A known
default without driver or port information is returned as its name
followed by two commas.
`SysWinSetDefaultPrinter(name)` accepts a printer name or a description. Text is UTF-8, with input
bytes that are not UTF-8 read using the active Windows ANSI code page.
It answers zero on success or a Windows system error code on failure.

## Windows interaction

r[util.windows.interaction]
`RxMessageBox` displays the supplied text and returns the selected
button's Windows identifier. These interaction routines take UTF-8 text
and read other input bytes using the active Windows ANSI code page.
`SysSwitchSession` looks up a top-level
window by title and asks Windows to bring it forward.
`SysWaitNamedPipe(name[, timeout])` waits for an available server
instance. It uses the server's default timeout for zero and waits
without a deadline for minus one. It answers zero for an available
pipe or the Windows system error code. `SysGetKey` reads one character
from the Windows keyboard with optional echo. A keyboard, pipe or
message-box wait parks its calling Rexx activity. Halting the activity
waiting for a message box closes that dialog.

## Windows processes

r[util.windows.process]
On Windows `SysQueryProcess` accepts `PID`, `TID`, `PPRIO`,
`TPRIO`, `PTIME` and `TTIME`. The default selector is `PID`.
`SysSetPriority` selects a process priority class and a thread
priority and answers a Windows status. `RxWinExec` starts a Windows
program from a command line with the requested window style. It
answers the process ID on success. Windows errors through 32 are returned
as positive numbers, and errors above 32 as negative numbers. `SysShutdownSystem` requests
shutdown or reboot through Windows using the caller's token.
