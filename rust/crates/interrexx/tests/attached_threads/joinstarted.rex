/* A thread that a native call of a started activity starts and joins attaches to an activity of
   its own, and a string it makes through the context its attach answered reads back. The main
   activity has made a native call first, so the interpreter has its root context.
   spec: native.threads.attach */
say 'main' AtDone()
say .joiner~new~start('go')~result
::requires 'attach' LIBRARY
::class joiner
::method go
  return AtJoinString()
