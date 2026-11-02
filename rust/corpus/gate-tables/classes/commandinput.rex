/* Table C wiring row: the .CommandInput environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/table_c/probes.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .CommandInput
say 'class-of-entry' .CommandInput~class~id
say 'id' .CommandInput~id
say 'class' .CommandInput~class
say 'superclass' .CommandInput~superClass
say 'superclasses' .CommandInput~superClasses~makeString('L', ' ')
say 'metaclass' .CommandInput~metaClass
say 'isa-class' .CommandInput~isA(.Class)
