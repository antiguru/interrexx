/* A non-strict comparison against the empty string is the string rule
   whatever the other operand holds, because the empty string is never a
   number: leading and trailing blanks and tabs are stripped, and the first
   byte left is compared against a blank. Each value is compared on both
   sides of every non-strict operator, held inline (short) and on the heap
   (long), with a control byte below the blank and a number-like operand. */
long = 'a fairly long heap string'
blanks = copies(' ', 20)
tabs = '09'x || '  ' || '09'x
ctl = '01'x
ctlh = copies(' ', 10) || '01'x || 'x'
vals = .array~of('', ' ', 'a', ' a', 'a ', long, blanks, tabs, ctl, ctlh, '0', ' 0 ', '-', '~', 'E')
ops = .array~of('=', '\=', '<>', '><', '<', '>', '<=', '>=', '\<', '\>')
do v over vals
  out = ''
  do op over ops
    interpret 'r1 = v' op "''"
    interpret 'r2 = ""' op 'v'
    out = out || r1 || r2
  end
  say c2x(left(v, 3)) length(v) out
end
/* The same operators written out, outside INTERPRET. */
do v over .array~of('', blanks, long, ctlh, ' 0 ')
  say (v = '') (v \= '') ('' = v) ('' \= v) (v < '') (v > '') ('' < v) ('' > v) (v <= '') ('' >= v)
end
/* The loop the comparison is usually written for. */
x = 'one two  three'
do while x \= ''
  parse var x w x
  say '['w']'
end
