/* probe for 99.932 */
signal on syntax name raised
p = .Package~new('probe', .Array~of('::constant k 1', '::constant k (1+2)'))
say 'raised nothing'; exit
raised: c = condition('O'); say c~code; do i = 1 to c~additional~items; say i c~additional[i]; end
