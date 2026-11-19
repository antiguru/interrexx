/* A thread that a native call of a started activity starts and joins, attached to nothing, adds
   a command environment through the instance, and a command to it from that activity reaches its
   handler. Only the started activity has made a native call, through a routine `RxFuncAdd`
   registered.
   spec: native.threads.attach */
say .joiner~new~start('go')~result
::class joiner
::method go
  call rxfuncadd 'AtJoinCommand', 'attach', 'AtJoinCommand'
  call AtJoinCommand
  address atjoined 'hello'
  return rc
