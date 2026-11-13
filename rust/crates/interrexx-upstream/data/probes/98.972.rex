quiet_ = .output~destination(.stream~new('/dev/null'))
call try 'lostdigits'
exit
try:
  which = arg(1)
  signal on syntax name e
  select
    when which = 'error' then address system 'exit 3'
    when which = 'novalue' then say undefinedvar
    when which = 'lostdigits' then do; numeric digits 3; say 12345 + 0; end
    otherwise nop
  end
  say which 'went on'
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
::options error condition all syntax novalue condition
