/* SPDX-License-Identifier: MIT OR Apache-2.0 */
parse arg first second
if first == '' | second == '' then exit 1
/* spec: util.windows.printers */
if SysWinGetPrinters('printers.') <> 0 then exit 2
description = ''
do i = 1 to printers.0
  parse var printers.i name ',' driver ',' port
  if name == second & driver <> '' & port == 'FILE:' then description = printers.i
end
if description == '' then exit 3
if SysWinSetDefaultPrinter(first) <> 0 then exit 4
answer = SysWinGetDefaultPrinter()
parse var answer name ',' driver ',' port
if name \== first | driver == '' | port \== 'FILE:' then exit 5
/* The description parser must treat a comma inside the port list as data. */
if SysWinSetDefaultPrinter(description || ',NUL:') <> 0 then exit 6
if SysWinGetDefaultPrinter() \== description then exit 7
