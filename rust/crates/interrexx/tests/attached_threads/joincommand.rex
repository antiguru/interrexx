/* A thread that a native call of a started activity starts and joins, attached to nothing, adds
   a command environment through the instance, and a command to it from that activity reaches its
   handler. The main activity has made a native call first, so the interpreter has its root
   context.
   spec: native.threads.attach */
say 'main' AtDone()
say .joiner~new~start('go')~result
::requires 'attach' LIBRARY
::class joiner
::method go
  call AtJoinCommand
  address atjoined 'hello'
  return rc
