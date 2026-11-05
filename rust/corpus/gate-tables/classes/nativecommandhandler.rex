/* Table C wiring row: the .NativeCommandHandler environment entry, asked what it
   renders as and what its class is -- questions any entry answers --
   and then the questions the class surface is wired by: ~id, ~class,
   ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived
   from corpus/docs/class-set.txt by
   crates/interrexx/tests/table_c/probes.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'entry' .NativeCommandHandler
say 'class-of-entry' .NativeCommandHandler~class~id
say 'id' .NativeCommandHandler~id
say 'class' .NativeCommandHandler~class
say 'superclass' .NativeCommandHandler~superClass
say 'superclasses' .NativeCommandHandler~superClasses~makeString('L', ' ')
say 'metaclass' .NativeCommandHandler~metaClass
say 'isa-class' .NativeCommandHandler~isA(.Class)
