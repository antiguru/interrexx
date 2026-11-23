/* SPDX-License-Identifier: MIT OR Apache-2.0 */
dialog = .SmokeDialog~new
if dialog~initCode <> 0 then exit 1
dialog~execute
if \dialog~handledCommand then exit 2
say 'ooDialog command event passed'
exit 0

::requires "ooDialog.cls"
::class SmokeDialog subclass UserDialog
::attribute handledCommand

::method init
  self~init:super
  self~handledCommand = .false
  if \self~createCenter(110, 50, 'InterRexx ooDialog smoke') then do
     self~initCode = 1
     return
  end

::method initDialog unguarded
  say 'HWND' self~dlgHandle

::method ok unguarded
  self~handledCommand = .true
  forward class (super)
