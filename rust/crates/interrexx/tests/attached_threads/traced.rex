/* An attached thread's TICK, traced with the rest of its package, waits for the guard of an object
   whose guarded method the main program runs, the method that started the thread.
   spec: native.threads.attach */
trace o
t = .traced~new
t~hold
do while AtDone() < 1
  call SysSleep 0.05
end
say 'ticked' t~count
::requires 'attach' LIBRARY
::options trace r
::class traced
::attribute count
::method init
  expose count
  trace o
  count = 0
::method hold
  trace o
  call AtSend self, 'TICK'
  call time 'R'
  do until time('E') > 1
  end
::method tick
  expose count
  count = count + 1
