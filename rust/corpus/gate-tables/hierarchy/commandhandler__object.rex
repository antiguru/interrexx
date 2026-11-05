/* Table C wiring row: provide.xml's class hierarchy list indents
   CommandHandler below Object, which claims Object is PRESENT IN
   CommandHandler's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/interrexx/tests/table_c/probes.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .CommandHandler~id
say 'parent' .Object~id
supers = .CommandHandler~superClasses
edge = 0
do at = 1 to supers~items
  if supers[at]~id == .Object~id then edge = 1
end
say 'documented-edge' edge
say 'superclasses' .CommandHandler~superClasses~makeString('L', ' ')
