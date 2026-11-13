/* A parenthesised PARSE pattern can change the TRACE setting while the
   clause runs; the operand's own >>> line follows the setting the operand
   left, not the one the clause started under. */
a = 'abc xyz def'
parse var a b (trace('R')) c
say '1' b '|' c
trace o
trace r
parse var a b (trace('O')) c
say '2' b '|' c
trace o
parse var a b ' ' (trace('I')) c 'x' d
say '3' b '|' c '|' d
trace o
trace r
parse var a b (trace('I')) c 'x' d
trace o
parse var a b (trace('I')) c 'x' d
trace o
say '4' b '|' c '|' d
