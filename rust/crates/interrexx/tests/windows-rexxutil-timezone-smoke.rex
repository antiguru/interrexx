/* SPDX-License-Identifier: MIT OR Apache-2.0 */
use strict arg root
/* spec: util.syssetfiledatetime.windows */
winter = root || '\winter'
summer = root || '\summer'
partial = root || '\partial'
if SysGetFileDateTime(partial) \== '2024-07-15 12:00:00' then exit 1
if SysSetFileDateTime(winter, '2024-01-15', '12:00:00') <> 0 then exit 2
if SysSetFileDateTime(summer, '2024-07-15', '12:00:00') <> 0 then exit 3
if SysSetFileDateTime(partial, '2024-01-15') <> 0 then exit 4
if SysGetFileDateTime(winter) \== '2024-01-15 12:00:00' then exit 5
if SysGetFileDateTime(summer) \== '2024-07-15 12:00:00' then exit 6
if SysGetFileDateTime(partial) \== '2024-01-15 12:00:00' then exit 7
