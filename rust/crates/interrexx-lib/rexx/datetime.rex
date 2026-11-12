/* The DateTime class. Spec rules: class.datetime.*.

   An instance holds two whole numbers: STAMP, the microseconds from
   0001-01-01T00:00:00.000000 to its fields (what fullDate answers), and ZONE,
   its offset east of UTC in microseconds, a whole number of minutes. The
   formats are written and read by the DATE and TIME builtins, whose F
   informat reads a STAMP. An instance whose init never ran has neither
   variable and is the first moment with the offset 0
   (r[class.datetime.deviation.131]): every method that reads them first
   supplies those values, and a method that answers an instance it made with
   new also gives it its moment through the private setTimestamp, which holds
   when the init of a subclass does not forward to this one.
   The methods do their own arithmetic under NUMERIC DIGITS 18, which holds
   every count of the class exactly, so the caller's setting plays no part
   (r[class.datetime.numeric]).

   The internal routine `whole`, repeated in each method that reads a
   whole-number argument, applies r[conv.whole.form] and r[conv.whole.digits]
   (r[class.datetime.deviation.130]) to a string already read: it answers the
   value, or the null string when the string is none. Each caller reads the
   string value of the argument as often as r[class.datetime.sends.new]
   says. */

::class "DateTime" public subclass Object inherit Comparable Orderable

-- r[class.datetime.limits]
::method minDate class
  use strict arg
  return self~new(1, 1, 1, 0)

::method maxDate class
  use strict arg
  return self~new(9999, 12, 31, 23, 59, 59, 999999, 0)

-- r[class.datetime.today]. The offset is read by init, whose 88.907 is this
-- method's 88.918 (r[class.datetime.offset]).
::method today class
  use strict arg offset = 0
  numeric digits 18
  parse value date('B') time('O') with days host
  if arg(1, 'E') then given = offset
  else given = host / 60000000
  signal on syntax name refused
  return self~new(days * 86400000000, given)
refused:
  if condition('O')~code == '88.907' then
    raise syntax 88.918 array('offset', given)
  raise propagate

-- r[class.datetime.fromdate]: the B form.
::method fromBaseDate class
  use strict arg days, offset = 0
  numeric digits 18
  n = whole('' || days)
  if n == '' then raise syntax 88.918 array('date', days)
  if n < 0 | n > 3652058 then raise syntax 88.918 array('date', days)
  if arg(2, 'E') then given = offset
  else given = time('O') / 60000000
  signal on syntax name refused
  return self~new(n * 86400000000, given)
refused:
  if condition('O')~code == '88.907' then
    raise syntax 88.918 array('offset', given)
  raise propagate
whole: procedure
  parse arg s
  if \datatype(s, 'N') then return ''
  numeric digits 18
  v = s + 0
  if abs(v) >= 1e18 then return ''
  if v // 1 \= 0 then return ''
  return v % 1

-- r[class.datetime.fromdate]: the E form.
::method fromEuropeanDate class
  use strict arg text, separator = '', offset = 0
  numeric digits 18
  signal on syntax name unreadable
  if arg(2, 'E') then days = date('B', text, 'E', , separator)
  else days = date('B', text, 'E')
  signal off syntax
  if arg(3, 'E') then given = offset
  else given = time('O') / 60000000
  signal on syntax name refused
  return self~new(days * 86400000000, given)
unreadable:
  raise syntax 88.918 array('date', text)
refused:
  if condition('O')~code == '88.907' then
    raise syntax 88.918 array('offset', given)
  raise propagate

-- r[class.datetime.fromdate]: the N form.
::method fromNormalDate class
  use strict arg text, separator = '', offset = 0
  numeric digits 18
  signal on syntax name unreadable
  if arg(2, 'E') then days = date('B', text, 'N', , separator)
  else days = date('B', text, 'N')
  signal off syntax
  if arg(3, 'E') then given = offset
  else given = time('O') / 60000000
  signal on syntax name refused
  return self~new(days * 86400000000, given)
unreadable:
  raise syntax 88.918 array('date', text)
refused:
  if condition('O')~code == '88.907' then
    raise syntax 88.918 array('offset', given)
  raise propagate

-- r[class.datetime.fromdate]: the O form.
::method fromOrderedDate class
  use strict arg text, separator = '', offset = 0
  numeric digits 18
  signal on syntax name unreadable
  if arg(2, 'E') then days = date('B', text, 'O', , separator)
  else days = date('B', text, 'O')
  signal off syntax
  if arg(3, 'E') then given = offset
  else given = time('O') / 60000000
  signal on syntax name refused
  return self~new(days * 86400000000, given)
unreadable:
  raise syntax 88.918 array('date', text)
refused:
  if condition('O')~code == '88.907' then
    raise syntax 88.918 array('offset', given)
  raise propagate

-- r[class.datetime.fromdate]: the S form.
::method fromStandardDate class
  use strict arg text, separator = '', offset = 0
  numeric digits 18
  signal on syntax name unreadable
  if arg(2, 'E') then days = date('B', text, 'S', , separator)
  else days = date('B', text, 'S')
  signal off syntax
  if arg(3, 'E') then given = offset
  else given = time('O') / 60000000
  signal on syntax name refused
  return self~new(days * 86400000000, given)
unreadable:
  raise syntax 88.918 array('date', text)
refused:
  if condition('O')~code == '88.907' then
    raise syntax 88.918 array('offset', given)
  raise propagate

-- r[class.datetime.fromdate]: the U form.
::method fromUsaDate class
  use strict arg text, separator = '', offset = 0
  numeric digits 18
  signal on syntax name unreadable
  if arg(2, 'E') then days = date('B', text, 'U', , separator)
  else days = date('B', text, 'U')
  signal off syntax
  if arg(3, 'E') then given = offset
  else given = time('O') / 60000000
  signal on syntax name refused
  return self~new(days * 86400000000, given)
unreadable:
  raise syntax 88.918 array('date', text)
refused:
  if condition('O')~code == '88.907' then
    raise syntax 88.918 array('offset', given)
  raise propagate

-- r[class.datetime.fromordinal]. The answer is a plain DateTime: nothing is
-- sent to the receiver (r[class.datetime.sends.new]).
::method fromOrdinalDate class
  use strict arg text, offset = 0
  numeric digits 18
  -- r[class.datetime.fromdate.argument]: the argument is sent `contains`
  -- first, and its answer picks the form.
  if text~contains('-') then dashed = 1
  else dashed = 0
  s = '' || text
  select
    when \dashed & length(s) == 7 then parse var s year 5 day
    when dashed & length(s) == 8 & substr(s, 5, 1) == '-' then
      parse var s year 5 . 6 day
    otherwise raise syntax 88.918 array('date', text)
  end
  if verify(year || day, '0123456789') \= 0 then
    raise syntax 88.918 array('date', text)
  if year == '0000' then raise syntax 88.918 array('date', text)
  first = date('B', year || '0101', 'S')
  size = date('B', year || '1231', 'S') - first + 1
  if day < 1 | day > size then raise syntax 88.918 array('date', text)
  if arg(2, 'E') then given = offset
  else given = time('O') / 60000000
  signal on syntax name refused
  return .DateTime~new((first + day - 1) * 86400000000, given)
refused:
  if condition('O')~code == '88.907' then
    raise syntax 88.918 array('offset', given)
  raise propagate

-- r[class.datetime.fromweek]. The offset argument is ignored; the messages
-- are those of r[class.datetime.sends.new].
::method fromWeekNumberDate class
  use strict arg text, offset = 0
  numeric digits 18
  -- r[class.datetime.fromdate.argument]: as for fromOrdinalDate.
  if text~contains('-') then dashed = 1
  else dashed = 0
  s = '' || text
  select
    when \dashed & length(s) == 8 then parse var s year 5 w 6 week 8 day
    when dashed & length(s) == 10 then do
      parse var s year 5 d1 6 w 7 week 9 d2 10 day
      if d1 \== '-' | d2 \== '-' then raise syntax 88.918 array('date', text)
    end
    otherwise raise syntax 88.918 array('date', text)
  end
  if w \== 'W' then raise syntax 88.918 array('date', text)
  if verify(year || week || day, '0123456789') \= 0 then
    raise syntax 88.918 array('date', text)
  if year == '0000' | week == '00' | day < 1 | day > 7 then
    raise syntax 88.918 array('date', text)
  january = self~new(year, 1, 4)
  if week > january~weeksInYear then raise syntax 88.918 array('date', text)
  weekday = january~weekDay
  signal on syntax name outside
  return january~addDays((week - 1) * 7 + day - weekday)
outside:
  if condition('O')~code == '40.19' then raise syntax 88.918 array('date', text)
  raise propagate

-- r[class.datetime.fromtime]: the N form.
::method fromNormalTime class
  use strict arg text, offset = 0
  numeric digits 18
  signal on syntax name unreadable
  count = time('F', text, 'N')
  signal off syntax
  if arg(2, 'E') then given = offset
  else given = time('O') / 60000000
  signal on syntax name refused
  return self~new(count, given)
unreadable:
  raise syntax 88.918 array('time', text)
refused:
  if condition('O')~code == '88.907' then
    raise syntax 88.918 array('offset', given)
  raise propagate

-- r[class.datetime.fromtime]: the C form.
::method fromCivilTime class
  use strict arg text, offset = 0
  numeric digits 18
  signal on syntax name unreadable
  count = time('F', text, 'C')
  signal off syntax
  if arg(2, 'E') then given = offset
  else given = time('O') / 60000000
  signal on syntax name refused
  return self~new(count, given)
unreadable:
  raise syntax 88.918 array('time', text)
refused:
  if condition('O')~code == '88.907' then
    raise syntax 88.918 array('offset', given)
  raise propagate

-- r[class.datetime.fromtime]: the L form.
::method fromLongTime class
  use strict arg text, offset = 0
  numeric digits 18
  signal on syntax name unreadable
  count = time('F', text, 'L')
  signal off syntax
  if arg(2, 'E') then given = offset
  else given = time('O') / 60000000
  signal on syntax name refused
  return self~new(count, given)
unreadable:
  raise syntax 88.918 array('time', text)
refused:
  if condition('O')~code == '88.907' then
    raise syntax 88.918 array('offset', given)
  raise propagate

-- r[class.datetime.fromticks]
::method fromTicks class
  use strict arg ticks, offset = 0
  numeric digits 18
  n = whole('' || ticks)
  if n == '' then raise syntax 88.918 array('ticks', ticks)
  if n < -62135596849 | n > 253402300799 then
    raise syntax 88.918 array('ticks', ticks)
  count = max(0, (n + 62135596800) * 1000000)
  if arg(2, 'E') then given = offset
  else given = time('O') / 60000000
  signal on syntax name refused
  return self~new(count, given)
refused:
  if condition('O')~code == '88.907' then
    raise syntax 88.918 array('offset', given)
  raise propagate
whole: procedure
  parse arg s
  if \datatype(s, 'N') then return ''
  numeric digits 18
  v = s + 0
  if abs(v) >= 1e18 then return ''
  if v // 1 \= 0 then return ''
  return v % 1

-- r[class.datetime.fromiso]
::method fromIsoDate class
  use strict arg text, offset = 0
  numeric digits 18
  signal on syntax name unreadable
  parse value text with day 'T' clock
  count = date('B', day, 'I') * 86400000000 + time('F', clock, 'L')
  signal off syntax
  if arg(2, 'E') then given = offset
  else given = time('O') / 60000000
  signal on syntax name refused
  return self~new(count, given)
unreadable:
  raise syntax 88.918 array('date', text)
refused:
  if condition('O')~code == '88.907' then
    raise syntax 88.918 array('offset', given)
  raise propagate

-- r[class.datetime.fromutciso], r[class.datetime.deviation.132]: the string
-- is the date and time of fromIsoDate, the fraction optional, then `Z` or a
-- sign and `hh:mm` of at most 15:00. new gets the offset in minutes.
::method fromUtcIsoDate class
  use strict arg text
  numeric digits 18
  s = '' || text
  if right(s, 1) == 'Z' then do
    body = left(s, length(s) - 1)
    minutes = 0
  end
  else do
    if length(s) < 6 then raise syntax 88.918 array('date', text)
    parse value right(s, 6) with sign 2 hours 4 colon 5 mins
    body = left(s, length(s) - 6)
    if sign \== '+' & sign \== '-' then raise syntax 88.918 array('date', text)
    if colon \== ':' | verify(hours || mins, '0123456789') \= 0 then
      raise syntax 88.918 array('date', text)
    minutes = hours * 60 + mins
    if mins > 59 | minutes > 900 then raise syntax 88.918 array('date', text)
    if sign == '-' then minutes = -minutes
  end
  signal on syntax name unreadable
  parse var body day 'T' clock
  if pos('.', clock) > 0 then micro = time('F', clock, 'L')
  else micro = time('F', clock, 'N')
  count = date('B', day, 'I') * 86400000000 + micro
  signal off syntax
  return self~new(count, minutes)
unreadable:
  raise syntax 88.918 array('date', text)

-- r[class.datetime.new], r[class.datetime.new.checks],
-- r[class.datetime.new.init-again]. The count of the arguments picks the form;
-- each argument gets the messages of r[class.datetime.sends.new].
::method init
  expose stamp zone
  numeric digits 18
  count = arg()
  if count > 8 then raise syntax 93.902 array(8)
  select
    when count <= 2 then required = min(count, 1)
    when count <= 4 then required = 3
    otherwise required = 5
  end
  do i = 1 to required
    if \arg(i, 'E') then raise syntax 93.903 array(i)
  end
  select
    when count = 0 then do
      parse value time('F') time('O') with now host
      if host // 60000000 \= 0 | abs(host) > 54000000000 then
        raise syntax 88.907 array('offset', -900, 900, host / 60000000)
      stamp = now
      zone = host
      return
    end
    when count <= 2 then do
      n = whole('' || arg(1))
      if n == '' then raise syntax 40.19 array('DATE', arg(1), 'F')
      if n < 0 | n > 315537897599999999 then
        raise syntax 40.19 array('DATE', arg(1), 'F')
      micro = n
      place = 2
    end
    otherwise do
      year = field(arg(1), 'year', 1, 9999, 4)
      month = field(arg(2), 'month', 1, 12, 2)
      day = field(arg(3), 'day', 1, 31, 2)
      days = date('B', right(year, 4, 0) || right(month, 2, 0) || right(day, 2, 0), 'S')
      hours = 0
      minutes = 0
      seconds = 0
      micro = 0
      place = 4
      if count >= 5 then do
        hours = field(arg(4), 'hours', 0, 23, 2)
        minutes = field(arg(5), 'minutes', 0, 59, 2)
        if arg(6, 'E') then seconds = field(arg(6), 'seconds', 0, 59, 2)
        if arg(7, 'E') then micro = field(arg(7), 'microseconds', 0, 999999, 6)
        place = 8
      end
      micro = days * 86400000000 + ((hours * 60 + minutes) * 60 + seconds) * 1000000 + micro
    end
  end
  if arg(place, 'E') then offset = offsetOf(arg(place))
  else do
    parse value time('O') with host
    if host // 60000000 \= 0 | abs(host) > 54000000000 then
      raise syntax 88.907 array('offset', -900, 900, host / 60000000)
    offset = host
  end
  stamp = micro
  zone = offset
  return
-- One field: its value read once, then the argument compared with the limits
-- and sent `right` with the width of its field, as the sends rule says.
field: procedure
  use arg v, name, low, high, width
  n = whole('' || v)
  if n == '' then raise syntax 88.907 array(name, low, high, v)
  if \(v >= low) then raise syntax 88.907 array(name, low, high, v)
  if \(v <= high) then raise syntax 88.907 array(name, low, high, v)
  padded = v~right(width, 0)
  return n
-- r[class.datetime.offset]: a TimeSpan of whole minutes, or a number of
-- minutes, from -900 to 900; answers microseconds.
offsetOf: procedure
  use arg v
  if v~isA(.TimeSpan) then do
    span = v~totalMicroseconds
    if span // 60000000 \= 0 | abs(span) > 54000000000 then
      raise syntax 88.907 array('offset', -900, 900, span / 60000000)
    return span % 1
  end
  n = whole('' || v)
  if n == '' then raise syntax 88.907 array('offset', -900, 900, v)
  if \(v >= -900) then raise syntax 88.907 array('offset', -900, 900, v)
  if \(v <= 900) then raise syntax 88.907 array('offset', -900, 900, v)
  span = v * 60000000
  return span % 1
whole: procedure
  parse arg s
  if \datatype(s, 'N') then return ''
  numeric digits 18
  v = s + 0
  if abs(v) >= 1e18 then return ''
  if v // 1 \= 0 then return ''
  return v % 1

-- Gives an instance that new made the moment the sending method computed, so
-- that it holds it also when the init of a subclass does not forward to this
-- one (r[class.datetime.deviation.131]). The sender is a method of this class.
::method setTimestamp private
  expose stamp zone
  use strict arg count, offset
  numeric digits 18
  if count < 0 | count > 315537897599999999 then
    raise syntax 40.19 array('DATE', count, 'F')
  stamp = count
  zone = offset

-- r[class.datetime.fields]
::method year
  expose stamp zone
  use strict arg
  if \var('STAMP') then parse value 0 0 with stamp zone
  return left(date('S', stamp, 'F'), 4)

::method month
  expose stamp zone
  use strict arg
  if \var('STAMP') then parse value 0 0 with stamp zone
  return substr(date('S', stamp, 'F'), 5, 2) + 0

::method day
  expose stamp zone
  use strict arg
  if \var('STAMP') then parse value 0 0 with stamp zone
  return right(date('S', stamp, 'F'), 2) + 0

::method hours
  expose stamp zone
  use strict arg
  numeric digits 18
  if \var('STAMP') then parse value 0 0 with stamp zone
  return stamp // 86400000000 % 3600000000

::method minutes
  expose stamp zone
  use strict arg
  numeric digits 18
  if \var('STAMP') then parse value 0 0 with stamp zone
  return stamp // 3600000000 % 60000000

::method seconds
  expose stamp zone
  use strict arg
  numeric digits 18
  if \var('STAMP') then parse value 0 0 with stamp zone
  return stamp // 60000000 % 1000000

::method microseconds
  expose stamp zone
  use strict arg
  numeric digits 18
  if \var('STAMP') then parse value 0 0 with stamp zone
  return stamp // 1000000

::method dayMinutes
  expose stamp zone
  use strict arg
  numeric digits 18
  if \var('STAMP') then parse value 0 0 with stamp zone
  return stamp // 86400000000 % 60000000

::method daySeconds
  expose stamp zone
  use strict arg
  numeric digits 18
  if \var('STAMP') then parse value 0 0 with stamp zone
  return stamp // 86400000000 % 1000000

-- r[class.datetime.sends.string]: sends daySeconds and microseconds.
::method dayMicroseconds
  use strict arg
  numeric digits 18
  return self~daySeconds * 1000000 + self~microseconds

-- r[class.datetime.calendar]
::method weekDay
  expose stamp zone
  use strict arg
  numeric digits 18
  if \var('STAMP') then parse value 0 0 with stamp zone
  return stamp % 86400000000 // 7 + 1

::method dayName
  expose stamp zone
  use strict arg
  if \var('STAMP') then parse value 0 0 with stamp zone
  return date('W', stamp, 'F')

::method monthName
  expose stamp zone
  use strict arg
  if \var('STAMP') then parse value 0 0 with stamp zone
  return date('M', stamp, 'F')

::method yearDay
  expose stamp zone
  use strict arg
  if \var('STAMP') then parse value 0 0 with stamp zone
  return date('D', stamp, 'F')

::method baseDate
  expose stamp zone
  use strict arg
  numeric digits 18
  if \var('STAMP') then parse value 0 0 with stamp zone
  return stamp % 86400000000

-- r[class.datetime.sends.string]: isLeapYear sends year, daysInMonth month
-- and isLeapYear, daysInYear isLeapYear.
::method isLeapYear
  use strict arg
  year = self~year
  return (year // 4 = 0 & year // 100 \= 0) | year // 400 = 0

::method daysInMonth
  use strict arg
  month = self~month
  leap = self~isLeapYear
  if month = 2 then return 28 + leap
  return word('31 28 31 30 31 30 31 31 30 31 30 31', month)

::method daysInYear
  use strict arg
  return 365 + self~isLeapYear

-- r[class.datetime.isoweek]. The sends are those of
-- r[class.datetime.sends.string]; weekNumber finds the length of the years
-- around its own without sending anything more.
::method weekNumber
  expose stamp zone
  use strict arg
  numeric digits 18
  if \var('STAMP') then parse value 0 0 with stamp zone
  yearDay = self~yearDay
  weekDay = self~weekDay
  week = (yearDay - weekDay + 10) % 7
  year = left(date('S', stamp, 'F'), 4)
  if week < 1 then return weeks(year - 1)
  if week > weeks(year) then return 1
  return week
-- The ISO weeks of a year: 53 when it begins on a Thursday, or on a
-- Wednesday in a leap year.
weeks: procedure
  parse arg year
  first = date('B', right(year, 4, 0) || '0101', 'S') // 7 + 1
  leap = (year // 4 = 0 & year // 100 \= 0) | year // 400 = 0
  if first = 4 | (first = 3 & leap) then return 53
  return 52

::method weekNumberYear
  use strict arg
  year = self~year
  week = self~weekNumber
  month = self~month
  if month = 1 & week >= 52 then return year - 1
  if month = 12 & week = 1 then return year + 1
  return year

::method weekNumberDate
  use strict arg
  year = self~weekNumberYear
  week = self~weekNumber
  return right(year, 4, 0) || '-W' || right(week, 2, 0) || '-' || self~weekDay

::method weeksInYear
  use strict arg
  year = self~year
  first = date('B', right(year, 4, 0) || '0101', 'S') // 7 + 1
  leap = (year // 4 = 0 & year // 100 \= 0) | year // 400 = 0
  if first = 4 | (first = 3 & leap) then return 53
  return 52

::method ordinalDate
  use strict arg
  return self~year || '-' || right(self~yearDay, 3, 0)

-- r[class.datetime.dateformats]: the separator is that of DATE's osep.
::method europeanDate
  expose stamp zone
  use strict arg separator = ''
  if \var('STAMP') then parse value 0 0 with stamp zone
  if arg(1, 'E') then return date('E', stamp, 'F', separator)
  return date('E', stamp, 'F')

::method normalDate
  expose stamp zone
  use strict arg separator = ''
  if \var('STAMP') then parse value 0 0 with stamp zone
  if arg(1, 'E') then return date('N', stamp, 'F', separator)
  return date('N', stamp, 'F')

::method orderedDate
  expose stamp zone
  use strict arg separator = ''
  if \var('STAMP') then parse value 0 0 with stamp zone
  if arg(1, 'E') then return date('O', stamp, 'F', separator)
  return date('O', stamp, 'F')

::method standardDate
  expose stamp zone
  use strict arg separator = ''
  if \var('STAMP') then parse value 0 0 with stamp zone
  if arg(1, 'E') then return date('S', stamp, 'F', separator)
  return date('S', stamp, 'F')

::method usaDate
  expose stamp zone
  use strict arg separator = ''
  if \var('STAMP') then parse value 0 0 with stamp zone
  if arg(1, 'E') then return date('U', stamp, 'F', separator)
  return date('U', stamp, 'F')

::method languageDate
  expose stamp zone
  use strict arg
  if \var('STAMP') then parse value 0 0 with stamp zone
  return date('L', stamp, 'F')

-- r[class.datetime.timeformats]
::method civilTime
  expose stamp zone
  use strict arg
  if \var('STAMP') then parse value 0 0 with stamp zone
  return time('C', stamp, 'F')

::method normalTime
  expose stamp zone
  use strict arg
  if \var('STAMP') then parse value 0 0 with stamp zone
  return time('N', stamp, 'F')

::method longTime
  expose stamp zone
  use strict arg
  if \var('STAMP') then parse value 0 0 with stamp zone
  return time('L', stamp, 'F')

-- r[class.datetime.counts]
::method fullDate
  expose stamp zone
  use strict arg
  if \var('STAMP') then parse value 0 0 with stamp zone
  return stamp

::method utcDate
  expose stamp zone
  use strict arg
  numeric digits 18
  if \var('STAMP') then parse value 0 0 with stamp zone
  return stamp - zone

::method ticks
  expose stamp zone
  use strict arg
  numeric digits 18
  if \var('STAMP') then parse value 0 0 with stamp zone
  return stamp % 1000000 - 62135596800

-- r[class.datetime.string], r[class.datetime.sends.string]
::method isoDate
  expose stamp zone
  use strict arg
  if \var('STAMP') then parse value 0 0 with stamp zone
  return date('I', stamp, 'F') || 'T' || time('L', stamp, 'F')

::method string
  use strict arg
  return self~isoDate

::method makeString
  use strict arg
  return self~string

::method utcIsoDate
  expose stamp zone
  use strict arg
  numeric digits 18
  if \var('STAMP') then parse value 0 0 with stamp zone
  text = date('I', stamp, 'F') || 'T' || time('L', stamp, 'F')
  if zone = 0 then return text || 'Z'
  minutes = abs(zone) % 60000000
  if zone < 0 then sign = '-'
  else sign = '+'
  return text || sign || right(minutes % 60, 2, 0) || ':' || right(minutes // 60, 2, 0)

-- r[class.datetime.offset]
::method offset
  expose stamp zone
  use strict arg
  if \var('STAMP') then parse value 0 0 with stamp zone
  return .TimeSpan~new(zone)

-- r[class.datetime.add], r[class.datetime.sends.add]: the amount is read
-- once and sent `*`; the sum is rounded to 18 digits and a part of a
-- microsecond is dropped toward the earlier moment.
::method addWeeks
  expose stamp zone
  use strict arg amount
  numeric digits 18
  if \var('STAMP') then parse value 0 0 with stamp zone
  if \datatype(amount, 'N') then raise syntax 88.902 array('weeks', amount)
  moved = stamp + amount * 604800000000
  whole = moved
  if abs(moved) < 1e18 then do
    whole = moved % 1
    if whole > moved then whole = whole - 1
  end
  answer = self~class~new(whole, self~offset)
  answer~setTimestamp(whole, zone)
  return answer

::method addDays
  expose stamp zone
  use strict arg amount
  numeric digits 18
  if \var('STAMP') then parse value 0 0 with stamp zone
  if \datatype(amount, 'N') then raise syntax 88.902 array('days', amount)
  moved = stamp + amount * 86400000000
  whole = moved
  if abs(moved) < 1e18 then do
    whole = moved % 1
    if whole > moved then whole = whole - 1
  end
  answer = self~class~new(whole, self~offset)
  answer~setTimestamp(whole, zone)
  return answer

::method addHours
  expose stamp zone
  use strict arg amount
  numeric digits 18
  if \var('STAMP') then parse value 0 0 with stamp zone
  if \datatype(amount, 'N') then raise syntax 88.902 array('hours', amount)
  moved = stamp + amount * 3600000000
  whole = moved
  if abs(moved) < 1e18 then do
    whole = moved % 1
    if whole > moved then whole = whole - 1
  end
  answer = self~class~new(whole, self~offset)
  answer~setTimestamp(whole, zone)
  return answer

::method addMinutes
  expose stamp zone
  use strict arg amount
  numeric digits 18
  if \var('STAMP') then parse value 0 0 with stamp zone
  if \datatype(amount, 'N') then raise syntax 88.902 array('minutes', amount)
  moved = stamp + amount * 60000000
  whole = moved
  if abs(moved) < 1e18 then do
    whole = moved % 1
    if whole > moved then whole = whole - 1
  end
  answer = self~class~new(whole, self~offset)
  answer~setTimestamp(whole, zone)
  return answer

::method addSeconds
  expose stamp zone
  use strict arg amount
  numeric digits 18
  if \var('STAMP') then parse value 0 0 with stamp zone
  if \datatype(amount, 'N') then raise syntax 88.902 array('seconds', amount)
  moved = stamp + amount * 1000000
  whole = moved
  if abs(moved) < 1e18 then do
    whole = moved % 1
    if whole > moved then whole = whole - 1
  end
  answer = self~class~new(whole, self~offset)
  answer~setTimestamp(whole, zone)
  return answer

-- The amount is read twice: once for its form, once by the sum.
::method addMicroseconds
  expose stamp zone
  use strict arg amount
  numeric digits 18
  if \var('STAMP') then parse value 0 0 with stamp zone
  if \datatype(amount, 'N') then raise syntax 88.902 array('microseconds', amount)
  moved = stamp + amount
  whole = moved
  if abs(moved) < 1e18 then do
    whole = moved % 1
    if whole > moved then whole = whole - 1
  end
  answer = self~class~new(whole, self~offset)
  answer~setTimestamp(whole, zone)
  return answer

-- r[class.datetime.add], r[class.datetime.sends.zone]: the years are read
-- twice, then the receiver is sent date, timeOfDay, year, month, day and
-- offset, new makes the day and `+` adds the time of day back.
::method addYears
  expose stamp zone
  use strict arg years
  numeric digits 18
  if \var('STAMP') then parse value 0 0 with stamp zone
  if \datatype(years, 'N') then raise syntax 88.903 array('years', years)
  n = 0 + years
  if abs(n) >= 1e9 then raise syntax 88.903 array('years', years)
  if n // 1 \= 0 then raise syntax 88.903 array('years', years)
  n = n % 1
  start = self~date
  clock = self~timeOfDay
  year = self~year + n
  month = self~month
  day = self~day
  offset = self~offset
  if month = 2 & day = 29 then
    if \((year // 4 = 0 & year // 100 \= 0) | year // 400 = 0) then day = 28
  answer = self~class~new(year, month, day, offset)
  answer~setTimestamp(date('B', right(year, 4, 0) || right(month, 2, 0) || right(day, 2, 0), 'S') * 86400000000, zone)
  return answer + clock

-- r[class.datetime.plus], r[class.datetime.sends.add]
::method '+'
  expose stamp zone
  numeric digits 18
  if arg() > 1 then raise syntax 93.902 array(1)
  if \var('STAMP') then parse value 0 0 with stamp zone
  span = 0
  if arg(1, 'E') then do
    if \arg(1)~isA(.TimeSpan) then raise syntax 93.948 array(1, 'TimeSpan')
    span = arg(1)~totalMicroseconds
  end
  answer = self~class~new(stamp + span, self~offset)
  answer~setTimestamp(stamp + span, zone)
  return answer

-- r[class.datetime.minus], r[class.datetime.sends.add]
::method '-'
  expose stamp zone
  use strict arg other
  numeric digits 18
  if \var('STAMP') then parse value 0 0 with stamp zone
  if other~isA(.TimeSpan) then do
    span = other~totalMicroseconds
    answer = self~class~new(stamp - span, self~offset)
    answer~setTimestamp(stamp - span, zone)
    return answer
  end
  if \other~isA(.DateTime) then
    raise syntax 88.914 array('subtractor', 'DateTime or TimeSpan')
  return .TimeSpan~new(stamp - zone - other~utcDate)

-- r[class.datetime.compare], r[class.datetime.sends.compare],
-- r[class.datetime.deviation.126]
::method compareTo
  expose stamp zone
  use strict arg other
  numeric digits 18
  if \var('STAMP') then parse value 0 0 with stamp zone
  if \other~isA(.DateTime) then raise syntax 88.914 array(1, 'DateTime')
  return sign(stamp - zone - other~utcDate)

-- r[class.datetime.hashcode], r[class.datetime.deviation.128]: the hash of
-- the instant's count in UTC.
::method hashCode
  expose stamp zone
  use strict arg
  numeric digits 18
  if \var('STAMP') then parse value 0 0 with stamp zone
  return (stamp - zone)~hashCode

-- r[class.datetime.zone], r[class.datetime.sends.zone]
::method toTimezone
  expose stamp zone
  use strict arg offset = 0
  numeric digits 18
  if \var('STAMP') then parse value 0 0 with stamp zone
  if offset~isA(.TimeSpan) then do
    span = offset~totalMicroseconds
    if span // 60000000 \= 0 | abs(span) > 54000000000 then
      raise syntax 88.907 array('offset', -900, 900, span / 60000000)
    given = span % 60000000
    span = span % 1
  end
  else do
    span = offset * 60000000
    n = whole('' || offset)
    if n == '' then raise syntax 88.907 array('offset', -900, 900, offset)
    if \(offset >= -900) then raise syntax 88.907 array('offset', -900, 900, offset)
    if \(offset <= 900) then raise syntax 88.907 array('offset', -900, 900, offset)
    given = offset
    span = span % 1
  end
  answer = self~class~new(stamp - zone + span, given)
  answer~setTimestamp(stamp - zone + span, span)
  return answer
whole: procedure
  parse arg s
  if \datatype(s, 'N') then return ''
  numeric digits 18
  v = s + 0
  if abs(v) >= 1e18 then return ''
  if v // 1 \= 0 then return ''
  return v % 1

::method toUtcTime
  use strict arg
  return self~toTimezone(0)

-- The default offset in minutes; toTimezone refuses one that is not whole.
::method toLocalTime
  use strict arg
  numeric digits 18
  return self~toTimezone(time('O') / 60000000)

-- r[class.datetime.parts], r[class.datetime.sends.zone]
::method date
  expose stamp zone
  use strict arg
  numeric digits 18
  if \var('STAMP') then parse value 0 0 with stamp zone
  day = self~standardDate
  offset = self~offset
  count = date('B', day, 'S') * 86400000000
  answer = self~class~new(count, offset)
  answer~setTimestamp(count, zone)
  return answer

::method timeOfDay
  use strict arg
  return .TimeSpan~new(time('F', self~longTime, 'L'))

-- r[class.datetime.deviation.129]: the current instant less the receiver's,
-- both in UTC.
::method elapsed
  expose stamp zone
  use strict arg
  numeric digits 18
  if \var('STAMP') then parse value 0 0 with stamp zone
  parse value time('F') time('O') with now host
  return .TimeSpan~new(now - host - (stamp - zone))
