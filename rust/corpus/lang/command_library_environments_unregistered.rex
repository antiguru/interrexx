/* Packages whose loaders run but register no command environment: an
   environment named after one of them is unregistered, rc 30 and FAILURE,
   as is HostEmu while the hostemu package is not loaded. */
call on failure name failed
address rxsockc 'hello'
say 'rxsockc' rc .rs
address RXSOCKC 'hello'
say 'RXSOCKC' rc .rs
address rxunixsysc 'hello'
say 'rxunixsysc' rc .rs
address HostEmu 'hello'
say 'HostEmu' rc .rs
exit
failed:
  say 'failure' condition('d') rc
  return
::requires "rxsockc" LIBRARY
::requires "rxunixsysc" LIBRARY
