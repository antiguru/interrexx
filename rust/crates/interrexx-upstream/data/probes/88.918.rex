quiet_ = .output~destination(.stream~new('/dev/null'))
t = .datetime~today
say t~normalTime t~microseconds (t~offset == .datetime~new~offset)
say .datetime~today(90)~offset .datetime~today(.timespan~fromHours(-3))~offset
signal on syntax name bad
say .datetime~today(901)
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
signal on syntax name bad2
say .datetime~today(901, 1)
bad2: say condition('o')~code
