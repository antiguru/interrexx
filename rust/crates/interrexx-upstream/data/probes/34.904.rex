quiet_ = .output~destination(.stream~new('/dev/null'))
p = .properties~new
do v over .array~of('1', '0', 'true', 'false')
  p~setProperty('k', v)
  call charout , p~getLogical('k') ''
end
say
p~setProperty('k', 'true')
say p~getLogical('k', 'junk') p~getLogical('zz', 1)
say p~getLogical('zz', 'false')
say p~getLogical('zz', .true) p~getLogical('zz', .false)
do v over .array~of('TRUE', 'False', 'yes', '2', '-1', '00', '', ' 1')
  p~setProperty('k', v)
end
p~setEntry('k', .text~new('true'))
call try 'p~getLogical("K")', 'object'
exit
try:
  signal on syntax name bad
  interpret 'r =' arg(1)
  say arg(2) '=>' r
  return
bad: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
::class text
::method init
  expose v
  use arg v
::method makestring
  expose v
  return v
::method request
  expose v
  if arg(1) = 'STRING' then return v
  return .nil
