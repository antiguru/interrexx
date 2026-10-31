/* Untraced ADDITIONAL renders nothing for a trace line. DESCRIPTION takes
   its required string at RAISE under Deviation 83. A SYNTAX code uses the
   RC value's string value. */

signal on syntax name caught
src = .array~of('say "prologue"', '::class c', '::constant k (self)', '::method defaultname class', 'raise syntax 98.900 array("boom")', '::options trace r')
p = .package~new('bad', src)
say 'unexpected'
exit
caught:
trace off
say 'caught' condition('o')~code
say 'after'
exit
