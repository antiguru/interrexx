/* probe for 35.934 */
signal on syntax name raised
p = .Package~new('probe', .Array~of('select case 1', 'when then nop', 'end'))
say 'raised nothing'; exit
raised: c = condition('O'); say c~code; do i = 1 to c~additional~items; say i c~additional[i]; end
