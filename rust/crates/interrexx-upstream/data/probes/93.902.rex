quiet_ = .output~destination(.stream~new('/dev/null'))
e = .eventsemaphore~new
say e~isPosted
e~post
say e~isPosted
e~reset
say e~isPosted
signal on syntax name bad
say e~isPosted(1)
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
