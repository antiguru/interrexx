/* synthetic host A: the prologue, then classes */
say 'prologue'
::class Alpha public
::method init
  expose v
  v = 1
::method hello
  return 'host hello'
::routine helper
  return 1
::method after
  return 'host after'
::class Beta public subclass Alpha
::method bye
  return 'host bye'
::class Last public
::attribute at get
::method final
  return 'host final'
