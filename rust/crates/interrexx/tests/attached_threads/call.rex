/* An attached thread calls, through CallRoutine, a routine that says a line, then reads an
   attribute through its guarded getter, of an object whose guarded method the main program runs,
   the method that started the thread. The thread then sends TICK to a second object.
   spec: native.threads.attach.deviation.178 */
h = .holder~new
t = .ticks~new
h~holdcall(.context~package~findRoutine('READCOUNT'), t)
do while AtDone() < 1
  call SysSleep 0.05
end
say 'ticked' t~count
::requires 'ticks.cls'
::requires 'attach' LIBRARY
::routine readcount
  use arg h
  say 'routine runs'
  return h~count
::class holder
::attribute count
::method init
  expose count
  count = 0
::method holdcall
  use arg routine, other
  call AtCallTick routine, self, other
  call time 'R'
  do until time('E') > 1
  end
