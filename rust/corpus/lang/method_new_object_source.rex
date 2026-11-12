/* Method~new with a source that is neither a string nor an array: 93.961
 * is the refusal. */
signal on syntax
m = .method~new('m', .object~new)
say 'no'
exit
syntax: say rc condition('O')~code
