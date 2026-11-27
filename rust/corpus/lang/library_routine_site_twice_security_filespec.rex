/* The security manager's CALL checkpoint names FILESPEC, a routine of the
   REXX package at language level 6.06 (Deviation 179,
   r[env.security.call.deviation.179]), on every call from one expression call
   site. */
src = .array~of("do a over .array~of('N', 'E')",,
  "  say 'internal' filespec(a, '/a/b.c')",,
  "end")
g = .Routine~new('global', src)
say 'declined' g~setSecurityManager(.Audit~new)
g~call
::class Audit
::method unknown
  use arg name, args
  if name = 'CALL' then say 'event CALL' args[1]['NAME']
  return 0
