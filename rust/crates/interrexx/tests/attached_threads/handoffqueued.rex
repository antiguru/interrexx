/* The main program holds the guard of an object, waking every 50 milliseconds, while two started
   activities queue for it, and an attached thread's POKE, which needs the guard too, arrives during
   one of those wakes. POKE gives the interpreter back before it runs anything and is sent again
   from the place its wait took, behind the two, so the guard goes to them in turn before POKE.
   spec: native.threads.attach.deviation.178 */
o = .held~new
.queued~new~start('RUN', o, 'A', 0.1)
.queued~new~start('RUN', o, 'B', 0.2)
o~hold
do while AtDone() < 1
  call SysSleep 0.05
end
::requires 'attach' LIBRARY
::class held
::method hold
  call AtSendAfter self, 'POKE', 500
  do 20
    call SysSleep 0.05
  end
  say 'hold'
::method turn
  use arg name
  say name
::method poke
  say 'poke'
::class queued
::method run
  use arg o, name, after
  call SysSleep after
  o~turn(name)
