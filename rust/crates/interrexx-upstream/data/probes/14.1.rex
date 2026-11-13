/* probe for 14.1 */
signal on syntax name raised
p = .Package~new('probe', .Array~of('nop', '', 'do', '', 'nop'))
say 'raised nothing'; exit
raised: c = condition('O'); say c~code; do i = 1 to c~additional~items; say i c~additional[i]; end
