# Message text style

The files in this directory hold the text of each error message, one file per major number.
The code and subcode are the contract and the text is not (`r[conv.error.text]`), so a text
serves a reader who looks at a report. The rules below keep the texts in one voice.

## Sources

A text is written fresh unless one of these three holds, and then it may match ooRexx's
wording:

* It is the text of the ANSI standard (X3.274-1996, the `#ErrorText.` table) for the code.
  Where ANSI and ooRexx both have a text, ANSI's is used. An ANSI text may name what ooRexx
  adds (such as `LOOP` next to `DO`, or a message next to a function), give a location in the
  form below, and drop a part whose insert the row does not render. It is recast into a
  canonical phrase where only the form changes, not the meaning. Where ooRexx gives the
  code a different meaning than ANSI does, ANSI's text does not apply, and neither does an
  ANSI text that does not read as a sentence once its inserts are placed.
* It is one of the canonical phrases below, or `&1.`, with only names filled in.
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
* Texts that state the same kind of fault use the same phrase, listed under Canonical
  phrases, even where ooRexx or ANSI word them differently.
* The texts of the `.900` subcodes are `&1.`: the program supplies the whole message.
* No word of the notice of interactive tracing ends in a colon (`r[env.debugging.notice]`).

## Canonical phrases

A text that states one of these kinds of fault uses its phrase. `X` and `Y` stand for names,
and `; found "&k"` closes the text when the row renders the value that broke the rule.

* A value or part that breaks a requirement: `X must be Y; found "&k".`
  A choice of keywords reads `X must be followed by one of the keywords A, B, or C; found "&k".`
* A construct used where it may not stand: `X must not be used in Y.`
  Other prohibitions read `X must not Y.`, as in `Data must not follow ...`.
* A construct that is valid in one place only: `X is valid only in Y.`
* A keyword or condition given twice: `X must not appear more than once.`
* A keyword a clause does not accept: `Invalid sub-keyword on X; found "&k".`
* A part absent where the clause ends or another part follows: `Missing X following Y.`,
  or `Missing X in Y.` for an item of a list, a directive or an invocation.
* A token of the wrong kind where a part must stand: `X expected after Y.`, or
  `X expected as Y.` where Y names the part's role rather than its position.
* A construct that lacks a part it needs: `X requires Y.`, as in
  `DO instruction on line &k requires a matching END.`
* An opening or closing delimiter without its partner: `Unmatched D.`, with `in Y` and a
  location where the row has them. `D` is the delimiter in double quotes, or the name of a
  quote character.
* A closing keyword without the instruction it closes: `X has no corresponding Y.`
* A symbol after END that has nothing to match: `END corresponding to X on line &k must not
  have a symbol following it because there is no Y; found "&k".`
* A token where none of its kind may stand: `Unexpected X.`
* A name that does not resolve: `X "&k" not found.`
* An operation that failed on a value: `X cannot be Y.` A failed conversion reads
  `Object "&k" cannot be converted to a Y.`, and code that fails to load reads
  `X cannot be loaded.`, with `from library "&k"` where the row has the library.
* A directive whose name another directive of its kind already took:
  `::X directive must not reuse the name of another Y.`
* A directive or method kind that takes no body but has one: `X must not have a body.`
* A malformed construct with no more to say: `Invalid X.`

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
