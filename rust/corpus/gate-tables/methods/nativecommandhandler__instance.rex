/* Table C method rows: NativeCommandHandler, instance arm. corpus/docs/class-set.txt
   records this class as `not-covered`, because
   no construction program is committed; a bare ~new raises 97.2 on the oracle.
   So ~new raises and no line below it is reached; the row's evidence
   is that raise, which is what the row set says there is to have.
   Derived by crates/interrexx/tests/table_c/probes.rs, which re-derives
   this file on every run and compares it in both directions. */
o = .NativeCommandHandler~new
say 'instance' o~hasMethod("command")
say 'instance' o~hasMethod("kind")
say 'instance' o~hasMethod("name")
