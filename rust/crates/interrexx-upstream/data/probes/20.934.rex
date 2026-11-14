/* probe for 20.934 */
signal on syntax name raised
p = .Package~new('probe', .Array~of('do counter "x" 3', 'end'))
say 'raised nothing'; exit
raised: c = condition('O'); say c~code; do i = 1 to c~additional~items; say i c~additional[i]; end
