/* A thread of the extension calls Halt through the instance while the UNINIT of an object the
   main program dropped runs Rexx clauses at the program's end: the UNINIT takes the HALT.
   spec: native.threads.halt */
o = .k~new
call AtHaltAfter 300
say 'main ends'
exit
::requires 'attach' LIBRARY
::class k
::method uninit
  signal on halt
  call time 'R'
  do while time('E') < 1
  end
  say 'uninit done'
  return
halt:
  say 'uninit halted'
