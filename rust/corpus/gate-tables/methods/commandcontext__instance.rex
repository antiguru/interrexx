/* Table C method rows: CommandContext, instance arm. corpus/docs/class-set.txt
   records this class as `not-covered`, because
   no construction program is committed; a bare ~new raises 93.901 on the oracle.
   So ~new raises and no line below it is reached; the row's evidence
   is that raise, which is what the row set says there is to have.
   Derived by crates/rexx-exec/tests/table_c/probes.rs, which re-derives
   this file on every run and compares it in both directions. */
o = .CommandContext~new
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
