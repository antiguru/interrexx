quiet_ = .output~destination(.stream~new('/dev/null'))
call try 'notready'
exit
try:
  which = arg(1)
  signal on syntax name e
  select
    when which = 'error' then address system 'exit 3'
    when which = 'failure' then address nosuchenv 'echo x'
    when which = 'lostdigits' then do; numeric digits 3; say 12345 + 0; end
    when which = 'nostring' then say 'a' || .nstr~new
    when which = 'notready' then say linein('nosuchfile.zzz')
    when which = 'novalue' then say undefinedvar
  end
  say which 'went on'
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
::class nstr
::method string
  return .nil
::options error syntax failure syntax lostdigits syntax
::options nostring syntax notready syntax novalue syntax
