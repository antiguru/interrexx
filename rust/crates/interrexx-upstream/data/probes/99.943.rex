/* probe for 99.943 */
signal on syntax name raised
p = .Package~new('probe', .Array~of('::resource ''&1'' end ''&2''', 'line'))
say 'raised nothing'; exit
raised: c = condition('O'); say c~code; do i = 1 to c~additional~items; say i c~additional[i]; end
