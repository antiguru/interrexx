/* probe for 15.6 */
signal on syntax name raised
p = .Package~new('probe', .Array~of('say c2x(''101 01''b)'))
say 'raised nothing'; exit
raised: c = condition('O'); say c~code; do i = 1 to c~additional~items; say i c~additional[i]; end
