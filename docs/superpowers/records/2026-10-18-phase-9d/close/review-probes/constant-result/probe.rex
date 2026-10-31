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
