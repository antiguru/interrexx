quiet_ = .output~destination(.stream~new('/dev/null'))
say xrange(1, 3) xrange(.mutablebuffer~new('c'), .mutablebuffer~new('e'))
signal on syntax name a
say xrange(.object~new)
a: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
