quiet_ = .output~destination(.stream~new('/dev/null'))
do 2
  do
    say 'group'
    iterate
  end
  say 'skipped'
end
do 2
  select
    when 1 then iterate
  end
  say 'skipped'
end
call try 'do label g; iterate g; end'
exit
try:
  signal on syntax name e
  interpret arg(1)
  say arg(1) '=> ok'
  return
e: do; c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end; exit; end
  return
