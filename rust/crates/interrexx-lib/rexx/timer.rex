/* Timer classes. Spec rules: class.alarm.*, class.ticker.*,
   class.alarmnotification.*, class.messagenotification.*. */

-- r[class.alarmnotification.creation]
::class "AlarmNotification" public mixinclass Object

-- r[class.alarmnotification.cancel]: does nothing, whatever it is sent.
::method cancel
  return

-- r[class.alarmnotification.triggered]
::method triggered abstract

-- r[class.messagenotification.creation]
::class "MessageNotification" public mixinclass Object

-- r[class.messagenotification.messagecomplete]
::method messageComplete abstract

-- r[class.alarm.creation]
::class "Alarm" public

-- r[class.alarm.attachment], r[class.alarm.canceled],
-- r[class.alarm.scheduledtime], r[class.alarm.triggered]: each reader refuses
-- an alarm whose init never ran (r[class.alarm.deviation.135]).
::attribute attachment get unguarded
  expose armed attachment
  if armed \== 1 then raise syntax 98.901 array(self)
  if arg() > 0 then raise syntax 93.902 array(0)
  return attachment

::attribute canceled get unguarded
  expose armed canceled
  if armed \== 1 then raise syntax 98.901 array(self)
  if arg() > 0 then raise syntax 93.902 array(0)
  return canceled

::attribute scheduledTime get unguarded
  expose armed scheduledTime
  if armed \== 1 then raise syntax 98.901 array(self)
  if arg() > 0 then raise syntax 93.902 array(0)
  return scheduledTime

::attribute triggered get
  expose armed triggered
  if armed \== 1 then raise syntax 98.901 array(self)
  if arg() > 0 then raise syntax 93.902 array(0)
  return triggered

-- r[class.alarm.canceled]: a method of its own, not a send of canceled.
::method cancelled unguarded
  expose armed canceled
  if armed \== 1 then raise syntax 98.901 array(self)
  if arg() > 0 then raise syntax 93.902 array(0)
  return canceled

-- r[class.alarm.init], r[class.alarm.init.seconds], r[class.alarm.init.clock],
-- r[class.alarm.init.moment], r[class.alarm.init.sends]
::method init guarded
  expose armed attachment target canceled triggered scheduledTime ,
    eventSemHandle timerStarted
  -- r[conc.halt.deviation.162]: a second init is refused and changes nothing.
  if armed == 1 then
    raise syntax 98.900 array('init was sent to an alarm that is already set up')
  use strict arg atime, tgt, att = .nil
  numeric digits 18
  if \tgt~isA(.AlarmNotification) then
    raise syntax 88.914 array(2, 'AlarmNotification')
  now = .DateTime~new
  select
    when atime~isA(.TimeSpan) then do
      us = atime~totalMicroseconds
      if atime~sign < 0 then raise syntax 93.951 array(atime)
      days = atime~days
      us = atime~totalMicroseconds
      sched = now~addMicroseconds(us)
    end
    when atime~isA(.DateTime) then do
      span = atime - now
      if span~sign <= 0 then raise syntax 93.951 array(atime)
      sched = atime
      us = span~totalMicroseconds
    end
    when atime~isA(.String) then do
      text = atime
      if pos(':', text) > 0 then do
        -- hh:mm:ss, optionally followed by one blank and d Mmm yyyy
        blank = pos(' ', text)
        if blank = 0 then do
          clock = text
          rest = ''
          dated = 0
        end
        else do
          clock = left(text, blank - 1)
          rest = substr(text, blank + 1)
          dated = 1
        end
        digits = '0123456789'
        if length(clock) \= 8 | substr(clock, 3, 1) \== ':' | ,
            substr(clock, 6, 1) \== ':' then raise syntax 93.951 array(text)
        hh = left(clock, 2)
        mm = substr(clock, 4, 2)
        ss = substr(clock, 7, 2)
        if verify(hh || mm || ss, digits) \= 0 then raise syntax 93.951 array(text)
        if hh >= 24 | mm >= 60 | ss >= 60 then raise syntax 93.951 array(text)
        hh = hh + 0
        mm = mm + 0
        ss = ss + 0
        if dated then do
          parse var rest dd mon yyyy more
          if rest \== dd mon yyyy | more \== '' | length(dd) > 2 | ,
              length(yyyy) \= 4 | verify(dd, digits) \= 0 | ,
              verify(yyyy, digits) \= 0 then raise syntax 93.951 array(text)
          month = wordpos(mon, 'Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec')
          if month = 0 | length(mon) \= 3 then raise syntax 93.951 array(text)
          dd = dd + 0
          yyyy = yyyy + 0
          if yyyy < 1 | dd < 1 then raise syntax 93.951 array(text)
          if .DateTime~new(yyyy, month, 1)~daysInMonth < dd then
            raise syntax 93.951 array(text)
          sched = .DateTime~new(yyyy, month, dd, hh, mm, ss)
          if sched <= now then raise syntax 93.951 array(text)
        end
        else do
          sched = .DateTime~new(now~year, now~month, now~day, hh, mm, ss)
          if sched <= now then sched = sched~addDays(1)
        end
      end
      else do
        if \datatype(text, 'N') then raise syntax 93.951 array(text)
        secs = text + 0
        if abs(secs) >= 10 ** 12 then raise syntax 88.903 array(1, text)
        secs = trunc(secs, 6)
        sched = now~addSeconds(secs)
        if secs < 0 then raise syntax 93.951 array(text)
      end
      us = (sched - now)~totalMicroseconds
    end
    otherwise raise syntax 93.951 array(atime)
  end
  -- The wait is a number of whole days and the milliseconds after them,
  -- the milliseconds rounded up.
  days = us % 86400000000
  millis = (us // 86400000000 + 999) % 1000
  attachment = att
  target = tgt
  scheduledTime = sched
  canceled = .false
  triggered = .false
  timerStarted = 0
  armed = 1
  self~start('!RUN', days, millis)
  guard on when timerStarted == 1

-- Waits out the time on an activity of its own, then tells the target.
::method '!run' private unguarded
  expose canceled triggered target
  use arg days, millis
  self~!startTimer(days, millis)
  guard on
  if canceled == 1 then return
  triggered = .true
  guard off
  target~triggered(self)

::method !startTimer private external "LIBRARY REXX alarm_startTimer"
::method !stopTimer private external "LIBRARY REXX alarm_stopTimer"

-- r[class.alarm.cancel]
::method cancel unguarded
  expose armed canceled triggered eventSemHandle target
  if armed \== 1 then raise syntax 98.901 array(self)
  if arg() > 0 then raise syntax 93.902 array(0)
  guard on
  if canceled == 1 | triggered == 1 then return
  canceled = .true
  guard off
  self~!stopTimer(eventSemHandle)
  target~cancel(self)

-- r[class.ticker.creation]
::class "Ticker" public

-- r[class.ticker.attachment], r[class.ticker.canceled], r[class.ticker.interval]:
-- each reader refuses a ticker whose init never ran
-- (r[class.ticker.deviation.135]).
::attribute attachment get unguarded
  expose armed attachment
  if armed \== 1 then raise syntax 98.901 array(self)
  if arg() > 0 then raise syntax 93.902 array(0)
  return attachment

::attribute canceled get unguarded
  expose armed canceled
  if armed \== 1 then raise syntax 98.901 array(self)
  if arg() > 0 then raise syntax 93.902 array(0)
  return canceled

::attribute interval get unguarded
  expose armed interval
  if armed \== 1 then raise syntax 98.901 array(self)
  if arg() > 0 then raise syntax 93.902 array(0)
  return interval

::method cancelled unguarded
  expose armed canceled
  if armed \== 1 then raise syntax 98.901 array(self)
  if arg() > 0 then raise syntax 93.902 array(0)
  return canceled

-- r[class.ticker.init], r[class.ticker.init.interval],
-- r[class.ticker.init.sends]
::method init guarded
  expose armed attachment target canceled interval eventSemHandle
  -- r[conc.halt.deviation.162]: a second init is refused and changes nothing.
  if armed == 1 then
    raise syntax 98.900 array('init was sent to a ticker that is already set up')
  use strict arg ival, tgt, att = .nil
  numeric digits 18
  if \tgt~isA(.AlarmNotification) then
    raise syntax 88.914 array(2, 'AlarmNotification')
  select
    when ival~isA(.TimeSpan) then do
      if ival~sign < 0 then raise syntax 93.951 array(ival)
      days = ival~days
      us = ival~totalMicroseconds
      span = .TimeSpan~fromMicroseconds(us)
    end
    when ival~isA(.String) then do
      text = ival
      if \datatype(text, 'N') then raise syntax 93.951 array(text)
      secs = text + 0
      if abs(secs) >= 10 ** 12 then raise syntax 88.903 array(1, text)
      secs = trunc(secs, 6)
      span = .TimeSpan~fromSeconds(secs)
      if secs < 0 then raise syntax 93.951 array(text)
      days = span~days
      us = span~totalMicroseconds
    end
    otherwise raise syntax 93.951 array(ival)
  end
  days = us % 86400000000
  millis = (us // 86400000000 + 999) % 1000
  attachment = att
  target = tgt
  interval = span
  canceled = .false
  armed = 1
  self~!createTimer
  self~start('!RUN', days, millis)

-- Sends the target triggered at the end of each interval until a cancel.
::method '!run' private unguarded
  expose canceled target eventSemHandle
  use arg days, millis
  do forever
    self~!waitTimer(eventSemHandle, days, millis)
    if canceled == 1 then leave
    target~triggered(self)
    if canceled == 1 then leave
  end

::method !createTimer private external "LIBRARY REXX ticker_createTimer"
::method !waitTimer private external "LIBRARY REXX ticker_waitTimer"
::method !stopTimer private external "LIBRARY REXX ticker_stopTimer"

-- r[class.ticker.cancel]
::method cancel unguarded
  expose armed canceled eventSemHandle target
  if armed \== 1 then raise syntax 98.901 array(self)
  if arg() > 0 then raise syntax 93.902 array(0)
  guard on
  if canceled == 1 then return
  canceled = .true
  guard off
  self~!stopTimer(eventSemHandle)
  target~cancel(self)
