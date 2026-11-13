/* The object the attached threads send to: it counts the TICKs, keeps the thread number of the
   last, raises SYNTAX 40 on BOOM, and says what SHOW is given. */
::class ticks public
::attribute count
::attribute thread
::method init
  expose count
  count = 0
::method tick
  expose count thread
  count = count + 1
  thread = .context~thread
::method boom
  raise syntax 40.1 array ('BOOM')
::method show
  say 'shown' arg(1)
