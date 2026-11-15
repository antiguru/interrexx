/* An activity that takes a mutex back and then ends hands it to an activity
   that asks for it only after that: the program ends. The other activity
   waits on the object's variable until the mutex is taken back, so this
   order holds under any schedule. */
o = .pair~new
m = .MutexSemaphore~new
o~work(m)
o~await(2)
say 'main acquire' m~acquire
say 'main release' m~release
::class pair
::method init
  expose step
  step = 0
::method await
  expose step
  use arg wanted
  guard on when step = wanted
::method work unguarded
  expose step
  use arg m
  reply
  say 'worker acquire' m~acquire
  say 'worker release' m~release
  step = 1
  say 'worker reacquire' m~acquire
  step = 2
