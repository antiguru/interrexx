/* SPDX-License-Identifier: MIT OR Apache-2.0 */
use strict arg root
dir = root || '\rexxutil-smoke'
source = dir || '\source'
copy = dir || '\copy'
moved = dir || '\moved'
if SysMkDir(dir) <> 0 then exit 1
if \SysIsFileDirectory(dir) | SysIsFileLink(dir) then exit 2
call lineout source, 'content'
call stream source, 'c', 'close'
if SysFileCopy(source, copy) <> 0 then exit 3
if SysFileMove(copy, moved) <> 0 then exit 4
if SysFileMove(source, moved) = 0 then exit 5
if linein(moved) <> 'content' then exit 6
call stream moved, 'c', 'close'
if SysFileDelete(source) <> 0 | SysFileDelete(moved) <> 0 then exit 7
if SysRmDir(dir) <> 0 then exit 8
