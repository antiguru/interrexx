/* Monitor. Spec rules: class.monitor.*. */

-- r[class.monitor.creation]
::class "Monitor" public

-- r[class.monitor.init]: the destinations are a queue, the current one at
-- its head and the one init set at its end.
::method init guarded
  expose destination
  use strict arg dest = .nil
  destination = .Queue~new
  destination~push(dest)

-- r[class.monitor.current]
::method current guarded
  expose destination
  if arg() > 0 then raise syntax 93.902 array(0)
  return destination~peek

-- r[class.monitor.destination]
::method destination guarded
  expose destination
  if arg() > 1 then raise syntax 93.902 array(1)
  if arg(1, 'E') then do
    use arg dest
    destination~push(dest)
  end
  else if destination~items > 0 then destination~pull
  return destination~peek

-- r[class.monitor.unknown]
::method unknown unguarded
  expose destination
  use strict arg name, args
  -- r[class.monitor.unknown.arguments], r[class.monitor.deviation.124]
  -- An array is used as it is, so its holes stay: a request for its
  -- makeArray would close them up.
  if args~isA(.Array) then list = args
  else list = args~request('ARRAY')
  if \list~isA(.Array) then raise syntax 98.946 array(args)
  -- A message keeps the holes of the argument array, which a FORWARD would
  -- close up, and an answer that is no answer stays none.
  msg = .Message~new(destination~peek, name, 'A', list)
  msg~send
  if msg~hasResult then return msg~result
