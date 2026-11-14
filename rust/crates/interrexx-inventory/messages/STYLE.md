# Message text style

The files in this directory hold the text of each error message, one file per major number.
The code and subcode are the contract and the text is not (`r[conv.error.text]`), so a text
serves a reader who looks at a report. The rules below keep the texts in one voice.

## Sources

* A text that states a fact mechanically may keep ooRexx's wording, and a text of the ANSI
  standard (X3.274-1996, the `#ErrorText.` table) may be used as it stands.
* A text whose wording is distinctive prose, such as an explanation, advice or an idiom, is
  written fresh.
* Where a text would break a rule below, it is rewritten to follow the rule.

## Form

* A text is one sentence that starts with a capital letter and ends with a period.
  Two clauses are joined with a semicolon.
* The text states the fault or the requirement that was not met, in the present tense.
  It gives no advice, does not address the reader and does not start with `Error`.
* The text of a major number (subcode 0) is a short phrase naming the kind of fault.
* A requirement reads `X must be Y; found "&k".`, with the value that broke it last.
  A missing part reads `X expected after Y keyword.` or `Missing X following Y keyword.`.
* The texts of the `.900` subcodes are `&1.`: the program supplies the whole message.
* No word of the notice of interactive tracing ends in a colon (`r[env.debugging.notice]`).

## Words

* A construct or value that is malformed is "invalid", which is ANSI's word.
  "Incorrect" is not used for it, except where an ANSI text of a major number has it.
* Instruction keywords, subkeywords, directive names, built-in function names and method names
  are written in upper case, as in `DO`, `::CLASS`, `NUMERIC DIGITS` and `COMPARETO`.
* A line number reads `on line &k` and a character position reads `in position &k`.

## Inserts

* An insert is used only at a position the row's `rendered` column of `data/errors.tsv` lists.
* A value taken from the program (a token, a string, a name) is in double quotes: `"&k"`.
* A number (a line, a position, a count or a limit) is not quoted.
* A character's hexadecimal encoding follows it as `('&k'X)`.
