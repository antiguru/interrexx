/* The instance and application data the native API gives the main activity, the activity REPLY
   makes and one START makes: `same` where all three are the main activity's. Run with the
   argument `data`, it returns the main activity's answer. */
main = instprobe()
o = .instwatch~new
o~replied
o~done~wait
replied = o~seen
started = o~start('plain')~result
if replied == main & started == main then say 'same'
else say main '|' replied '|' started
/* The interpreter holds what a program says until the program ends or waits, so the driver's own
   line, written once the call returns, can overtake it. Handing it over here fixes the order. */
.stdout~flush
if arg(1) == 'data' then return main
::requires 'inst' LIBRARY
::class instwatch
::attribute done
::attribute seen
::method init
  expose done
  done = .eventsemaphore~new
::method replied
  expose done seen
  reply
  seen = instprobe()
  done~post
::method plain
  return instprobe()
