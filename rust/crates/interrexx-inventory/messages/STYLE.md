# Message text style

The files in this directory hold the text of each error message, one file per major number.
The code and subcode are the contract and the text is not (`r[conv.error.text]`), so a text
serves a reader who looks at a report. The rules below keep the texts in one voice.

## Sources

A text is written fresh unless one of these three holds, and then it may match ooRexx's
wording:

* It is the text of the ANSI standard (X3.274-1996, the `#ErrorText.` table) for the code.
  Where ANSI and ooRexx both have a text, ANSI's is used. An ANSI text may name the keywords
  ooRexx adds (such as `LOOP` next to `DO`), give a location in the form below, and drop a
  part whose insert the row does not render.
* It is one of the patterns under Form with only names filled in: `X must be Y; found "&k".`,
  `X must not Y.`, `X expected after Y keyword.`, `Missing X following Y keyword.` or `&1.`.
* It is the plain statement anyone given only the meaning of the code would write nearly word
  for word: a subject, then `must`, `must not`, `is` or `expected`, then an object. It has no
  adverb, no parenthetical remark, no second clause and no turn of phrase peculiar to ooRexx.
  `INTERPRET data must not contain directive instructions.` is such a text.

Any other text is written fresh, even where ooRexx's sentence reads well, and so is a text
that would break a rule below. When in doubt, a text is rewritten.

## Form

* A text is one sentence that starts with a capital letter and ends with a period.
  Two clauses are joined with a semicolon.
* The text states the fault or the requirement that was not met, in the present tense.
  It gives no advice, does not address the reader and does not start with `Error`.
* The text of a major number (subcode 0) is a short phrase naming the kind of fault.
* A requirement reads `X must be Y; found "&k".`, with the value that broke it last.
  A prohibition reads `X must not Y.`, and a repeated keyword reads
  `X keyword must not appear more than once.`.
  A missing part reads `X expected after Y keyword.` or `Missing X following Y keyword.`.
  A keyword an instruction or directive does not accept reads
  `Invalid sub-keyword on X; found "&k".`.
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
