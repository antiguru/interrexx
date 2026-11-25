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

title = 'interrexx-rexxutil-dialog-' || SysQueryProcess()
dialog = .smokeDialog~new~start('show', title)
if SysGetKey('NOECHO') \== 'H' then exit 18
if \dialog~halt then exit 20
/* The fixture must see closure before this process exits. */
call SysSleep 1.5
exit 0

::class smokeDialog
::method show
  use strict arg title
  return RxMessageBox('RexxUtil cancel smoke', title, 'OK')
