/* A thread that a native call of a started activity starts and joins attaches to an activity of
   its own, and a string it makes through the context its attach answered reads back. Only the
   started activity has made a native call, through a routine `RxFuncAdd` registered.
   spec: native.threads.attach */
say .joiner~new~start('go')~result
::class joiner
::method go
  call rxfuncadd 'AtJoinString', 'attach', 'AtJoinString'
  return AtJoinString()
