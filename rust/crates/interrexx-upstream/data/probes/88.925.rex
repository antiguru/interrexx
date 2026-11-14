/* probe 88.925 */
signal on syntax name raised
x = .validate~nonNegativeNumber('v', 'abc')
say 'raised nothing'
exit
raised: c_ = condition('O'); say c_~code; do i_ = 1 to c_~additional~items; say i_ c_~additional[i_]; end
