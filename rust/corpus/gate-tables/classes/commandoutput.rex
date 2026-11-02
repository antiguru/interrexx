/* Table C wiring row: the .CommandOutput environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/table_c/probes.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .CommandOutput
say 'class-of-entry' .CommandOutput~class~id
say 'id' .CommandOutput~id
say 'class' .CommandOutput~class
say 'superclass' .CommandOutput~superClass
say 'superclasses' .CommandOutput~superClasses~makeString('L', ' ')
say 'metaclass' .CommandOutput~metaClass
say 'isa-class' .CommandOutput~isA(.Class)
