quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name s
signal on error name t
address nosuchenv 'echo x'
say 'went on'
exit
t: say 'error trap' condition('c')
exit
s: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
::options failure syntax
