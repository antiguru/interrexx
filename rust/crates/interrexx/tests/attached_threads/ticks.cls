/* The object the attached threads send to: it counts the TICKs and answers the count, keeps the
   thread number of the last, raises SYNTAX 40 on BOOM, says what SHOW is given, on INNER detaches
   the thread running it, on SLOW notes that it started, sleeps two seconds and says it is done,
   and on JOIN, unguarded so that a TICK may run meanwhile, calls AtTickJoin on itself.
   The rest are unguarded unless they say otherwise. NAP notes that it naps, sleeps a second and
   notes that it napped. LIFOA notes that it started, sleeps two seconds and adds `a` to ORDER,
   and LIFOB sleeps two seconds and adds `b`. HOLD, guarded, has a thread send TICK and runs Rexx
   code for a second, and BUSY, guarded, runs Rexx code for a second. SERVE has a thread send
   AWAITREPLY, runs Rexx code for a second and answers `served`, and AWAITREPLY says the result of
   the message MSG. AWAITSEM waits up to ten seconds on the event semaphore SEM and says so,
   POSTSEM posts it, and POSTNAP posts it, sleeps a second and says it ends. */
::requires 'attach' LIBRARY
::class ticks public
::attribute count
::attribute thread
::attribute started
::attribute napping
::attribute napped
::attribute astarted
::attribute order
::attribute msg
::method init
  expose count started napping napped astarted order sem
  count = 0
  started = 0
  napping = 0
  napped = 0
  astarted = 0
  order = ''
  sem = .eventsemaphore~new
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
  call SysSleep 2
  order = strip(order 'b')
::method hold
  call AtSend self, 'TICK'
  call time 'R'
  do until time('E') > 1
  end
::method serve unguarded
  call AtSend self, 'AWAITREPLY'
  call time 'R'
  do until time('E') > 1
  end
  return 'served'
::method awaitreply unguarded
  expose msg
  say 'got' msg~result
::method awaitsem unguarded
  expose sem
  sem~wait(10)
  say 'sem seen'
::method postsem unguarded
  expose sem
  sem~post
::method postnap unguarded
  expose sem
  sem~post
  call SysSleep 1
  say 'callback ends'
::method busy
  call time 'R'
  do until time('E') > 1
  end
