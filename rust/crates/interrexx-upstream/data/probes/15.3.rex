/* probe for 15.3 */
signal on syntax name raised
p = .Package~new('probe', .Array~of('say c2x(''4g''x)'))
say 'raised nothing'; exit
raised: c = condition('O'); say c~code; do i = 1 to c~additional~items; say i c~additional[i]; end
