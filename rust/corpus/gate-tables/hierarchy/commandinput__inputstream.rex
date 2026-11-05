/* Table C wiring row: provide.xml's class hierarchy list indents
   CommandInput below InputStream, which claims InputStream is PRESENT IN
   CommandInput's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/interrexx/tests/table_c/probes.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .CommandInput~id
say 'parent' .InputStream~id
supers = .CommandInput~superClasses
edge = 0
do at = 1 to supers~items
  if supers[at]~id == .InputStream~id then edge = 1
end
say 'documented-edge' edge
say 'superclasses' .CommandInput~superClasses~makeString('L', ' ')
