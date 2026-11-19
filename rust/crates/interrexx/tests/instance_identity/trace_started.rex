/* SetTrace through the instance from a thread that runs no Rexx code traces the activity
   `start` made, which is waiting when it is called. spec: native.threads.trace */
call htsave
w = .htt~new
w~start('spin')
w~ready~wait
call httrace 1
w~go~post
w~done~wait
::requires 'halttrace' LIBRARY
::class htt
::attribute ready
::attribute go
::attribute done
::method init
  expose ready go done
  ready = .eventsemaphore~new
  go = .eventsemaphore~new
  done = .eventsemaphore~new
::method spin unguarded
  expose ready go done
  ready~post
  go~wait
  say 'worker ran'
  done~post
