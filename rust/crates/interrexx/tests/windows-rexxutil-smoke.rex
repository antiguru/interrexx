/* SPDX-License-Identifier: MIT OR Apache-2.0 */
use strict arg root
/* spec: util.windows.system-paths */
systemDir = value('REXX_TEST_SYSTEM_DIRECTORY', , 'ENVIRONMENT')
bootDrive = value('REXX_TEST_BOOT_DRIVE', , 'ENVIRONMENT')
if SysBootDrive() \== bootDrive then exit 17
if SysSystemDirectory() \== systemDir then exit 18
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
