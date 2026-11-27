/* An OPEN through the COMMAND method that the file system refuses answers
   ERROR:n and raises NOTREADY, which the caller's CALL ON or SIGNAL ON
   trap takes, or that of an internal routine; an untrapped one leaves
   COMMAND's answer as ERROR:n. ::OPTIONS NOTREADY SYNTAX does not turn it
   into an error. The OPEN method's own raise is 98.974 in an activation with
   no CALL OFF NOTREADY in force. A RAISE PROPAGATE of a command's ERROR
   returns its RESULT. Whether the stream's NOTREADY is PROPAGATED is
   unspecified (r[cond.information.propagated]), so it is not shown. */
say 'directory' stream('/', 'c', 'open read')
say 'directory write' stream('/', 'c', 'open write')
say 'missing' stream('zz_missing.dat', 'C', 'OPEN READ')
say 'missing path' stream('nodir/x', 'c', 'open read')
say 'method' .Stream~new('/')~command('open read')
call on notready name caught
say 'call on' stream('/', 'c', 'open read')
call off notready
signal on notready name signalled
say 'signal on' stream('/', 'c', 'open read')
say 'not reached'
signalled:
call show 'signalled'
say 'routine' viaRoutine()
say 'method returns' .Opener~new~open('/')
say 'open method, CALL OFF in force' .Stream~new('/')~open('read')
say 'open method' directOpen()
say 'not reached'
exit

caught:
  call show 'caught'
  return

show:
  o = condition('O')
  say arg(1) condition('C') condition('D') o~instruction o~result,
    o~additional~class~id o~additional~string
  return

viaRoutine: procedure
  signal on notready name inner
  return stream('/', 'c', 'open read')
inner:
  return 'inner' condition('O')~result

::routine directOpen
  return .Stream~new('/')~open('read')

::class Opener
::method open
  use arg name
  signal on error name failed
  'exit 7'
  return 'not reached'
failed:
  raise propagate

::options notready syntax
