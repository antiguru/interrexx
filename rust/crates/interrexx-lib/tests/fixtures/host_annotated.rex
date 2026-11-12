/* a host that annotates a member of Alpha */
::class Alpha public
::method hello
  return 'host hello'
::annotate method hello author 'x'
::method other
  return 1
