quiet_ = .output~destination(.stream~new('/dev/null'))
f = 'spec-address-using.tmp'
call sysfiledelete f
call try 'nil', ".nil", ''
call sysfiledelete f
exit
try: procedure expose f
  signal on syntax name e
  interpret "address system 'exit 0' with output" arg(3) "using (" arg(2) ")"
  say arg(1) 'ok'
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
::class own
::method append
  return 1
::class substem subclass stem
