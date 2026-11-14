/* probe for 27.902 */
signal on syntax name raised
p = .Package~new('probe', .Array~of('do i = 1 by 2 by 3', 'end'))
say 'raised nothing'; exit
raised: c = condition('O'); say c~code; do i = 1 to c~additional~items; say i c~additional[i]; end
