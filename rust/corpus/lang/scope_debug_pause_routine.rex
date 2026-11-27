/* A routine called from a line typed at a debug pause runs untraced: its own
   TRACE ?R shows none of its clauses and takes no stop in it, and the next
   stop is the caller's (Deviation 182, r[env.debugging.deviation.182]). */
trace ?r
x = 1
say 'main'
exit
::routine s
  trace ?r
  y = 2
  say 'in s'
  return
