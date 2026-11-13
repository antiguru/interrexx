/* Two started activities trace into one guarded destination whose LINEOUT is
   longer than a delivery's slice, with no wait inside it: the guard holds one
   delivery at a time, so no two deliveries overlap and every line arrives. */
.local~inside = 0
.local~bad = 0
.local~n = 0
.traceoutput~destination(.sink~new)
a = .tracer~new~start('run')
b = .tracer~new~start('run')
ra = a~result
rb = b~result
.traceoutput~destination(.output)
say 'lines' .local~n 'overlaps' .local~bad
::class tracer
::method run
  trace r
  do i = 1 to 5
    x = 1
  end
  trace off
  return 0
::class sink
::method lineout
  if .local~inside \= 0 then .local~bad = .local~bad + 1
  .local~inside = .local~inside + 1
  do i = 1 to 40
    nop
  end
  .local~n = .local~n + 1
  .local~inside = .local~inside - 1
  return 0
