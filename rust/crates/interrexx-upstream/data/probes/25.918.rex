/* probe for 25.918 */
signal on syntax name raised
p = .Package~new('probe', .Array~of('forward arguments (1) array (2)'))
say 'raised nothing'; exit
raised: c = condition('O'); say c~code; do i = 1 to c~additional~items; say i c~additional[i]; end
