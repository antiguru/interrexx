/* probe for 15.5 */
signal on syntax name raised
p = .Package~new('probe', .Array~of('say c2x(''4 1''x)'))
say 'raised nothing'; exit
raised: c = condition('O'); say c~code; do i = 1 to c~additional~items; say i c~additional[i]; end
