/* An argument with no string value is refused before the extension runs, and
   the refusal is reported against the calling line, below the method's
   compiled frame. */
r = .Re~new('a*b')
say r~does(.object~new)

::requires 're.cls'
