/* One REPLY per method invocation. The first one answers the sender, which is
   why `one` reaches stdout at all; the second raises 98.935 from the resumed
   body, where nothing is left to carry a status, so this is rc 0 with a
   traceback and `tail` is never reached. The second reply's own expression is
   evaluated before the check, which is the order the oracle takes -- the raise is
   in the reply instruction, past the expression's evaluation. */

say .K~m

::class K

::method m class
  reply 'one'
  reply 'two'
  say 'tail'
  return
