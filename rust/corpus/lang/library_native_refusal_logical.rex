/* A native method's refusal of its argument is reported against the calling
   line, below the method's compiled frame: a logical_t that is not exactly 0 or 1 is 34.901. */
say 'before'
t = .T~new
say t~logical(3)
say 'after'

::requires 'pk.cls'
