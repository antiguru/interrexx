/* probe for 36.901 */
signal on syntax name raised
p = .Package~new('probe', .Array~of('r = (a'))
say 'raised nothing'; exit
raised: c = condition('O'); say c~code; do i = 1 to c~additional~items; say i c~additional[i]; end
