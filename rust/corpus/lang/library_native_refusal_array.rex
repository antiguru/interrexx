/* A native method's refusal of its argument is reported against the calling
   line, below the method's compiled frame: an object with no single-dimensional array value is 98.913. */
say 'before'
t = .T~new
say t~array(.object~new)
say 'after'

::requires 'pk.cls'
