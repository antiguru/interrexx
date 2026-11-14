/* The object the attached threads send to: it counts the TICKs and answers the count, keeps the
   thread number of the last, raises SYNTAX 40 on BOOM, says what SHOW is given, on INNER detaches
   the thread running it, on SLOW notes that it started, sleeps two seconds and says it is done,
   on JOIN, unguarded so that a TICK may run meanwhile, calls AtTickJoin on itself, on NAP notes
   that it naps, sleeps a second and notes that it napped, and on LIFOA and LIFOB, after a sleep
   of two and four seconds, adds its letter to ORDER, LIFOA noting first that it started. */
::requires 'attach' LIBRARY
::class ticks public
::attribute count
::attribute thread
::attribute started
::attribute napping
::attribute napped
::attribute astarted
::attribute order
::method init
  expose count started napping napped astarted order
  count = 0
  started = 0
  napping = 0
  napped = 0
  astarted = 0
  order = ''
::method inner
  return AtDetachHere()
::method join unguarded
  return AtTickJoin(self)
::method slow unguarded
  expose started
  started = 1
  call SysSleep 2
  say 'slow done'
::method tick
  expose count thread
  count = count + 1
  thread = .context~thread
  return count
::method boom
  raise syntax 40.1 array ('BOOM')
::method show
  say 'shown' arg(1)
::method nap unguarded
  expose napping napped
  napping = 1
  call SysSleep 1
  napped = 1
::method lifoa unguarded
  expose astarted order
  astarted = 1
  call SysSleep 2
  order = strip(order 'a')
::method lifob unguarded
  expose order
  call SysSleep 4
  order = strip(order 'b')
