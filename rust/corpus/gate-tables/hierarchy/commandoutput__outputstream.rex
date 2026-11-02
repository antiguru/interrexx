/* Table C wiring row: provide.xml's class hierarchy list indents
   CommandOutput below OutputStream, which claims OutputStream is PRESENT IN
   CommandOutput's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/table_c/probes.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .CommandOutput~id
say 'parent' .OutputStream~id
supers = .CommandOutput~superClasses
edge = 0
do at = 1 to supers~items
  if supers[at]~id == .OutputStream~id then edge = 1
end
say 'documented-edge' edge
say 'superclasses' .CommandOutput~superClasses~makeString('L', ' ')
