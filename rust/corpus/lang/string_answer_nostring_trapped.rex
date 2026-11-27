/* SIGNAL ON NOSTRING takes a STRING answer with no string value before a
   consumer of its bytes reads it; the description is the default string
   value. */
o = .sn~new
signal on nostring name h
say length(o)
exit
h: say 'h' condition('C') '['condition('D')']' sigl; exit
::class sn
::method string
  return .directory~new
