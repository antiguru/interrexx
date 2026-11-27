/* A STRING answer with no string value counts as none, and the default string
   value stands in for it in each consumer: SIGNAL VALUE, a stem tail,
   arithmetic, NUMERIC DIGITS and FUZZ, a DO count and FOR, a host command,
   INTERPRET, an equality, NUMERIC FORM VALUE and a semaphore timeout. */
o = .sn~new
signal on syntax name s1
signal value o
s1: say 'signal value' condition('O')~code
x.o = 1
say 'tail' x.o x.['a SN']
signal on syntax name s2
say 1 + o
s2: say 'arithmetic' condition('O')~code
signal on syntax name s3
say 2 ** o
s3: say 'power' condition('O')~code
signal on syntax name s4
numeric digits o
s4: say 'digits' condition('O')~code
signal on syntax name s5
numeric fuzz o
s5: say 'fuzz' condition('O')~code
signal on syntax name s6
do o; end
s6: say 'do count' condition('O')~code
signal on syntax name s7
do i = 1 for o; end
s7: say 'do for' condition('O')~code
signal on syntax name s8
numeric form value o
s8: say 'form' condition('O')~code
signal on syntax name s9
say .eventsemaphore~new~wait(o)
s9: say 'timeout' condition('O')~code
say 'equal' ("abc" = o) ("a SN" = o) ("a SN" == o) ("abc" \= o) ("abc" \== o) ("" = o) ("" == o) ("" \= o) (1 = o) (1 <> o)
address system o
say 'command rc' rc
interpret o
say 'interpret rc' rc
exit
::class sn
::method string
  return .directory~new
