/* SPDX-License-Identifier: MIT OR Apache-2.0 */
use strict arg root
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
