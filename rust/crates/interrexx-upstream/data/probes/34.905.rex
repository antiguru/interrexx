quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name raised_
/* A SELECT CASE value's == answer that is not 0 or 1 is Error 34.905. */
select case .c~new
  when 'x' then say 'x'
  otherwise say 'otherwise'
end
exit
exit
raised_: c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end
::class c
::method '=='; return 'banana'
