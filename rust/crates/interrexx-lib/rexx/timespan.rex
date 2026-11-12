/* TimeSpan: a signed count of microseconds. The count lives in the object
   variable US; an instance whose init never ran has no US and is the span
   of 0 (spec: class.timespan.deviation.131). Every method sets NUMERIC DIGITS
   to 18 itself, so the caller's setting plays no part (spec:
   class.timespan.value). Every message the spec says a method sends is sent,
   in the order the spec gives; nothing else is sent to the receiver. */

::class "TimeSpan" public inherit Comparable Orderable

/* spec: class.timespan.new, class.timespan.new.checks, class.timespan.sends.new */
::method init
  expose us
  numeric digits 18
  n = arg()
  if n = 0 then raise syntax 93.901 array (1)
  if n > 5 then raise syntax 93.902 array (5)
  last = 0
  do i = 1 to n
    if arg(i, 'E') then last = i
  end
  do i = 1 to last - 1
    if \arg(i, 'E') then raise syntax 93.903 array (i)
  end
  if n = 1 then do
    /* three readings of the argument: the form, the range, the count */
    s = substr(arg(1), 1)
    if \datatype(s, 'W') then raise syntax 88.903 array (1, s)
    if abs(arg(1)) > 315537897599999999 then raise syntax 40.19 array ('TimeSpan', s, 'range')
    us = trunc(arg(1))
    return
  end
  kind = 'hms'
  if n >= 4 then kind = 'dhmsu'
  a. = 0
  do i = 1 to n
    if \arg(i, 'E') then iterate
    f = substr(kind, i, 1)
    s = substr(arg(i), 1)
    ok = datatype(s, 'W')
    if ok & f = 'd' then if length(abs(s + 0)) > 9 then ok = 0
    if \ok then do
      if f = 'd' then raise syntax 88.904 array (i, s)
      raise syntax 88.907 array (i, 0, word('0 23 59 59 999999', pos(f, 'dhmsu')), s)
    end
    if \(arg(i) >= 0) then do
      if f = 'd' then raise syntax 88.904 array (i, s)
      raise syntax 88.907 array (i, 0, word('0 23 59 59 999999', pos(f, 'dhmsu')), s)
    end
    if f \= 'd' then if \(arg(i) <= word('0 23 59 59 999999', pos(f, 'dhmsu'))) then,
      raise syntax 88.907 array (i, 0, word('0 23 59 59 999999', pos(f, 'dhmsu')), s)
    a.i = arg(i)
  end
  select
    when n = 2 then secs = a.1 * 3600 + a.2 * 60
    when n = 3 then secs = a.1 * 3600 + a.2 * 60 + substr(a.3, 1)
    otherwise secs = (a.1 * 24 + substr(a.2, 1)) * 3600 + a.3 * 60 + substr(a.4, 1)
  end
  total = secs * 1000000
  if n = 5 then total = total + substr(a.5, 1)
  if abs(total) > 315537897599999999 then raise syntax 40.19 array ('TimeSpan', total, 'range')
  us = trunc(total)

/* spec: class.timespan.from */
::method fromDays class
  numeric digits 18
  if arg() > 1 then raise syntax 93.902 array (1)
  if \arg(1, 'E') then raise syntax 93.901 array (1)
  n = arg(1)
  s = substr(n, 1)
  if \datatype(s, 'N') then raise syntax 88.902 array (1, s)
  return self~new(trunc(n * 24 * 3600 * 1000000))

::method fromHours class
  numeric digits 18
  if arg() > 1 then raise syntax 93.902 array (1)
  if \arg(1, 'E') then raise syntax 93.901 array (1)
  n = arg(1)
  s = substr(n, 1)
  if \datatype(s, 'N') then raise syntax 88.902 array (1, s)
  return self~new(trunc(n * 3600 * 1000000))

::method fromMinutes class
  numeric digits 18
  if arg() > 1 then raise syntax 93.902 array (1)
  if \arg(1, 'E') then raise syntax 93.901 array (1)
  n = arg(1)
  s = substr(n, 1)
  if \datatype(s, 'N') then raise syntax 88.902 array (1, s)
  return self~new(trunc(n * 60 * 1000000))

::method fromSeconds class
  numeric digits 18
  if arg() > 1 then raise syntax 93.902 array (1)
  if \arg(1, 'E') then raise syntax 93.901 array (1)
  n = arg(1)
  s = substr(n, 1)
  if \datatype(s, 'N') then raise syntax 88.902 array (1, s)
  return self~new(trunc(n * 1000000))

::method fromMicroseconds class
  numeric digits 18
  if arg() > 1 then raise syntax 93.902 array (1)
  if \arg(1, 'E') then raise syntax 93.901 array (1)
  n = arg(1)
  s = substr(n, 1)
  if \datatype(s, 'N') then raise syntax 88.902 array (1, s)
  return self~new(trunc(n))

/* spec: class.timespan.fromtime */
::method fromNormalTime class
  numeric digits 18
  if arg() > 1 then raise syntax 93.902 array (1)
  if \arg(1, 'E') then raise syntax 93.901 array (1)
  s = substr(arg(1), 1)
  parse var s hh ':' mm ':' ss
  ok = length(hh) = 2 & length(mm) = 2 & length(ss) = 2
  if ok then ok = verify(hh || mm || ss, '0123456789') = 0
  if ok then ok = hh <= 23 & mm <= 59 & ss <= 59
  if \ok then raise syntax 88.918 array (1, s)
  return self~new(((hh * 60 + mm) * 60 + ss) * 1000000)

::method fromCivilTime class
  numeric digits 18
  if arg() > 1 then raise syntax 93.902 array (1)
  if \arg(1, 'E') then raise syntax 93.901 array (1)
  s = substr(arg(1), 1)
  parse var s hh ':' rest
  mm = left(rest, 2)
  ap = substr(rest, 3)
  ok = (length(hh) = 1 | length(hh) = 2) & length(rest) = 4 & (ap == 'am' | ap == 'pm')
  if ok then ok = verify(hh || mm, '0123456789') = 0 & length(mm) = 2
  if ok then ok = hh >= 1 & hh <= 12 & mm <= 59
  if \ok then raise syntax 88.918 array (1, s)
  h = hh // 12
  if ap == 'pm' then h = h + 12
  return self~new((h * 60 + mm) * 60 * 1000000)

::method fromLongTime class
  numeric digits 18
  if arg() > 1 then raise syntax 93.902 array (1)
  if \arg(1, 'E') then raise syntax 93.901 array (1)
  s = substr(arg(1), 1)
  parse var s hh ':' mm ':' rest
  parse var rest ss '.' f
  ok = length(hh) = 2 & length(mm) = 2 & length(ss) = 2 & length(f) >= 1 & length(f) <= 6 & countstr('.', rest) = 1
  if ok then ok = verify(hh || mm || ss || f, '0123456789') = 0
  if ok then ok = hh <= 23 & mm <= 59 & ss <= 59
  if \ok then raise syntax 88.918 array (1, s)
  return self~new((((hh * 60 + mm) * 60 + ss) * 1000000) + left(f, 6, '0'))

/* spec: class.timespan.fromstringformat, class.timespan.deviation.132,
   class.timespan.deviation.133 */
::method fromStringFormat class
  numeric digits 18
  if arg() > 1 then raise syntax 93.902 array (1)
  if \arg(1, 'E') then raise syntax 93.901 array (1)
  t = substr(arg(1), 1)
  neg = 0
  if left(t, 1) = '-' then do
    neg = 1
    t = substr(t, 2)
  end
  ok = verify(t, '0123456789.:') = 0 & countstr(':', t) = 2
  if ok then do
    parse var t head ':' mm ':' tail
    ok = countstr('.', tail) = 1
  end
  if ok then do
    parse var tail ss '.' uu
    dd = 0
    dots = countstr('.', head)
    if dots = 1 then do
      parse var head dd '.' hh
      ok = dd \= '' & verify(dd, '0123456789') = 0 & left(dd, 1) \= '0' & length(dd) <= 7
      if ok then ok = dd <= 3652058
    end
    else do
      hh = head
      ok = dots = 0
    end
  end
  if ok then ok = length(hh) = 2 & length(mm) = 2 & length(ss) = 2 & length(uu) = 6
  if ok then ok = verify(hh || mm || ss || uu, '0123456789') = 0
  if ok then ok = hh <= 23 & mm <= 59 & ss <= 59
  if ok then do
    total = (((dd * 24 + hh) * 60 + mm) * 60 + ss) * 1000000 + uu
    if neg then do
      if total = 0 then ok = 0
      else total = -total
    end
  end
  if \ok then raise syntax 88.918 array (1, substr(arg(1), 1))
  return .TimeSpan~new(total)

/* spec: class.timespan.parts */
::method days
  expose us
  numeric digits 18
  if arg() > 0 then raise syntax 93.902 array (0)
  if datatype(us, 'W') = 0 then us = 0
  return abs(us) % 86400000000

::method hours
  expose us
  numeric digits 18
  if arg() > 0 then raise syntax 93.902 array (0)
  if datatype(us, 'W') = 0 then us = 0
  return abs(us) % 3600000000 // 24

::method minutes
  expose us
  numeric digits 18
  if arg() > 0 then raise syntax 93.902 array (0)
  if datatype(us, 'W') = 0 then us = 0
  return abs(us) % 60000000 // 60

::method seconds
  expose us
  numeric digits 18
  if arg() > 0 then raise syntax 93.902 array (0)
  if datatype(us, 'W') = 0 then us = 0
  return abs(us) % 1000000 // 60

::method microseconds
  expose us
  numeric digits 18
  if arg() > 0 then raise syntax 93.902 array (0)
  if datatype(us, 'W') = 0 then us = 0
  return abs(us) // 1000000

::method sign
  expose us
  numeric digits 18
  if arg() > 0 then raise syntax 93.902 array (0)
  if datatype(us, 'W') = 0 then us = 0
  return sign(us)

::method duration
  expose us
  numeric digits 18
  if arg() > 0 then raise syntax 93.902 array (0)
  if datatype(us, 'W') = 0 then us = 0
  c = abs(us)
  r = self~class~new(c)
  r~'!setCount'(c)
  return r

/* spec: class.timespan.total */
::method totalDays
  expose us
  numeric digits 18
  if arg() > 0 then raise syntax 93.902 array (0)
  if datatype(us, 'W') = 0 then us = 0
  return format(us / 86400000000, , , 0)

::method totalHours
  expose us
  numeric digits 18
  if arg() > 0 then raise syntax 93.902 array (0)
  if datatype(us, 'W') = 0 then us = 0
  return format(us / 3600000000, , , 0)

::method totalMinutes
  expose us
  numeric digits 18
  if arg() > 0 then raise syntax 93.902 array (0)
  if datatype(us, 'W') = 0 then us = 0
  return format(us / 60000000, , , 0)

::method totalSeconds
  expose us
  numeric digits 18
  if arg() > 0 then raise syntax 93.902 array (0)
  if datatype(us, 'W') = 0 then us = 0
  return format(us / 1000000, , , 0)

::method totalMicroseconds
  expose us
  numeric digits 18
  if arg() > 0 then raise syntax 93.902 array (0)
  if datatype(us, 'W') = 0 then us = 0
  return us

/* spec: class.timespan.string, class.timespan.sends.parts */
::method string
  expose us
  numeric digits 18
  if arg() > 0 then raise syntax 93.902 array (0)
  if datatype(us, 'W') = 0 then us = 0
  d = self~days
  h = self~hours
  m = self~minutes
  s = self~seconds
  u = self~microseconds
  text = right(h, 2, '0') || ':' || right(m, 2, '0') || ':' || right(s, 2, '0') || '.' || right(u, 6, '0')
  if d > 0 then text = d || '.' || text
  if us < 0 then text = '-' || text
  return text

::method makeString
  if arg() > 0 then raise syntax 93.902 array (0)
  return self~string

/* spec: class.timespan.add, class.timespan.sends.arithmetic */
::method addWeeks
  expose us
  numeric digits 18
  if arg() > 1 then raise syntax 93.902 array (1)
  if \arg(1, 'E') then raise syntax 93.901 array (1)
  if datatype(us, 'W') = 0 then us = 0
  n = arg(1)
  s = substr(n, 1)
  if \datatype(s, 'N') then raise syntax 88.902 array (1, s)
  amount = n * 604800000000
  if abs(amount) >= 1000000000000000000 then raise syntax 88.903 array (1, amount)
  c = trunc(us + amount)
  r = self~class~new(c)
  r~'!setCount'(c)
  return r

::method addDays
  expose us
  numeric digits 18
  if arg() > 1 then raise syntax 93.902 array (1)
  if \arg(1, 'E') then raise syntax 93.901 array (1)
  if datatype(us, 'W') = 0 then us = 0
  n = arg(1)
  s = substr(n, 1)
  if \datatype(s, 'N') then raise syntax 88.902 array (1, s)
  amount = n * 86400000000
  if abs(amount) >= 1000000000000000000 then raise syntax 88.903 array (1, amount)
  c = trunc(us + amount)
  r = self~class~new(c)
  r~'!setCount'(c)
  return r

::method addHours
  expose us
  numeric digits 18
  if arg() > 1 then raise syntax 93.902 array (1)
  if \arg(1, 'E') then raise syntax 93.901 array (1)
  if datatype(us, 'W') = 0 then us = 0
  n = arg(1)
  s = substr(n, 1)
  if \datatype(s, 'N') then raise syntax 88.902 array (1, s)
  amount = n * 3600000000
  if abs(amount) >= 1000000000000000000 then raise syntax 88.903 array (1, amount)
  c = trunc(us + amount)
  r = self~class~new(c)
  r~'!setCount'(c)
  return r

::method addMinutes
  expose us
  numeric digits 18
  if arg() > 1 then raise syntax 93.902 array (1)
  if \arg(1, 'E') then raise syntax 93.901 array (1)
  if datatype(us, 'W') = 0 then us = 0
  n = arg(1)
  s = substr(n, 1)
  if \datatype(s, 'N') then raise syntax 88.902 array (1, s)
  amount = n * 60000000
  if abs(amount) >= 1000000000000000000 then raise syntax 88.903 array (1, amount)
  c = trunc(us + amount)
  r = self~class~new(c)
  r~'!setCount'(c)
  return r

::method addSeconds
  expose us
  numeric digits 18
  if arg() > 1 then raise syntax 93.902 array (1)
  if \arg(1, 'E') then raise syntax 93.901 array (1)
  if datatype(us, 'W') = 0 then us = 0
  n = arg(1)
  s = substr(n, 1)
  if \datatype(s, 'N') then raise syntax 88.902 array (1, s)
  amount = n * 1000000
  if abs(amount) >= 1000000000000000000 then raise syntax 88.903 array (1, amount)
  c = trunc(us + amount)
  r = self~class~new(c)
  r~'!setCount'(c)
  return r

::method addMicroseconds
  expose us
  numeric digits 18
  if arg() > 1 then raise syntax 93.902 array (1)
  if \arg(1, 'E') then raise syntax 93.901 array (1)
  if datatype(us, 'W') = 0 then us = 0
  s = substr(arg(1), 1)
  if \datatype(s, 'N') then raise syntax 88.902 array (1, s)
  amount = substr(arg(1), 1)
  if abs(amount) >= 1000000000000000000 then raise syntax 88.903 array (1, amount)
  c = trunc(us + amount)
  r = self~class~new(c)
  r~'!setCount'(c)
  return r

/* spec: class.timespan.plus, class.timespan.deviation.127 */
::method '+'
  expose us
  numeric digits 18
  if arg() > 1 then raise syntax 93.902 array (1)
  if datatype(us, 'W') = 0 then us = 0
  if arg() = 0 then do
    r = self~class~new(us)
    r~'!setCount'(us)
    return r
  end
  x = arg(1)
  if x~isA(.TimeSpan) then do
    c = us + x~totalMicroseconds
    r = self~class~new(c)
    r~'!setCount'(c)
    return r
  end
  if x~isA(.DateTime) then do
    off = x~offset
    return x~class~new(x~fullDate + us, off)
  end
  raise syntax 88.914 array (1, 'TimeSpan')

/* spec: class.timespan.minus */
::method '-'
  expose us
  numeric digits 18
  if arg() > 1 then raise syntax 93.902 array (1)
  if datatype(us, 'W') = 0 then us = 0
  if arg() = 0 then do
    c = -us
    r = self~class~new(c)
    r~'!setCount'(c)
    return r
  end
  x = arg(1)
  if \x~isA(.TimeSpan) then raise syntax 93.948 array (1, 'TimeSpan')
  c = us - x~totalMicroseconds
  r = self~class~new(c)
  r~'!setCount'(c)
  return r

/* spec: class.timespan.times */
::method '*'
  expose us
  numeric digits 18
  if arg() > 1 then raise syntax 93.902 array (1)
  if \arg(1, 'E') then raise syntax 93.901 array (1)
  if datatype(us, 'W') = 0 then us = 0
  v = substr(arg(1), 1)
  c = trunc(us * v)
  r = self~class~new(c)
  r~'!setCount'(c)
  return r

/* spec: class.timespan.divide */
::method '/'
  if arg() > 1 then raise syntax 93.902 array (1)
  if \arg(1, 'E') then raise syntax 93.901 array (1)
  return self~'%'(arg(1))

::method '%'
  expose us
  numeric digits 18
  if arg() > 1 then raise syntax 93.902 array (1)
  if \arg(1, 'E') then raise syntax 93.901 array (1)
  if datatype(us, 'W') = 0 then us = 0
  v = substr(arg(1), 1)
  c = us % v
  r = self~class~new(c)
  r~'!setCount'(c)
  return r

/* spec: class.timespan.remainder */
::method '//'
  expose us
  numeric digits 18
  if arg() > 1 then raise syntax 93.902 array (1)
  if \arg(1, 'E') then raise syntax 93.901 array (1)
  if datatype(us, 'W') = 0 then us = 0
  v = substr(arg(1), 1)
  r = us // v
  if \datatype(r, 'W') then raise syntax 88.903 array (1, v)
  c = trunc(r)
  r = self~class~new(c)
  r~'!setCount'(c)
  return r

/* spec: class.timespan.compare, class.timespan.sends.compare,
   class.timespan.deviation.126 */
::method compareTo
  expose us
  numeric digits 18
  if arg() > 1 then raise syntax 93.902 array (1)
  if \arg(1, 'E') then raise syntax 93.901 array (1)
  if datatype(us, 'W') = 0 then us = 0
  x = arg(1)
  if \x~isA(.TimeSpan) then raise syntax 88.914 array (1, 'TimeSpan')
  o = x~totalMicroseconds
  if us < o then return -1
  if us > o then return 1
  return 0

/* The answer of an arithmetic method carries its count even when the init of
   a subclass does not forward to the init of TimeSpan (spec:
   class.timespan.deviation.131). */
::method '!setCount' private
  expose us
  numeric digits 18
  use strict arg count
  if abs(count) > 315537897599999999 then raise syntax 40.19 array ('TimeSpan', count, 'range')
  us = count

/* spec: class.timespan.hashcode */
::method hashCode
  expose us
  numeric digits 40
  if arg() > 0 then raise syntax 93.902 array (0)
  if datatype(us, 'W') = 0 then us = 0
  /* a 64-bit hash of the decimal text of the count, least significant byte first */
  h = 0
  do i = 1 to length(us)
    h = (h * 31 + c2d(substr(us, i, 1))) // 18446744073709551616
  end
  return reverse(d2c(h, 8))
