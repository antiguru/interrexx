/* probe for 10.7 */
signal on syntax name raised
p = .Package~new('probe', .Array~of('nop', '', 'select', '', 'when 1 = 1 then nop', '', 'end 1'))
say 'raised nothing'; exit
raised: c = condition('O'); say c~code; do i = 1 to c~additional~items; say i c~additional[i]; end
