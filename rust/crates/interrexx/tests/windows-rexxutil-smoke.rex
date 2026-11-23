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
