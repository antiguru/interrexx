# Spec pilot: the ten questions

Written by the controller on 2026-11-05, before any pilot page. Each answer was run on the crate
(`rust/target/debug/rexx` at 55f53d1ec) and on the oracle; both agree on every one. An answer is a
returned string in single quotes, or `SYNTAX` and the error code.

| # | Question | Answer | Kind |
|---|---|---|---|
| 1 | What does `substr('abc', 2, 4, '.')` return? | `'bc..'` | plain |
| 2 | What does `substr('abc', 5)` return? | `''` | edge |
| 3 | What does `substr('abc', 0)` do? | `SYNTAX 93.924` | error |
| 4 | What does `substr('abc', 1, 2, 'xy')` do? | `SYNTAX 40.23` | error |
| 5 | What does `subword('  a  b  c  ', 2)` return? | `'b  c'` | edge |
| 6 | What does `word('a' \|\| '09'x \|\| 'b', 2)` return? | `'b'` | edge |
| 7 | What does `wordindex('  ab cd', 2)` return? | `'6'` | plain |
| 8 | What does `wordpos('b c', 'a b  c d')` return? | `'2'` | edge |
| 9 | What does `translate('aa', 'xy', 'aa')` return? | `'xx'` | edge |
| 10 | What does `verify('abc', 'b', 'X')` do? | `SYNTAX 93.915` | error |

The answering agent gets only the Question column.
