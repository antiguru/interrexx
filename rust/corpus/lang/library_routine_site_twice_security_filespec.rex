/* The security manager's CALL checkpoint is not sent for FILESPEC, a builtin,
   at any call from one expression call site (r[env.security.call]). */
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
