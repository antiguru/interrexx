call on halt name h
s.0 = 5; s.1 = 'old'
say 'a'
address system 'echo first; exec 1>&-; printf err >&2; sleep 5; printf late >&2' with output stem s. error stem e.
say 'b' rc s.0 s.1
exit
h: say 'halted' sigl; return
