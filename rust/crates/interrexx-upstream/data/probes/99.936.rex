/* probe for 99.936 */
signal on syntax name raised
p = .Package~new('probe', .Array~of('::method m external "LIBRARY x"', '  return 1'))
say 'raised nothing'; exit
raised: c = condition('O'); say c~code; do i = 1 to c~additional~items; say i c~additional[i]; end
