/* A native call starts a thread that attaches, sends a message and detaches, and joins it before
   it returns: the message runs while the call blocks.
   spec: native.threads.attach */
t = .ticks~new
call AtTickJoin t
say 'ticked' t~count
::requires 'ticks.cls'
::requires 'attach' LIBRARY
