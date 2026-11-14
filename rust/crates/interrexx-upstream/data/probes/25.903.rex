/* probe for 25.903 */
signal on syntax name raised
p = .Package~new('probe', .Array~of('::routine r junk'))
say 'raised nothing'; exit
raised: c = condition('O'); say c~code; do i = 1 to c~additional~items; say i c~additional[i]; end
