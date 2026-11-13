/* probe for 21.913 */
signal on syntax name raised
p = .Package~new('probe', .Array~of('::constant c 5 6'))
say 'raised nothing'; exit
raised: c = condition('O'); say c~code; do i = 1 to c~additional~items; say i c~additional[i]; end
