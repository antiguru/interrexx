/* A destination's guarded LINEOUT loops for ever while a started activity is ready from its
   first second, and a thread of the extension, which never attaches, calls Halt through the
   instance twenty seconds in. LINEOUT traps the HALT and answers every later line at once, and the
   started activity has ended before it.
   spec: instr.trace.delivery.owner, conc.halt.signal */
s = .sink~new
.traceoutput~destination(s)
m = .setter~new~start('go')
call AtHaltAfter 20000
say 'start'
trace r
x = 1
trace off
say 'done, halted' s~halted
::requires 'attach' LIBRARY
::class sink
::attribute halted unguarded
::method init
  expose halted
  halted = 0
::method lineout guarded
  expose halted
  if halted then return 0
  signal on halt
  do forever; end
halt:
  halted = 1
  return 0
::class setter
::method go
  signal on halt
  call syssleep 0.2
  return
halt:
  return
