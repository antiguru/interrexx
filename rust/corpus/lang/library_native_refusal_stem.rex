/* A native method's refusal of its argument is reported against the calling
   line, below the method's compiled frame: a stem's name given to a method is 93.969. */
say 'before'
t = .T~new
say t~stem('zz')
say 'after'

::requires 'pk.cls'
