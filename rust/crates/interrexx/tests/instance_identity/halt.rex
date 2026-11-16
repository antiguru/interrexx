/* Halt through the instance from a thread that runs no Rexx code: the main activity, which the
   native call leaves to wait for that thread, and the activity `start` made both take HALT.
   spec: native.threads.halt */
call htsave
w = .htw~new
w~start('spin')
w~ready~wait
signal on halt name h
call hthalt
do i = 1 to 1000000; nop; end
say 'main not halted'
h: w~done~wait
say 'main' condition('c') '/ worker' w~how
::requires 'halttrace' LIBRARY
::class htw
::attribute ready
::attribute done
::attribute how
::method init
  expose ready done how
  ready = .eventsemaphore~new
  done = .eventsemaphore~new
  how = 'running'
::method spin unguarded
  expose ready done how
  signal on halt name h
  ready~post
  do 200
    call syssleep 0.01
  end
  how = 'not halted'
  done~post
  return
h: how = 'halted'
  done~post
