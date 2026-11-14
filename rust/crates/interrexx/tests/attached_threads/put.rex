/* An attached thread puts an item, through ArrayPut, into an array whose class defines a guarded
   PUT, while the main program runs a guarded method of that array, the method that started the
   thread. The thread then sends TICK to a second object.
   spec: native.threads.attach.deviation.174 */
a = .putter~new
t = .ticks~new
a~holdput(t)
do while AtDone() < 1
  call SysSleep 0.05
end
say 'ticked' t~count
::requires 'ticks.cls'
::requires 'attach' LIBRARY
::class putter subclass array
::method holdput
  use arg other
  call AtPutTick self, other
  call time 'R'
  do until time('E') > 1
  end
::method put
  forward class (super)
