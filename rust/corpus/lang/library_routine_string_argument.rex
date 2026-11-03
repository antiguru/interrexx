/* A raise inside a library routine's argument conversion is re-raised in the
   routine's caller: POSITION and the report's line are the calling clause's,
   the traceback keeps the raising line. Called by its own name after
   ::REQUIRES LIBRARY, and through a ::ROUTINE bound to the entry. */
signal on syntax name s1
say RxCalcSqrt(.Raiser~new)
s1:
c = condition('O')
say 'by name' c~code c~position filespec('n', c~program) c~traceback~items
do l over c~traceback; say ' ' l; end
signal on syntax name s2
say sq(.Raiser~new)
s2:
c = condition('O')
say 'directive' c~code c~position c~traceback~items
signal off syntax
say sq(.Raiser~new)

::requires 'rxmathc' LIBRARY
::routine sq external "LIBRARY rxmathc RxCalcSqrt"

::class Raiser
::method makestring
  raise syntax 40.1
