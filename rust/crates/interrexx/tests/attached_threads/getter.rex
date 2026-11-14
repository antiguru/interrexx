/* An attached thread's unguarded message says a line, then reads an attribute through its guarded
   getter, of an object whose guarded method the main program runs, the method that started the
   thread.
   spec: native.threads.attach.deviation.174 */
g = .holder~new
g~holdget
do while AtDone() < 1
  call SysSleep 0.05
end
say 'done'
::requires 'attach' LIBRARY
::class holder
::attribute count
::method init
  expose count
  count = 0
::method holdget
  call AtSend self, 'GETLATER'
  call time 'R'
  do until time('E') > 1
  end
::method getlater unguarded
  say 'getlater runs'
  return self~count
