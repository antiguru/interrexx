/* probe for 15.2 */
signal on syntax name raised
p = .Package~new('probe', .Array~of('say c2x('' 1010''b)'))
say 'raised nothing'; exit
raised: c = condition('O'); say c~code; do i = 1 to c~additional~items; say i c~additional[i]; end
