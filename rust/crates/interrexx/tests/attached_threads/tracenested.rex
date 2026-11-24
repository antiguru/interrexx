/* The example of conc.locks.trace.deviation.14 with the extension loaded: the destination's
   guarded LINEOUT loops in a method that run runs until a thread of the extension, which never
   attaches, raises a flag of the extension's own twenty seconds in, while a started activity is
   ready from its first second.
   spec: conc.locks.trace.deviation.14 */
.traceoutput~destination(.sink~new)
m = .setter~new~start('go')
call AtRaiseAfter 20000
say 'start'
trace r
x = 1
trace off
say 'done' m~result
::requires 'attach' LIBRARY
::class sink
::method lineout guarded
  return self~run(.method~new('w', 'do while AtRaised() = 0; end; return 0'))
::class setter
::method go
  call syssleep 0.2
  return 'set'
