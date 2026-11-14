/* Table C method rows: NativeCommandHandler, instance arm -- one line per method
   corpus/docs/class-methods.txt documents on this arm, asked of the
   instance `.CommandHandler~handler('SYSTEM')` answers, in the row set's own order. That
   expression is corpus/docs/class-set.txt's committed construction
   program for this class, and carrying one is what `covered` claims.
   Derived by crates/interrexx/tests/table_c/probes.rs, which re-derives
   this file on every run and compares it in both directions. */
o = .CommandHandler~handler('SYSTEM')
say 'instance' o~hasMethod("command")
say 'instance' o~hasMethod("kind")
say 'instance' o~hasMethod("name")
