/* SPDX-License-Identifier: MIT OR Apache-2.0 */
/* spec: util.windows.console-cursor */
if SysCurPos() \== '0 0' then exit 10
if SysTextScreenRead(0, 0, 1) \== 'W' then exit 11
if SysCurPos(1, 1) \== '0 0' then exit 12
if SysCurPos() \== '1 1' then exit 13
if SysCurState('OFF') <> 0 then exit 14
if SysCurState('ON') <> 0 then exit 15
/* spec: util.windows.interaction */
if SysGetKey('NOECHO') \== 'K' then exit 16
if SysCurPos() \== '1 1' then exit 17
