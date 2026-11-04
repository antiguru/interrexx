/* ADDRESS ... WITH set by INTERPRET. What an INTERPRET sets is the
 * running code's own configuration, as if written in place: in an internal
 * routine it is the routine's and not its caller's (bug #2131, r13342),
 * and it stays in force after the fragment ends, its stem and USING
 * expression read in whichever frame issues the command. A configuration
 * set outside a fragment holds inside one, and one an INTERPRET nested in
 * an INTERPRET sets holds at the top. The trace shows which stem a later
 * command reads. */
address value address() with output stem m.
call withB
m.0 = 0
'echo caller'
say 'caller m.0' m.0 m.1
call withNone
m.0 = 0
interpret 'pp = 1; "echo inside"; say "in fragment m.0" m.0 m.1'
arr = .array~new
interpret 'kk = 0; address value address() with output using (arr)'
'echo one'
interpret '"echo two"'
say 'arr items' arr~items arr[1]
interpret 'interpret "address value address() with output stem deep."'
deep.0 = 0
'echo three'
interpret 'deep.0 = deep.0; "echo four"'
say 'deep.0' deep.0 deep.1 deep.2
call sub
say 'after sub deep.0' deep.0
trace i
'echo five'
trace o
exit

withB:
  interpret 'address value address() with output stem b.'
  m.0 = 0; b.0 = 0
  'echo routine'
  say 'withB b.0' b.0 b.1 'm.0' m.0
  return

withNone: procedure
  interpret 'address value address() with output stem c.'
  c.0 = 0
  'echo none'
  say 'withNone c.0' c.0 c.1
  return

sub: procedure
  deep.0 = 0
  'echo six'
  say 'sub deep.0' deep.0 deep.1
  return
