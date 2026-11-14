/* probe for 20.903 */
signal on syntax name raised
p = .Package~new('probe', .Array~of('parse'))
say 'raised nothing'; exit
raised: c = condition('O'); say c~code; do i = 1 to c~additional~items; say i c~additional[i]; end
