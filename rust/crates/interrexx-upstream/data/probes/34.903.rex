/* probe 34.903 */
signal on syntax name raised
r = .routine~new('agent', 'address system "true"')
r~setSecurityManager(.m~new)
r~call
say 'raised nothing'
exit
raised: c_ = condition('O'); say c_~code; do i_ = 1 to c_~additional~items; say i_ c_~additional[i_]; end
::class m
::method command
  return 5
