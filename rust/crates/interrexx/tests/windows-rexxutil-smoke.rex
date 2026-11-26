/* SPDX-License-Identifier: MIT OR Apache-2.0 */
use strict arg root
/* spec: util.windows.version */
windowsVersion = value('REXX_TEST_WINDOWS_VERSION', , 'ENVIRONMENT')
if SysVersion() \== windowsVersion | SysWinVer() \== windowsVersion then exit 47
/* spec: util.windows.system-paths */
systemDir = value('REXX_TEST_SYSTEM_DIRECTORY', , 'ENVIRONMENT')
bootDrive = value('REXX_TEST_BOOT_DRIVE', , 'ENVIRONMENT')
if SysBootDrive() \== bootDrive then exit 17
if SysSystemDirectory() \== systemDir then exit 18
/* spec: util.windows.volume */
numeric digits 20
currentDrive = value('REXX_TEST_CURRENT_DRIVE', , 'ENVIRONMENT')
driveRoot = translate(currentDrive, 'abcdefghijklmnopqrstuvwxyz', 'ABCDEFGHIJKLMNOPQRSTUVWXYZ') || '\'
driveFormat = value('REXX_TEST_VOLUME_FORMAT', , 'ENVIRONMENT')
driveLabel = value('REXX_TEST_VOLUME_LABEL', , 'ENVIRONMENT')
driveSize = value('REXX_TEST_VOLUME_SIZE', , 'ENVIRONMENT')
if SysFileSystemType() \== driveFormat | SysFileSystemType(driveRoot) \== driveFormat then exit 32
if SysVolumeLabel() \== driveLabel | SysVolumeLabel(driveRoot) \== driveLabel then exit 33
parse value SysDriveInfo(driveRoot) with actualDrive free total label
if actualDrive \== currentDrive | total \== driveSize | label \== driveLabel then exit 34
if \datatype(free, 'Whole') then exit 35
if free < 0 | free > total then exit 36
/* spec: util.windows.drive-map */
used = value('REXX_TEST_USED_DRIVES', , 'ENVIRONMENT')
free = value('REXX_TEST_FREE_DRIVES', , 'ENVIRONMENT')
if SysDriveMap('a:', 'used') \== used then exit 48
if SysDriveMap('A:', 'FREE') \== free then exit 49
if wordpos(bootDrive, SysDriveMap()) = 0 then exit 50
if wordpos(bootDrive, SysDriveMap('A:', 'LOCAL')) = 0 then exit 51
if SysDriveInfo('?:') \== '' | SysFileSystemType('?:') \== '' | SysVolumeLabel('?:') \== '' then exit 37
if SysLoadFuncs() <> 0 | SysDropFuncs() <> 0 | SysUtilVersion() = "" then exit 12
if \datatype(SysCls(), "Whole") then exit 16
dir = root || '\rexxutil-smoke'
source = dir || '\source'
copy = dir || '\copy'
moved = dir || '\moved'
if SysMkDir(dir) <> 0 then exit 1
if \SysIsFileDirectory(dir) | SysIsFileLink(dir) then exit 2
call lineout source, 'content'
call stream source, 'c', 'close'
if SysFileSearch('CONTENT', source, 'hits.', 'n') <> 0 then exit 38
if hits.0 <> 1 | hits.1 \== '1 content' then exit 39
hits = .array~of('discard me')
if SysFileSearch('CONTENT', source, hits, 'C') <> 0 | hits~items <> 0 then exit 40
if SysFileSearch('content', dir, hits) <> 3 then exit 41
call value 'REXX_TEST_SEARCH_PATH', dir || '\missing;' || '"' || dir || '"', 'ENVIRONMENT'
if SysSearchPath('REXX_TEST_SEARCH_PATH', 'source', 'N') \== source then exit 42
if SysSearchPath('REXX_TEST_SEARCH_PATH', source) \== source then exit 43
if SysSearchPath('REXX_TEST_SEARCH_PATH', 'sourc*') \== '' then exit 44
savedDirectory = directory()
selected = directory(dir)
if selected == '' then exit 52
if SysSearchPath('REXX_TEST_MISSING_SEARCH_PATH', 'source') \== selected || '\source' then exit 45
if SysSearchPath('REXX_TEST_MISSING_SEARCH_PATH', 'source', 'N') \== '' then exit 46
call directory savedDirectory
if length(SysGetFileDateTime(source, 'Creation')) <> 19 then exit 13
if SysGetFileDateTime(source) <> SysGetFileDateTime(source, 'Write') then exit 14
/* spec: util.syssetfiledatetime.windows */
created = SysGetFileDateTime(source, 'C')
accessed = SysGetFileDateTime(source, 'A')
if SysSetFileDateTime(source, '2004-12-17', '11:12:13') <> 0 then exit 19
if SysGetFileDateTime(source) \== '2004-12-17 11:12:13' then exit 20
if SysSetFileDateTime(source, , '04:05:06') <> 0 then exit 21
if SysSetFileDateTime(source, '2005-09-14') <> 0 then exit 22
if SysSetFileDateTime(source, '2005-02-29') <> -1 then exit 23
if SysGetFileDateTime(source) \== '2005-09-14 04:05:06' then exit 24
if SysGetFileDateTime(source, 'C') \== created then exit 25
if SysGetFileDateTime(source, 'A') \== accessed then exit 26
if SysSetFileDateTime(source) <> 0 then exit 30
if SysGetFileDateTime(source) <= '2005-09-14 04:05:06' then exit 31
if SysSetFileDateTime(dir, '2004-12-17', '11:12:13') <> 0 then exit 27
if SysGetFileDateTime(dir) \== '2004-12-17 11:12:13' then exit 28
if SysFileCopy(source, copy) <> 0 then exit 3
if SysFileMove(copy, moved) <> 0 then exit 4
if SysFileMove(source, moved) = 0 then exit 5
if linein(moved) <> 'content' then exit 6
call stream moved, 'c', 'close'
if SysFileDelete(source) <> 0 | SysFileDelete(moved) <> 0 then exit 7
/* spec: util.windows.file-name-spelling */
fullName = dir || '\Some Long Name.txt'
call lineout fullName, 'name check'
call stream fullName, 'c', 'close'
shortFull = SysGetShortPathName(fullName)
if shortFull == '' then exit 73
if translate(SysGetLongPathName(shortFull)) \== translate(fullName) then exit 74
savedDirectory = directory()
if directory(dir) == '' then exit 75
shortRelative = SysGetShortPathName('Some Long Name.txt')
if shortRelative == '' | pos('\', shortRelative) <> 0 then exit 76
if translate(SysGetLongPathName(shortRelative)) \== 'SOME LONG NAME.TXT' then exit 77
rootName = substr(fullName, 3)
if translate(SysGetLongPathName(rootName)) \== translate(rootName) then exit 80
if SysGetLongPathName('gone.txt') \== '' | SysGetShortPathName('gone.txt') \== '' then exit 78
call directory savedDirectory
if SysFileDelete(fullName) <> 0 then exit 79
caseParent = value('REXX_TEST_CASE_PARENT', , 'ENVIRONMENT')
if caseParent \== '' then do
  if directory(caseParent || '\Foo') == '' then exit 81
  if linein('entry.txt') \== 'upper' then exit 82
  call stream 'entry.txt', 'c', 'close'
  longCase = SysGetLongPathName('..\foo\entry.txt')
  if longCase == '' then exit 83
  if linein(longCase) \== 'lower' then exit 84
  call stream longCase, 'c', 'close'
  shortCase = SysGetShortPathName('..\foo\entry.txt')
  if shortCase == '' then exit 85
  if linein(shortCase) \== 'lower' then exit 86
  call stream shortCase, 'c', 'close'
  call directory savedDirectory
end
/* spec: util.sysfiletree.windows */
if SysMkDir(dir || '\tree') <> 0 then exit 87
treeFile = dir || '\tree\Mixed.TXT'
if charout(treeFile, 'data') <> 0 then exit 88
call stream treeFile, 'c', 'close'
if SysSetFileDateTime(treeFile, '2024-02-03', '14:05:06') <> 0 then exit 89
if SysFileTree(dir || '\*.txt', 'matches.', 'FSO') <> 0 then exit 90
if matches.0 <> 1 | matches.1 \== treeFile then exit 91
treeResults = .array~of('old-1', 'old-2')
if SysFileTree(dir || '\tree\*.*', treeResults, 'FLH') <> 0 then exit 92
if treeResults~items <> 1 then exit 93
if left(treeResults[1], 19) \== '2024-02-03 14:05:06' then exit 94
if word(treeResults[1], 3) \== '4' then exit 95
if right(treeResults[1], length(treeFile)) \== treeFile then exit 96
if SysFileTree(treeFile, 'matches.', 'FO', , '**+**') <> 0 then exit 97
if SysFileTree(treeFile, 'matches.', 'FO', '**+**') <> 0 then exit 98
if matches.0 <> 1 | matches.1 \== treeFile then exit 99
if SysFileTree(treeFile, 'matches.', 'F', , '**-**') <> 0 then exit 100
if SysFileTree(treeFile, 'matches.', 'FO', '**+**') <> 0 then exit 101
if matches.0 <> 0 then exit 102
if SysFileDelete(treeFile) <> 0 | SysRmDir(dir || '\tree') <> 0 then exit 103
if SysRmDir(dir) <> 0 then exit 8
missing = dir || '\missing'
if SysIsFileCompressed(missing) | SysIsFileEncrypted(missing) then exit 9
if SysIsFileNotContentIndexed(missing) | SysIsFileOffline(missing) then exit 10
if SysIsFileSparse(missing) | SysIsFileTemporary(missing) then exit 11
if SysGetFileDateTime(missing) <> -1 then exit 15
if SysSetFileDateTime(missing) <> -1 then exit 29

/* spec: util.windows.efs */
if SysWinEncryptFile(missing) <= 0 | SysWinDecryptFile(missing) <= 0 then exit 53
probe = value('REXX_TEST_EFS_PROBE', , 'ENVIRONMENT')
if probe \== '' then do
  if \SysIsFileEncrypted(probe) then exit 54
  if SysWinDecryptFile(probe) <> 0 then exit 55
  if SysIsFileEncrypted(probe) then exit 56
  if SysWinEncryptFile(probe) <> 0 then exit 57
  if \SysIsFileEncrypted(probe) then exit 58
  if linein(probe) \== 'test data' then exit 59
  call stream probe, 'c', 'close'
end

/* spec: util.systempfilename.answer */
prefix = dir || '\temporary'
temporary = SysTempFileName(prefix || 'QQ', 'Q')
if left(temporary, length(prefix)) \== prefix then exit 60
digits = substr(temporary, length(prefix) + 1)
if length(digits) <> 2 | verify(digits, '0123456789') <> 0 then exit 61
if SysFileExists(temporary) then exit 62
if SysTempFileName(dir || '\never') \== '' then exit 63

/* spec: util.windows.console-size */
size = SysTextScreenSize()
if words(size) <> 2 | size \== SysTextScreenSize('buffersize') then exit 64
if words(SysTextScreenSize('WINDOWRECT')) <> 4 then exit 65
if words(SysTextScreenSize('MAXWINDOWSIZE')) <> 2 then exit 66
if size == '0 0' then do
  if SysTextScreenSize('WINDOWRECT') \== '0 0 0 0' then exit 67
  if SysTextScreenSize('MAXWINDOWSIZE') \== '0 0' then exit 68
  if SysTextScreenSize('BUFFERSIZE', 25, 80) <= 0 then exit 69
  if SysTextScreenSize('WINDOWRECT', 0, 0, 24, 79) <= 0 then exit 70
end
if SysTextScreenSize('BUFFERSIZE', 32768, 80) <> 87 then exit 71
if SysTextScreenSize('WINDOWRECT', 0, 0, 24, 32768) <> 87 then exit 72

/* spec: util.windows.code-pages */
converted = .stem~new
if SysToUnicode('41C3A900'x, 'UTF8', 'ERR_INVALID_CHARS', converted) <> 0 then exit 110
if converted['!TEXT'] \== '4100E9000000'x then exit 111
if SysFromUnicode(converted['!TEXT'], 'UTF8', '0', , 'back.') <> 0 then exit 112
if back.!TEXT \== '41C3A900'x | back.!USEDDEFAULTCHAR <> 0 then exit 113
if SysToUnicode('FF'x, 'UTF8', 'ERR_INVALID_CHARS', converted) <> 1113 then exit 114
if converted['!TEXT'] \== '4100E9000000'x then exit 115
if SysFromUnicode('0026'x, 1252, , '!', 'back.') <> 0 then exit 116
if back.!TEXT \== '!' | back.!USEDDEFAULTCHAR <> 1 then exit 117
if SysToUnicode('', 'UTF8', , converted) <> 0 | converted['!TEXT'] \== '' then exit 118
if SysFromUnicode('', 'UTF8', , , 'back.') <> 0 then exit 119
if back.!TEXT \== '' | back.!USEDDEFAULTCHAR <> 0 then exit 162
if SysFromUnicode('4100'x, 'UTF8', , '?', 'back.') <> 0 then exit 163
if back.!TEXT \== 'A' | back.!USEDDEFAULTCHAR <> 0 then exit 164

/* spec: util.windows.ini */
iniFile = SysTempFileName(root || '\rexxutil-ini-????.ini')
if SysIni(iniFile, 'interrexx', 'test') \== 'ERROR:' then exit 120
if SysIni(iniFile, 'interrexx', 'test', 'value') \== '' then exit 121
signal on syntax name missingIniKey
call SysIni iniFile, 'interrexx'
exit 165
missingIniKey:
if condition('O')~code <> 88.901 then exit 166
signal off syntax
if SysIni(iniFile, 'interrexx', '') \== 'ERROR:' then exit 167
if SysIni(iniFile, 'INTERREXX', 'TEST') \== 'value' then exit 122
if SysIni(iniFile, 'ALL:', 'sections.') \== '' then exit 123
if sections.0 <> 1 | sections.1 \== 'interrexx' then exit 124
keys = .array~of('old1', 'old2')
if SysIni(iniFile, 'interrexx', 'ALL:', keys) \== '' then exit 125
if keys~items <> 1 | keys[1] \== 'test' then exit 126
if SysIni(iniFile, 'interrexx', 'test', 'DELETE:') \== '' then exit 127
if SysIni(iniFile, 'interrexx', 'test') \== 'ERROR:' then exit 128
if SysFileDelete(iniFile) <> 0 then exit 129

/* spec: util.windows.console-cursor */
if words(SysCurPos()) <> 2 then exit 130
if SysTextScreenRead(0, 0, 0) \== '' then exit 131
if SysTextScreenRead(-1, 0, 1) \== '' then exit 132
