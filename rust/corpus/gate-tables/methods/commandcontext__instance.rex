/* Table C method rows: CommandContext, instance arm -- one line per method
   corpus/docs/class-methods.txt documents on this arm, asked of the
   instance `gatekc()` answers, in the row set's own order. That
   expression is corpus/docs/class-set.txt's committed construction
   program for this class, and carrying one is what `covered` claims.
   `::routine gatekc; .CommandHandler~register('GATEKC', .gatekc~new); address GATEKC 'x'; return .local~gatekc; ::class gatekc subclass CommandHandler; ::method command; use arg , context; .local~gatekc = context; return 0` below the readbacks is that row's `directives` field,
   which is what the expression reads.
   Derived by crates/interrexx/tests/table_c/probes.rs, which re-derives
   this file on every run and compares it in both directions. */
o = gatekc()
say 'instance' o~hasMethod("[ ]")
say 'instance' o~hasMethod("[ ]=")
say 'instance' o~hasMethod("dropVar")
say 'instance' o~hasMethod("environment")
say 'instance' o~hasMethod("error")
say 'instance' o~hasMethod("getVar")
say 'instance' o~hasMethod("hasVar")
say 'instance' o~hasMethod("input")
say 'instance' o~hasMethod("isRedirected")
say 'instance' o~hasMethod("isValid")
say 'instance' o~hasMethod("output")
say 'instance' o~hasMethod("raise")
say 'instance' o~hasMethod("raiseError")
say 'instance' o~hasMethod("raiseFailure")
say 'instance' o~hasMethod("setVar")
say 'instance' o~hasMethod("variables")
::routine gatekc; .CommandHandler~register('GATEKC', .gatekc~new); address GATEKC 'x'; return .local~gatekc; ::class gatekc subclass CommandHandler; ::method command; use arg , context; .local~gatekc = context; return 0
