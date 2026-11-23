dialog = .SmokeDialog~new
if dialog~initCode <> 0 then exit 1
dialog~execute
if \dialog~handledClick then exit 2
say 'ooDialog button event passed'
exit 0

::requires "ooDialog.cls"
::class SmokeDialog subclass UserDialog
::attribute handledClick

::method init
  self~init:super
  self~handledClick = .false
  if \self~createCenter(110, 50, 'InterRexx ooDialog smoke') then do
     self~initCode = 1
     return
  end
  self~connectButtonEvent(101, 'CLICKED', 'onClick')

::method defineDialog
  self~createPushButton(101, 10, 10, 70, 15, 'DEFAULT', 'Close')

::method initDialog
  self~newPushButton(101)~click

::method onClick
  self~handledClick = .true
  self~ok
