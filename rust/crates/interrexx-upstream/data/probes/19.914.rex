/* probe for 19.914 */
signal on syntax name raised
p = .Package~new('probe', .Array~of('::attribute'))
say 'raised nothing'; exit
raised: c = condition('O'); say c~code; do i = 1 to c~additional~items; say i c~additional[i]; end
