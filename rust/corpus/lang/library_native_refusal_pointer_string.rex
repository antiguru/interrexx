/* A native method's refusal of its argument is reported against the calling
   line, below the method's compiled frame: a POINTERSTRING not written after 0x is 88.919. */
say 'before'
t = .T~new
say t~pointerstring('zz')
say 'after'

::requires 'pk.cls'
