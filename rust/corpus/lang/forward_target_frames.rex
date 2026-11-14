/* A FORWARD's target runs with the forwarding activation still below it on
   the activation stack: the target sees it in .context~stackFrames, with and
   without CONTINUE. A target that is a primitive method which waits (an
   EventSemaphore's WAIT with a timeout) answers the forwarding clause once
   the wait ends. */
o = .K~new
say o~plain
say o~cont
say o~waits
say o~waitscont

::CLASS K
::METHOD plain
  forward message('FRAMES')
::METHOD cont
  forward continue message('FRAMES')
  return 'cont:' result
::METHOD frames
  line = ''
  do f over .context~stackFrames
    if f~type = 'PROGRAM' then line = line f~type'@'f~line
    else line = line f~name'@'f~line
  end
  return strip(line)
::METHOD waits
  forward to (.EventSemaphore~new) message('WAIT') array(0.01)
::METHOD waitscont
  forward continue to (.EventSemaphore~new) message('WAIT') array(0.01)
  return 'waited:' result
