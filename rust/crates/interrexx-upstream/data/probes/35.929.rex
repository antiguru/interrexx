/* probe for 35.929 */
signal on syntax name raised
p = .Package~new('probe', .Array~of('if'))
say 'raised nothing'; exit
raised: c = condition('O'); say c~code; do i = 1 to c~additional~items; say i c~additional[i]; end
