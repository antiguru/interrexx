/* The object the attached threads send to: it counts the TICKs and answers the count, keeps the
   thread number of the last, raises SYNTAX 40 on BOOM, says what SHOW is given, on INNER detaches
   the thread running it, on SLOW notes that it started, sleeps two seconds and says it is done,
   and on JOIN, unguarded so that a TICK may run meanwhile, calls AtTickJoin on itself. */
::requires 'attach' LIBRARY
::class ticks public
::attribute count
::attribute thread
::attribute started
::method init
  expose count started
  count = 0
  started = 0
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
