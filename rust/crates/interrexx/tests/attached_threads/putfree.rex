/* An attached thread puts an item, through ArrayPut, into an array whose class defines a guarded
   PUT that says it runs, while the main program sleeps in an unguarded method of that array, the
   method that started the thread. The thread then sends TICK to a second object.
   spec: native.threads.attach */
a = .putter~new
t = .ticks~new
a~holdput(t)
do while AtDone() < 1
  call SysSleep 0.05
end
say 'ticked' t~count 'items' a~items
::requires 'ticks.cls'
::requires 'attach' LIBRARY
::class putter subclass array
::method holdput unguarded
  use arg other
  call AtPutTick self, other
  call SysSleep 1
::method put
  say 'put runs'
  forward class (super)
