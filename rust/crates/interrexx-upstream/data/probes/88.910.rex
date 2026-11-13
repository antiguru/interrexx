/* probe 88.910 */
signal on syntax name raised
b = .mutablebuffer~new('abc'); x = b~replaceAt('x', 1, 1, '**')
say 'raised nothing'
exit
raised: c_ = condition('O'); say c_~code; do i_ = 1 to c_~additional~items; say i_ c_~additional[i_]; end
