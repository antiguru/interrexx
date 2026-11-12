/* replacement for Alpha: keeps every member */
::class Alpha public
::method init
  expose v
  v = 2
::method hello
  return 'replaced hello'
::method after
  return 'replaced after'
