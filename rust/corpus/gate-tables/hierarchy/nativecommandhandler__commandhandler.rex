/* Table C wiring row: provide.xml's class hierarchy list indents
   NativeCommandHandler below CommandHandler, which claims CommandHandler is PRESENT IN
   NativeCommandHandler's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/interrexx/tests/table_c/probes.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .NativeCommandHandler~id
say 'parent' .CommandHandler~id
supers = .NativeCommandHandler~superClasses
edge = 0
do at = 1 to supers~items
  if supers[at]~id == .CommandHandler~id then edge = 1
end
say 'documented-edge' edge
say 'superclasses' .NativeCommandHandler~superClasses~makeString('L', ' ')
