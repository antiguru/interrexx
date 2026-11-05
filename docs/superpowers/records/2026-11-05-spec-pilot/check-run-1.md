# Ten-question check, run 1

2026-11-05, at `511c9172f`. A fresh agent (Sonnet) was given a copy of `spec/` and the question
column of `questions.md` only. It was told to read nothing else and run nothing. Its answers, with
the rules it cited:

| # | Answer | Rules cited | Expected | Match |
|---|---|---|---|---|
| 1 | `'bc..'` | bif.substr.result, bif.substr.pad.default | `'bc..'` | yes |
| 2 | `''` | bif.substr.past-end, bif.substr.length.default | `''` | yes |
| 3 | `SYNTAX 93.924` | conv.position.check, bif.substr.method | `SYNTAX 93.924` | yes |
| 4 | `SYNTAX 40.23` | conv.pad, bif.substr.method | `SYNTAX 40.23` | yes |
| 5 | `'b  c'` | bif.subword.length.default, bif.subword.result, bif.words.split | `'b  c'` | yes |
| 6 | `'b'` | conv.string.whitespace, bif.words.split, bif.word.result | `'b'` | yes |
| 7 | `6` | bif.wordindex.result, bif.words.split | `'6'` | yes |
| 8 | `2` | bif.wordpos.result, bif.wordpos.whitespace | `'2'` | yes |
| 9 | `'xx'` | bif.translate.first, bif.translate.result | `'xx'` | yes |
| 10 | `SYNTAX 93.915` | bif.verify.option.check | `SYNTAX 93.915` | yes |

10 of 10. No page needed a fix.
