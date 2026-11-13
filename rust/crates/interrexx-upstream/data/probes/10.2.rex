/* probe for 10.2 */
signal on syntax name raised
p = .Package~new('probe', .Array~of('do i = 1 to 2', 'end j'))
say 'raised nothing'; exit
raised: c = condition('O'); say c~code; do i = 1 to c~additional~items; say i c~additional[i]; end
