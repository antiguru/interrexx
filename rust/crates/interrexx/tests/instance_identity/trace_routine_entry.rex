call htsave
call first
call only
call later
say 'end'
exit
::requires 'halttrace' LIBRARY
::routine first
call httrace 1
say 'first'
return
::routine only
return httrace(1)
::routine later
nop
call httrace 1
say 'later'
return
