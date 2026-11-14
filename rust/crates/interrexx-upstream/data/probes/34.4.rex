quiet_ = .output~destination(.stream~new('/dev/null'))
do until 1
  say 'once'
end
do i = 1 to 5 until t(i)
  say 'body' i
  if i = 2 then iterate
  say 'tail' i
end
say 'i' i
do e over .array~of(1, 2, 3) until e = 2
  say 'u' e
end
say 'e' e
call try 'do until 2; end'
exit
t: say 'test' arg(1); return arg(1) = 3
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
