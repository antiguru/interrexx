p = .context~package
say 'psl' '['p~sourceLine(1)']'
say 'rs' .resources['RZ']~items .resources['RZ'][1]
say 'pa' p~annotations~items
say 'ra' p~findRoutine('RR')~annotations~items p~findRoutine('RR')~source~items
say 'ca' .kqd~annotations~items .kqd~method('M')~source~items
exit
::resource rz
line one -- not a comment
::END
::annotate package pk "v"
::routine rr
::annotate routine rr ka "kb"
::class kqd
::annotate class kqd ca "cb"
::method m
  return 1
