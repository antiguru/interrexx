/* Table C wiring row: the .CommandHandler environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/rexx-exec/tests/table_c/probes.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .CommandHandler
say 'class-of-entry' .CommandHandler~class~id
say 'id' .CommandHandler~id
say 'class' .CommandHandler~class
say 'superclass' .CommandHandler~superClass
say 'superclasses' .CommandHandler~superClasses~makeString('L', ' ')
say 'metaclass' .CommandHandler~metaClass
say 'isa-class' .CommandHandler~isA(.Class)
