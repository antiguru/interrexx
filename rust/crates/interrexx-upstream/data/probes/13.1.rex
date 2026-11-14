/* probe for 13.1 */
signal on syntax name raised
p = .Package~new('probe', .Array~of('b'||'c3'x||'a4'x||'c = 2'))
say 'raised nothing'; exit
raised: c = condition('O'); say c~code; do i = 1 to c~additional~items; say i c~additional[i]; end
