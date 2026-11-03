# Extension libraries: behaviour

What replacements for the ooRexx extension libraries `rxmath`, `rxregexp`, `rxunixsys` and `rxsock`
must do, as a Rexx program sees them. It also covers the Rexx packages shipped with two of them:
`rxregexp.cls`, and `socket.cls`, `streamsocket.cls`, `mime.cls` and `smtp.cls`. It is the
clean-room description named in `docs/superpowers/specs/2026-11-02-phase-10-design.md`.

Each statement describes the oracle's observed behaviour (`third_party/oracle`, ooRexx 5.3.0,
x86_64 Linux, glibc) unless marked otherwise. Results come from running the oracle, the ooTest groups
`extensions/rxmath`, `extensions/rxregexp`, `extensions/rxsock` and
`extensions/platform/unix/rxunixsys`, and `oodocs`. The rxmath, rxsock and unix-extension books are
not in the repository's `oodocs` checkout; only the RegularExpression class is documented there, in
`rexxref/en-US/utilityclasses.xml`. Where the oracle is defective, the text says **Defect:** and the
defect is listed under "Oracle defects". **Observed:** and **Documented:** mark disagreements.
Decisions on these are in the "Rulings" section at the end.

## Conventions shared by all four libraries

### Loading

Each library is a *package library*: loading it makes all of its routines, or its native methods,
available at once.

* `::requires 'name' LIBRARY` loads it. `.context~package~loadLibrary('name')` also loads it,
  returning 1 on success and 0 when no such library exists.
* The library name is case-sensitive: `::requires 'RXMATH' LIBRARY` fails with
  `Error 98.903: Unable to load library "RXMATH".` and `RxFuncAdd('RxCalcSqrt', 'RXMATH')` returns 1.
  After that failed `RxFuncAdd`, `RxFuncQuery('RxCalcSqrt')` returns 0, yet calling `RxCalcSqrt`
  raises 43.1.
* `RxFuncAdd(name, library [, procedure])` with a package library loads the whole library and then
  answers 0 if `name` (caselessly) is one of its routines and 1 otherwise. The procedure argument is
  ignored for package libraries, so aliases do not work:
  `RxFuncAdd('MySqrt', 'rxmath', 'RxCalcSqrt')` returns 1 and `MySqrt(9)` then fails with
  `Error 43.1: Could not find routine "MYSQRT".` A second `RxFuncAdd` of a loaded routine returns 0.
* Once the library is loaded by any route, every routine in it is callable from every package in the
  process, and `RxFuncQuery(name)` returns 0 for each of them, even for names that were never passed
  to `RxFuncAdd`.
* `RxFuncDrop(name)` on a routine that came from a package library returns 1 and the routine stays
  callable.
* Routine names compare caselessly (`RxCalcSinh` calls the routine registered as `RxCalcSinH`).

### Argument conversion and the errors it raises

Typed arguments are converted before the routine runs, in argument order, and the first failure is
raised. The messages below are the secondary messages (`condition('o')~message`). The `&n`
placeholders that are not substituted in the oracle are shown as they appear.

| Situation | Error | Message |
|---|---|---|
| a required argument omitted | 88.901 | `Missing argument; argument N is required.` |
| more arguments than declared | 88.922 | `Too many arguments in invocation; K expected.` |
| 32-bit integer argument not a whole number in range | 88.907 | `Argument N must be in the range -2147483648 to 2147483647; found "V".` |
| positive whole number argument is not one | 88.905 | `Argument N must be a positive whole number; found "V".` |
| floating argument not convertible | 88.921 | `Argument N must be a valid double value; found "V".` |

Whole-number arguments accept surrounding blanks (`" 3 "` is 3). An argument written as an empty
position (`f(4,,)`) counts as omitted.

A routine that rejects its input after conversion raises `Error 40.1: External routine "NAME"
failed.` where NAME is the routine name as called, in upper case. This text calls that a *routine
failure*.

## rxmath

### Routines

Library `rxmath`. All routines below are in it. None has side effects except `MathLoadFuncs`.

| Routine | Arguments | Result |
|---|---|---|
| `MathLoadFuncs` | `[version]` | `""` |
| `MathDropFuncs` | none | `""` |
| `RxCalcPi` | `[precision]` | π |
| `RxCalcSqrt` | `x [, precision]` | √x |
| `RxCalcExp` | `x [, precision]` | eˣ |
| `RxCalcLog` | `x [, precision]` | ln x |
| `RxCalcLog10` | `x [, precision]` | log₁₀ x |
| `RxCalcSinH` | `x [, precision]` | sinh x |
| `RxCalcCosH` | `x [, precision]` | cosh x |
| `RxCalcTanH` | `x [, precision]` | tanh x |
| `RxCalcPower` | `x, y [, precision]` | xʸ |
| `RxCalcSin` | `angle [, precision [, units]]` | sine |
| `RxCalcCos` | `angle [, precision [, units]]` | cosine |
| `RxCalcTan` | `angle [, precision [, units]]` | tangent |
| `RxCalcCotan` | `angle [, precision [, units]]` | cotangent |
| `RxCalcArcSin` | `x [, precision [, units]]` | arcsine |
| `RxCalcArcCos` | `x [, precision [, units]]` | arccosine |
| `RxCalcArcTan` | `x [, precision [, units]]` | arctangent |

`MathLoadFuncs` with any argument present writes these four lines to standard output, then returns
`""`. Without an argument it writes nothing. It does not register anything; loading is described
above.

```
rxmath 1.2 - REXX mathematical function library
Copyright (c) 2005-2021 Rexx Language Association.
All rights reserved.

```

`MathDropFuncs` does nothing. Given an argument it raises 88.922 with `0 expected`.

### Number arguments

`x`, `y` and `angle` are floating arguments. A floating argument is accepted if it is a Rexx number
or exactly one of the strings `nan`, `+infinity` and `-infinity` (lower case only). `NAN`,
`Infinity`, `inf`, `+inf` and `-nan` raise 88.921. A Rexx number converts to the double nearest to
the decimal value as written, using all of its digits whatever `NUMERIC DIGITS` is:
`RxCalcSqrt('2.0000000001', 16)` is `1.414213562408450`, not √2. A number too large for a double
becomes ±infinity (`RxCalcSqrt('1e400')` is `+infinity`), and one too small becomes 0.

### Precision

`precision` must be a positive whole number. 0, negatives, `1.5` and `abc` raise 88.905. A value
above 16 is treated as 16. If it is omitted, the caller's `NUMERIC DIGITS` is used, capped at 16. The
units argument is checked after all typed arguments have converted.

### Units

`units` applies to the trigonometric routines. If omitted, the unit is degrees. Otherwise only its
first character matters, caselessly: `D` for degrees, `R` for radians and `G` for grades (400 to a
circle). `radxx` therefore means radians. Any other first character, and the empty string, raise
88.916 `Argument 3 must be one of D, R, or G; found "V".` This error replaces the result even when
the result would have been `nan`.

### Result formatting

Every result is a double `v` rendered with precision `p`:

* NaN gives `nan`, positive infinity `+infinity` and negative infinity `-infinity`.
* Otherwise, take the C `%.*g` rendering of `v` with `p + 2` significant digits. Read that text as a
  Rexx number and round it to `p` significant digits, rounding half up on the decimal text. The
  result is that number as Rexx arithmetic would show it with `NUMERIC DIGITS p` and `NUMERIC FORM
  SCIENTIFIC`, whatever the caller's form is.
* Trailing zeros that `%g` dropped stay dropped. Zeros produced by the rounding stay. Negative zero
  shows as `0`.

Rounding is done twice, first to `p + 2` digits by `%g` and then to `p`. It works on the decimal
text, not on the binary value:

| Call | Result | Why |
|---|---|---|
| `RxCalcSqrt(3)` (digits 9) | `1.73205081` | |
| `RxCalcSqrt(3,16)` | `1.732050807568877` | |
| `RxCalcSqrt(25,16)` | `5` | `%.18g` gives `5` |
| `RxCalcPower(10,9)` | `1.00000000E+9` | `%.11g` is `1000000000`; rounding to 9 keeps zeros |
| `RxCalcPower(10,8)` | `100000000` | |
| `RxCalcPower(10,20)` | `1E+20` | `%.11g` is `1e+20` |
| `RxCalcPower(10,-5)` | `0.00001` | |
| `RxCalcPower(10,-20)` | `1E-20` | |
| `RxCalcSqrt(1e-20,16)` | `0.0000000001000000000000000` | `%.18g` is `1.00000000000000004e-10` |
| `RxCalcPower(6.13,3,1)` | `2E+2` | |
| `RxCalcSqrt(90.25,1)` | `1E+1` | |
| `RxCalcPower(1.05,1,2)` | `1.1` | `%.4g` is `1.05`, half up |
| `RxCalcPower(0.125,1,2)` | `0.13` | |
| `RxCalcPower(0.15,1,1)` | `0.2` | |
| `RxCalcSqrt(99.8001,2)` | `10` | |
| `RxCalcPower(2,100,16)` | `1.267650600228229E+30` | |
| `RxCalcPower(2,1023)` | `8.98846567E+307` | |
| `RxCalcExp(-700)` | `9.85967654E-305` | |
| `RxCalcExp(-745.2)`, `RxCalcExp(-746)` | `0` | |
| `RxCalcSin(3.14159265358979,16,'r')` | `0.000000000000003231089148865173` | |
| `RxCalcSqrt(2,10)` under `NUMERIC DIGITS 5` | `1.414213562` | precision, not digits |
| `RxCalcExp(20)` under digits 5 | `4.8517E+8` | |
| `RxCalcPower(10,10)` under `NUMERIC FORM ENGINEERING` | `1.00000000E+10` | form ignored |

### Values computed

The value is the platform C library's (glibc `libm`) result for the double operation named.
Implementations should call the same functions: Rust's `f64` methods `sqrt`, `exp`, `ln`, `log10`,
`sinh`, `cosh`, `tanh`, `powf`, `sin`, `cos`, `tan`, `asin`, `acos` and `atan` resolve to glibc on
Linux.

* `RxCalcPi` is the double nearest π: `3.14159265` at 9 digits, `3.141592653589793` at 16.
* `RxCalcLog` and `RxCalcLog10` give `nan` for any negative `x` and `-infinity` for 0.
* `RxCalcArcSin` and `RxCalcArcCos` give `nan` for `x < -1` or `x > 1`.
* `RxCalcSqrt(-1)` is `nan`. `RxCalcPower(-2, 5.1)` is `nan`. `RxCalcPower(0,-1)` is `+infinity`.
  `RxCalcPower(0,0)` is `1`. `RxCalcPower(-8,1/3)` is `nan`.
* A `nan` argument gives `nan` from every routine.
* Infinite arguments follow IEEE rules: `RxCalcExp('-infinity')` is `0` and
  `RxCalcArcTan('+infinity')` is `90`. `RxCalcExp(709.8)` and `RxCalcPower(2,1024)` are `+infinity`.

**Radians.** `sin`, `cos` and `tan` are applied directly to the argument. Cotangent is `1 / tan(x)`,
so `RxCalcCotan(0,,'R')` is `+infinity`.

**Degrees.** The angle is reduced before any library call, so multiples of 90° give exact results:
`RxCalcSin(180)` is `0`, `RxCalcCos(90)` is `0`, `RxCalcTan(90)` is `+infinity` and
`RxCalcTan(270)` is `-infinity`. Let `s` be −1 if the angle is negative and +1 otherwise. Let
`a = fmod(|angle|, 360)`, which is exact. Choose the reduced angle `r` in degrees and the formula by
the range of `a`:

| Range of `a` | `r` | sin | cos | tan | cotan |
|---|---|---|---|---|---|
| `a ≤ 45` | `a` | `s·sin ρ` | `cos ρ` | `s·tan ρ` | `s / tan ρ` |
| `45 < a < 135` | `90 − a` | `s·cos ρ` | `sin ρ` | `s / tan ρ` | `s·tan ρ` |
| `135 ≤ a ≤ 225` | `180 − a` | `s·sin ρ` | `−cos ρ` | `−s·tan ρ` | `−s / tan ρ` |
| `225 < a < 315` | `a − 270` | `−s·cos ρ` | `sin ρ` | `−s / tan ρ` | `−s·tan ρ` |
| `a ≥ 315` | `a − 360` | `s·sin ρ` | `cos ρ` | `s·tan ρ` | `s / tan ρ` |

Here ρ is `r` in radians. The subtraction giving `r` is a double operation. ρ is computed as `r × π`
and then `÷ 180` in x87 extended precision (64-bit significand), each operation rounded to extended,
using the extended value nearest π. The result is then rounded to double. This matters:
`RxCalcSin(30,16)` is `0.5` only because ρ rounds to `0.52359877559829892668`. Computing ρ in double
arithmetic gives `0.4999999999999999`. A division by `tan(+0)` gives a signed infinity:
`RxCalcCotan(180)` is `-infinity` and `RxCalcCotan(-180)` is `+infinity`.

Observed values: `RxCalcSin(-30)` is `-0.5`, `RxCalcSin(390)` is `0.5`, `RxCalcTan(-45)` is `-1`,
`RxCalcCotan(45)` is `1`, `RxCalcCotan(90)` is `0`, `RxCalcTan(45,16)` is `0.9999999999999999`,
`RxCalcSin(1e20)` is `-0.984807753` and `RxCalcSin(12)` is `0.207911691`.

**Grades.** The same scheme applies with 400 in place of 360. The range limits become 50, 150, 250
and 350, the offsets become 100, 200, 300 and 400, and ρ is `r × π ÷ 200`. `RxCalcSin(100,,'g')` is
`1`, `RxCalcCos(200,,'g')` is `-1` and `RxCalcSin(12,,'g')` is `0.187381315`.

**Arc functions.** The library returns radians. For degrees the result is `(v × 180)`, a double
product, then `÷ π` in extended precision, rounded to double. For grades it is the same with 200.
`RxCalcArcSin(1)` is `90`, `RxCalcArcSin(1,,'g')` is `100`, `RxCalcArcSin(1,,'r')` is `1.57079633`,
`RxCalcArcCos(-1)` is `180`, `RxCalcArcSin(.5,16)` is `30` and `RxCalcArcTan(.1526,,'g')` is
`9.64044525`.

## rxregexp

### What is visible

The library `rxregexp` holds no routines. It holds five native methods: `RegExp_Init`,
`RegExp_Uninit`, `RegExp_Parse`, `RegExp_Pos` and `RegExp_Match`. They are used only through the
package `rxregexp.cls`, which programs load with `::requires "rxregexp.cls"`. That package defines a
single public class, `RegularExpression`, with these methods:

| Method | Kind |
|---|---|
| `new` (class) | takes `init`'s arguments. A SYNTAX error from `init` is raised to the caller of `new` (traceback line per Deviation 89, see "Rulings"). |
| `init([pattern [, matchtype]])` | native |
| `uninit` | native |
| `parse(pattern [, matchtype])` | native |
| `match(string)` | native |
| `pos(haystack)` | native |
| `position` | Rexx, takes no arguments |

Each instance holds a compiled pattern, a match mode (MAXIMAL or MINIMAL) and the object variable
`!POS`, which `position` returns.

### Methods

**`init([pattern [, matchtype]])`**, reached through `new`. More than two arguments raise 88.922
with `2 expected`.

1. If `matchtype` is given, it must be `MINIMAL` or `MAXIMAL`, compared caselessly. `CURRENT` is not
   accepted here. Anything else, including `""`, raises error 93 with no subcode and no secondary
   message (`Error 93 ... Incorrect call to method.`).
2. If `pattern` is given, it is compiled. A compile error raises error 38 with no subcode (`Error 38
   ... Invalid template or pattern.`). `new("")` raises 38. If both arguments are bad, the error is
   38.
3. The mode defaults to MAXIMAL.
4. `!POS` is not assigned. `position` therefore returns the string `!POS` until `parse`, `match` or
   `pos` runs. This holds even after `new('ab')`.
5. `new` takes `init`'s arguments; a SYNTAX error from `init` is raised to the caller of `new`
   (traceback line per Deviation 89; see the line-numbers bullet under "Rulings").

**`parse(pattern [, matchtype])`**. A missing pattern raises 88.901 and more than two arguments
raise 88.922.

1. `matchtype` may be `MINIMAL`, `MAXIMAL` or `CURRENT` (caseless). `CURRENT` and an omitted
   `matchtype` keep the mode. Anything else, including `""`, raises error 93 with no subcode. The
   mode is set before the pattern is compiled.
2. The method returns 0 on success or an error code from 1 to 6 (see "Compile errors"). It never
   raises for a bad pattern. `parse("")` returns 1.
3. It sets `!POS` (see "The position after parse").
4. After an error, the object matches everything: `match(x)` returns 1 with position 0, and `pos(x)`
   returns 1 with position 0.

**`match(string)`** returns 1 or 0 and sets `!POS`. A missing argument raises 88.901 and two
arguments raise 88.922.

**`pos(haystack)`** returns the 1-based start of the first match, or 0, and sets `!POS`. Argument
errors are as for `match`.

**`position`** returns `!POS`. Any argument raises 93.902 `Too many arguments in invocation of
method; 0 expected.`

**`uninit`** frees the compiled pattern. **Defect:** calling any native method after an explicit
`uninit` crashes the process (D3).

### The pattern language

A pattern is a byte string. A NUL byte ends it: everything after the first `'00'x` is ignored, so
`'a'||'00'x||'b'` compiles as `a`. Grammar, with alternation binding loosest:

```
expression ::= term [ "|" expression ]
term       ::= factor { factor }            -- a factor begins with "(", "[", or a literal byte
factor     ::= atom [ "*" | "+" | "{" count "}" ]
atom       ::= "?" | "\" byte | "[" set "]" | "(" expression ")" | literal
```

* **Literal.** Any byte except `( ) { } | * + [ ] \ ?` and NUL matches itself, compared as an exact
  byte. This includes `.`, `^`, `$`, `-`, `:`, blank, and bytes ≥ `'80'x`. There are no anchors and
  no case folding.
* **`?`** matches any one byte.
* **`\b`** matches byte `b` literally, for any `b` other than NUL. `\n` matches the letter `n`.
  `\` at the end of the pattern is error 4.
* **`( … )`** groups. `()` is error 1, because a group must contain a term. An empty alternative is
  also error 1: `a(b|)c`, `a|`, `|a`.
* **`X*`** matches zero or more `X`, **`X+`** one or more, and **`X{n}`** exactly `n`.
  * The `count` is all text up to the next `}`, read the way C `atoi` reads it: leading white space,
    an optional sign, digits up to the first non-digit, and 0 if there are no digits. So
    `a{ 3}` and `a{3x}` mean 3, and `a{e3}` means 0.
  * `n` must be at least 2. `{0}`, `{1}` and `{-1}` are error 5.
  * Reaching the end of the pattern before `}` is error 4 (`a{2`).
* **One quantifier per factor.** A second quantifier is not an error. It stops the parse, and the
  rest is ignored: `a**` behaves as `a*`, `a{2}*` as `a{2}`, and `a{2}{2}` as `a{2}`.
* **Trailing text is ignored.** Parsing stops at the first byte that cannot continue the
  expression, and that byte and everything after it are ignored. A pattern that stops after
  consuming at least one complete expression compiles successfully. `a)`, `a]`, `a}` and `ab*+c`
  compile as `a`, `a`, `a` and `ab*`. `x)` matches `"x"`. A pattern that cannot start, such as `*`,
  `)`, `{`, `}`, `]`, `+` or `|`, is error 1. A byte that cannot start an atom is error 1 wherever
  an atom is required: `(A|+)`, `A({})`.
* **Sets.** `[` is optionally followed by `^`, which negates the set, and then by items up to `]`.
  An item is one of the following:
  * `\b`, the byte `b` literally, including `]`, `-`, `:`, `^` and `\`. `\` at the end of the
    pattern is error 3.
  * `:NAME:`, a class. `NAME` is matched caselessly against `ALPHA`, `LOWER`, `UPPER`, `ALNUM`,
    `DIGIT`, `XDIGIT`, `BLANK`, `SPACE`, `CNTRL`, `PRINT`, `PUNCT` and `GRAPH`. An unknown name, a
    missing closing `:`, or a name longer than 15 bytes is error 6. `[a:]` is error 6 because `:`
    always starts a class. Classes use the C locale's ASCII classification: no byte ≥ `'80'x` is in
    any class. `BLANK` is blank and tab. `SPACE` is `'09'x` to `'0D'x` and blank. `CNTRL` is `'00'x`
    to `'1F'x` and `'7F'x`. `PRINT` is `'20'x` to `'7E'x`. `GRAPH` is `PRINT` without blank.
    `PUNCT` is `GRAPH` without `ALNUM`.
  * `-`. As the first item, or after only `^`, it is a literal `-`. Otherwise it is a range. Let
    `b0` be the pattern byte immediately before the `-` and `b1` the pattern byte immediately after
    it, taken literally even if it is `\` or `:`. If `b1` is `]` or the end of the pattern, the
    result is error 3. Compare `b0` and `b1` as signed bytes (−128..127) and call the smaller one
    `lo` and the larger `hi`. The range adds every byte whose signed value `v` satisfies
    `lo < v ≤ hi`; `lo` itself is not added. `b0` was usually added already as an item, so `[a-c]`
    is {a,b,c}. But `[c-a]` is {c,b}, without `a`. `[:alpha:-z]` adds `';'` to `'z'` but not `:`. In
    `[a-c-e]` the second range starts at `c`, giving {a,b,c,d,e}. In `[\--/]` the range starts at
    `-`, giving {`-`,`.`,`/`}. In `[a-\]]` the range runs to `\`. Because the comparison is signed,
    `[a-'E9'x]` is the bytes `'EA'x`..`'FF'x` and `'00'x`..`'61'x`. **Defect:** a range whose upper
    end is `'7F'x` never finishes compiling, and memory grows (D2).
  * Any other byte is itself, including `[`, `(`, `|`, `*`, `?`, and `^` when it is not first.

  A set with no members is error 3: `[]a]`, `[^]`. A set not followed by `]` is error 3: `[`, `[a`,
  `[:alpha:`. A set matches one byte by membership. `^` complements the set over all 256 byte
  values, so `[^a]` matches `'00'x`.

### Compile errors

| Code | Meaning | Position after the error (0-based byte index) |
|---|---|---|
| 1 | unexpected symbol | index of the offending byte, or the pattern length at the end (`*`→0, `(`→1, `()`→1, `a\|`→2, `(A\|+)`→3, `A({})`→2, `ab(c\|)`→5, `a\|b\|`→4) |
| 2 | missing `)` | where `)` was expected (`a(b`→3, `ab(c\|d`→6) |
| 3 | illegal set | just after `[` when the set's contents are bad (`[a-]`→1, `[]a]`→1, `[\`→1, `abc[]`→4). After the contents when `]` is missing (`[a`→2, `ab[c`→4, `[:alpha:`→8). |
| 4 | unexpected end | after the `\` for a trailing escape (`\`→1, `ab\`→3). The index of `{` for a missing `}` (`ab{2`→2). |
| 5 | illegal number | index of `{` (`ab{0}`→2, `a{1}`→1). Also a count whose copies would take the automaton past 2,000,000 states (Deviation 87): `a{2000000000}`→1, `((a{1000}){1000}){1000}`→17; where only the sum of several counts passes it, the last `{`. |
| 6 | bad class name | just after `[` (`ab[:foo:]`→3, `[:alphaxxxxxxxxxxxxxxxxx:]`→1) |

### The position after parse

On success, `!POS` is the number of pattern bytes consumed. This is the pattern's length unless
trailing text was ignored, in which case it is the index where parsing stopped: `abc{2}x`→7,
`ab*+c`→3, `a)`→1. On error, it is the position given in the table above.

### Matching

The compiled pattern denotes a set of byte strings (its *language*). **Defect:** for some patterns
the oracle's language differs from the grammar's meaning (D4, D5). See "Resolutions".

**`match` in MAXIMAL mode** succeeds if the whole subject is in the language. A NUL byte inside the
subject also ends it: matching succeeds if, for some `k`, the first `k` bytes are in the language
and either `k` is the subject length `N` or byte `k + 1` is `'00'x`. With the smallest such `k`,
`!POS` is `N` if `k = N` and `k + 1` otherwise. So `a` matches `'a'||'00'x||'b'` with position 2,
and `a` matches `'a'||'00'x` with position 2.

**`match` in MINIMAL mode** succeeds if any prefix of the subject, including the empty prefix, is in
the language. `!POS` is the length of the shortest such prefix. `a*` against `aaa` gives 1 with
position 0. `ab+` against `abbbbb` gives position 2. `?*foo` against `xfooxxxxxxfo` gives 1 with
position 4. Nothing past the subject's end is ever consumed.

**On failure**, `!POS` is the length of the longest prefix of the subject that at least one partial
match consumed, capped at `N` (`a?` against `a` gives 1; `abc?` against `abc` gives 3, verified).
Examples: `abc` against `abcd` gives 0 with
position 3. `abc` against `xabc` gives position 0. `a?` against `a` gives position 1. `ab|cd` against
`abd` gives position 2. `ab` against `'a'||'00'x` gives position 1.

**`pos(haystack)`** searches whatever the mode, and the mode is the same afterwards:

1. For `k = 0, 1, …, N` in turn, try a MINIMAL match of the suffix starting after `k` bytes. The
   trial at `k = N` matches against the empty suffix. Let `k` be the first offset that succeeds and
   `m` the shortest match there.
2. If none succeeds, return 0 with `!POS` = 0.
3. In MINIMAL mode, return `k + 1` with `!POS` = `k + m`. `!POS` is then the 1-based index of the
   last matched byte, or `k` for an empty match.
4. In MAXIMAL mode, at the same `k`, try MAXIMAL `match` on the haystack bytes `k+1 .. k+L` for
   `L` = `N − k` down to 0. Stop at the first `L` that succeeds. Return `k + 1` with `!POS` =
   `k + L`. With a NUL inside that substring, `!POS` is `k` plus the MAXIMAL match position of the
   substring.

So the start is always the leftmost start of the shortest match, and MAXIMAL only extends it. This
is the quirk the test group documents: `b+` on `abbbc` gives start 2, end 4 in MAXIMAL mode and end
2 in MINIMAL mode. `b*` on `abbbc` gives 1 with position 0 in both modes.

### Worked examples

Each was run on the oracle. Columns: mode, pattern, subject → parse rc@position, match@position,
pos@position.

```
max abc        abc          0@3  1@3  1@3
max abc        xabc         0@3  0@0  2@4
min abc        ab           0@3  0@2  0@0
max a*         ""           0@2  1@0  1@0
max b+         abbbc        0@2  0@0  2@4
min b+         abbbc        0@2  0@0  2@2
max ab+        abbbbb       0@3  1@6  1@6
min ab+        abbbbb       0@3  1@2  1@2
max a|b        b            0@3  1@1  1@1
max a|b        xbx          0@3  0@0  2@2
max ab|cd      abd          0@5  0@2  1@2
max (a|b){3}   abba         0@8  0@3  1@3
max a{2}*      aaaa         0@4  0@2  1@2      (the * is ignored)
max a)         a            0@1  1@1  1@1
max a(b        ab           2@3  1@0  1@0      (after an error everything matches)
max [a-%e9]    %ea          0@5  1@1  1@1      (%hh is a byte)
max [a-%e9]    %e9          0@5  0@0  0@0
max a          a%00b        0@1  1@2  1@2
min a          a%00b        0@1  1@1  1@1
max ?*foo      xfooxxxxxxfoo 0@5 1@13 1@13
min ?*foo      xfooxxxxxxfo 0@5  1@4  1@4
max (a|ab)(c|bcd) abcd      0@13 1@4  1@4
max x(ab|c){2}y xabcy       0@11 1@5  1@5
max (ab|cd)*e  abcde        0@9  1@5  1@5
max a+b+       xaabbbyab    0@4  0@0  2@6
```

The csv pattern from the test group, `("[^"]*" *, *)*"[^"]*"`, matches `'" " ,'` repeated 20 times
followed by ` "final"`. The ooTest `test_bug_1045` cases hold.

## rxunixsys

### Routines

Library `rxunixsys` (`::requires 'rxunixsys' LIBRARY` or `loadLibrary('rxunixsys')`). It is not
loaded by default: without it every routine below is error 43.1. Integer arguments are 32-bit
(88.907 on failure) and string arguments may be any string.

"Return code" below means the OS call's result exactly: 0 or a value on success, and −1 on failure.
No routine raises because an OS call failed. The cause is read separately with `SysGeterrno`.

**Process and identity**

| Routine | Result |
|---|---|
| `SysGetpid()`, `SysGetppid()` | process id, parent id |
| `SysGettid()` | the calling thread's `pthread_self()` value as an unsigned 64-bit decimal (for example `139826341637056`). This is not the kernel thread id. With digits 9 it is not a `datatype(,'W')` whole number, but it is `NUM`. |
| `SysGetuid()`, `SysGeteuid()`, `SysGetgid()`, `SysGetegid()` | ids |
| `SysSetuid(uid)` | returns the `setuid(uid)` return code. SysSetuid also drops the supplementary group list. |
| `SysSeteuid(uid)`, `SysSetgid(gid)`, `SysSetegid(gid)` | return code |
| `SysSetsid()` | the `setsid()` result: the new session id or −1 |
| `SysGetsid(pid)` | `getsid(pid)`. `pid` 0 means the caller. |
| `SysSetpgrp()` | `setpgrp()` return code |
| `SysGetpgrp()` | process group |
| `SysSetpgid(pid, pgid)` | return code |
| `SysGetpgid(pid)` | `getpgid(pid)` |
| `SysKill(pid, sig)` | `kill()` return code |
| `SysSignal(sig, action)` | `action`'s first character, caselessly: `D` sets the default disposition for `sig` and `I` ignores it. The result is 0 even if `sig` is invalid. An empty or other action is a routine failure. With SIGCHLD (17) ignored, a command's return code is negative with no FAILURE trace: the oracle's value is unset stack (measured −35, −64, −100, −112), this crate's −10 (−ECHILD). |
| `SysUmask(mask)` | sets the mask and returns the previous mask in decimal (`SysUmask(18)` returns `2`, then `18`) |

**Files**

| Routine | Result |
|---|---|
| `SysSymlink(target, link)`, `SysLink(old, new)`, `SysUnlink(path)` | return code |
| `SysChown(path, uid, gid)`, `SysLchown(path, uid, gid)` | return code. −1 means "unchanged". |
| `SysChroot(path)` | returns the `chroot(path)` return code. SysChroot leaves the current directory at `/` even when chroot fails. |
| `SysClose(fd)` | return code |
| `SysAccess(path, mode)`, `SysEuidaccess(path, mode)` | return code of `access` or `euidaccess`. The mode bits are 4 read, 2 write, 1 execute and 0 existence. |
| `SysMkdirUnix(dir, mode)` | `mkdir(dir, mode)` return code. `mode` is a decimal number (448 = 0700) and is subject to the umask. |
| `SysRmdirUnix(dir)` | return code |
| `SysChmod(path, perm)` | see below |
| `SysStat(path, option)` | see below |
| `SysGetdirlist(dir)` | an Array of all entry names, including `.` and `..`, in the order the directory stream returns them (unsorted). An unopenable directory gives an empty Array. |

The Rexx names are `SysMkdirUnix` and `SysRmdirUnix`. `SysMkdir` and `SysRmdir` are the core
`RexxUtil` routines and are unaffected.

`SysChmod(path, perm)`: `perm` must be exactly 9 characters, case-sensitive, with each position from
its own set:

| Positions | Allowed |
|---|---|
| 1, 4, 7 | `r` or `-` |
| 2, 5, 8 | `w` or `-` |
| 3, 6 | `x` (execute), `s` (execute and set-id), `S` (set-id only), or `-` |
| 9 | `x`, `t` (execute and sticky), `T` (sticky only), or `-` |

Position 3's set-id is set-user-id and position 6's is set-group-id. Any other length or character is
a routine failure. The result is the `chmod` return code.

`SysStat(path, option)`:

* An empty `path` or empty `option` is a routine failure.
* If the file does not exist, the result is `""` for every option, valid or not.
* Otherwise the first character of `option`, caselessly, selects the result. Any other character is
  a routine failure.
* The file is examined with `stat`, which follows symbolic links.

| First character | Result |
|---|---|
| `D` | device id |
| `I` | inode |
| `N` | link count |
| `U` | uid |
| `G` | gid |
| `R` | special-file device id (`/dev/null` gives 259 here) |
| `S` | size in bytes |
| `A`, `M`, `C` | access, modification and status-change time, in local time, as `YYYY-MM-DD HH:MM:SS` |
| `P` | a 10-character `ls -l` style permission string |

For `P`, the first character is the type: `-` regular, `d` directory, `l` symlink, `s` socket, `c`
character device, `b` block device, `p` FIFO. The nine permission characters use `s`/`S` and
`t`/`T` as in `ls`. Examples: `drwxr-x--T`, `drwsr-S--t`, `crw-rw-rw-`.

**Users, groups and services**

`SysGetpwnam(user, option)` and `SysGetpwuid(uid, option)`:

* An empty `user` or empty `option` is a routine failure.
* If no such user exists, the result is `""` for every option, valid or not.
* Otherwise the first character of `option`, caselessly, selects the result: `N` login name, `U`
  uid, `G` gid, `R` real name (GECOS), `D` home directory, `S` shell, `P` always `""`. Any other
  character is a routine failure.

`SysGetgrnam(group, option)` and `SysGetgrgid(gid, option)`:

* An empty `group` or empty `option` is a routine failure.
* The first character of `option`, caselessly, selects the result: `N` group name, `G` gid, `P`
  always `""`, `M` an Array of member names (possibly empty). Any other character is a routine
  failure. That check happens even when the group does not exist.
* **Defect:** a group that does not exist crashes the process (D6).

`SysGetservbyname(name, proto, option)` and `SysGetservbyport(port, proto, option)`:

* Empty `name` or `proto` is a routine failure, as is a `port` ≤ 0 or ≥ 65535.
* If the service is not found, the result is `""`.
* Otherwise the first character of `option`, caselessly, selects the result: `N` official name, `P`
  port number, `A` an Array of aliases. Any other character, including an empty option, is a routine
  failure.
* `SysGetservbyname('http','tcp','p')` is 80 and `SysGetservbyport(80,'tcp','n')` is `http`.

**System**

| Routine | Result |
|---|---|
| `SysUname([option])` | omitted: system name (`Linux`). The first character, caselessly, selects: `S` system, `N` node name, `R` release, `V` version, `M` machine (`x86_64`). Other characters and `""` are routine failures. |
| `SysGethostname()` | the host name, equal to `SysUname('n')` |
| `SysGetsizeofptr()` | `64` |
| `SysGettzname1()`, `SysGettzname2()` | after re-reading the time zone, the standard and daylight zone abbreviations (`CET`, `CEST`) |
| `SysWordexp(text)` | an Array of the words of POSIX word expansion of `text`, as glibc `wordexp` with no flags answers it, against the program's environment and directory. Command substitution raises 40.1 and no shell is started (Deviation 90). `SysWordexp('a b  c')` is `a,b,c` and `$HOME` expands. Empty `text` is a routine failure. |
| `SysSetxattr(path, name, value)` | sets the extended attribute to `value` plus a trailing NUL byte, and returns the return code |
| `SysGetxattr(path, name)` | the stored value up to its first NUL, or `""` if it is absent or unreadable |
| `SysListxattr(path)` | an Array of attribute names, or `""` (a string, not an Array) on failure |
| `SysRemovexattr(path, name)` | return code |
| `SysGeterrno()` | the calling thread's C `errno` now, as a decimal |
| `SysGeterrnomsg(n)` | message `n` from the table below. `n` outside 0..132, including negatives, gives `Unknown`. |

`SysGeterrno` is not tied to any routine. It reports the thread's last OS error, and the interpreter
makes OS calls of its own. Immediately after a failing call it is correct:
`r = SysUnlink('/nonexist/x'); e = SysGeterrno()` gives `-1 2`. With a `SAY` in between, the oracle
reports an errno from the console write. At program start the value is unspecified (observed 25 or
29 depending on the host and on where standard output goes). See the errno bullet under "Rulings".

`SysGeterrnomsg` table, indexed from 0:

```
0 Ok | 1 Operation not permitted | 2 No such file or directory | 3 No such process
4 Interrupted system call | 5 I/O error | 6 No such device or address | 7 Argument list too long
8 Exec format error | 9 Bad file number | 10 No child processes | 11 Try again | 12 Out of memory
13 Permission denied | 14 Bad address | 15 Block device required | 16 Device or resource busy
17 File exists | 18 Cross-device link | 19 No such device | 20 Not a directory | 21 Is a directory
22 Invalid argument | 23 File table overflow | 24 Too many open files | 25 Not a typewriter
26 Text file busy | 27 File too large | 28 No space left on device | 29 Illegal seek
30 Read-only file system | 31 Too many links | 32 Broken pipe | 33 Math argument out of domain of func
34 Math result not representable | 35 Resource deadlock would occur | 36 File name too long
37 No record locks available | 38 Function not implemented | 39 Directory not empty
40 Too many symbolic links encountered | 41 Operation would block | 42 No message of desired type
43 Identifier removed | 44 Channel number out of range | 45 Level 2 not synchronized
46 Level 3 halted | 47 Level 3 reset | 48 Link number out of range | 49 Protocol driver not attached
50 No CSI structure available | 51 Level 2 halted | 52 Invalid exchange
53 Invalid request descriptor | 54 Exchange full | 55 No anode | 56 Invalid request code
57 Invalid slot | 58 Deadlock | 59 Bad font file format | 60 Device not a stream
61 No data available | 62 Timer expired | 63 Out of streams resources
64 Machine is not on the network | 65 Package not installed | 66 Object is remote
67 Link has been severed | 68 Advertise error | 69 Srmount error | 70 Communication error on send
71 Protocol error | 72 Multihop attempted | 73 RFS specific error | 74 Not a data message
75 Value too large for defined data type | 76 Name not unique on network
77 File descriptor in bad state | 78 Remote address changed
79 Can not access a needed shared library | 80 Accessing a corrupted shared library
81 .lib section in a.out corrupted | 82 Attempting to link in too many shared libraries
83 Cannot exec a shared library directly | 84 Illegal byte sequence
85 Interrupted system call should be restarted | 86 Streams pipe error | 87 Too many users
88 Socket operation on non-socket | 89 Destination address required | 90 Message too long
91 Protocol wrong type for socket | 92 Protocol not available | 93 Protocol not supported
94 Socket type not supported | 95 Operation not supported on transport endpoint
96 Protocol family not supported | 97 Address family not supported by protocol
98 Address already in use | 99 Cannot assign requested address | 100 Network is down
101 Network is unreachable | 102 Network dropped connection because of reset
103 Software caused connection abort | 104 Connection reset by peer | 105 No buffer space available
106 Transport endpoint is already connected | 107 Transport endpoint is not connected
108 Cannot send after transport endpoint shutdown | 109 Too many references: cannot splice
110 Connection timed out | 111 Connection refused | 112 Host is down | 113 No route to host
114 Operation already in progress | 115 Operation now in progress | 116 Stale NFS file handle
117 Structure needs cleaning | 118 Not a XENIX named type file | 119 No XENIX semaphores available
120 Is a named type file | 121 Remote I/O error | 122 Quota exceeded | 123 No medium found
124 Wrong medium type | 125 Operation Canceled | 126 Required key not available
127 Key has expired | 128 Key has been revoked | 129 Key was rejected by service | 130 Owner died
131 State not recoverable | 132 Operation not possible due to RF-kill
```

Verified on the oracle: 41 is `Operation would block`, 88 is `Socket operation on non-socket`, 132
is the last entry, and 133 gives `Unknown`.

**`SysCrypt`** is absent from the oracle, which was built without `crypt()`. Calling it is
`Error 43.1: Could not find routine "SYSCRYPT".`, so ooTest `test_SysCrypt` fails on the oracle.
**Documented by the source:** `SysCrypt(str, salt)` returns `crypt(str, salt)`, or `""` when that
returns nothing. An empty `str` or `salt` raises error 40 with the "null argument" subcode,
naming `SYSCRYPT` and the argument number. See the SysCrypt bullet under "Rulings".

## rxsock

### Routines

Library `rxsock`. Only IPv4 is supported. A socket is its OS file descriptor number. The result type
is a decimal integer unless stated otherwise.

**The `errno` and `h_errno` variables.** Most routines set two variables in the caller's variable
pool. Inside a method these are the method's variables, so a method that `expose`s `errno` updates
its object's attribute; `socket.cls` relies on this.

* `h_errno` is always set to `0`.
* On success, `errno` is set to `0`.
* On failure, `errno` is set to the symbolic name for these error numbers: `EWOULDBLOCK`,
  `EINPROGRESS`, `EALREADY`, `ENOTSOCK`, `EDESTADDRREQ`, `EMSGSIZE`, `EPROTOTYPE`, `ENOPROTOOPT`,
  `EPROTONOSUPPORT`, `ESOCKTNOSUPPORT`, `EOPNOTSUPP`, `EPFNOSUPPORT`, `EAFNOSUPPORT`, `EADDRINUSE`,
  `EADDRNOTAVAIL`, `ENETDOWN`, `ENETUNREACH`, `ENETRESET`, `ECONNABORTED`, `ECONNRESET`,
  `ENOBUFS`, `EISCONN`, `ENOTCONN`, `ESHUTDOWN`, `ETOOMANYREFS`, `ETIMEDOUT`, `ECONNREFUSED`,
  `ELOOP`, `ENAMETOOLONG`, `EHOSTDOWN`, `EHOSTUNREACH` and `ENOTEMPTY`. Any other number is set in
  decimal: EBADF is `9` and EPIPE is `32`.
* For host lookups, failure reports the C `errno` rather than the resolver error. That is usually
  `0`.

**Stem arguments.** A stem argument may be a stem object or a string:

* `name.` or `name` (no dot) names the stem `NAME.` in the caller.
* `name.tail` names stem `NAME.`. Every element name the routine reads or writes is prefixed with
  `TAIL`, upper-cased. So `'stem.!'` uses `STEM.!FAMILY` and `'c.x'` uses `C.XFAMILY`. A tail that
  ends in a dot is kept: `'q.ab.'` uses `Q.AB.FAMILY`.
* If the string cannot name a stem (`''`, `'1abc'`), the routine returns 0 at once, does nothing
  else, and does not set `errno`. The exceptions are `SockAccept`, which still accepts the
  connection (its descriptor is lost) and returns 0, and `SockSelect`, which treats the set as
  empty.

**Address stems** have three elements:

* `FAMILY`: on input, `AF_INET` (caseless) or a decimal number; any other text means 0. On output it
  is always a number (`2`).
* `PORT`: a decimal number, reduced modulo 65536. Non-numeric text means 0.
* `ADDR`: on input, `INADDR_ANY` (caseless) means 0.0.0.0. Otherwise it is parsed as a classic
  IPv4 text address: dotted with 1 to 4 parts, decimal, octal or hex, so `127.1` is 127.0.0.1.
  Invalid text means 255.255.255.255. On output it is dotted decimal.

Input values have leading and trailing blanks removed. If any of the three elements is unset, the
whole address is zero (family 0, port 0, 0.0.0.0) and the OS call goes ahead with it: `SockConnect`
with no `ADDR` returns 0.

**Host stems** contain these elements:

| Element | Value |
|---|---|
| `NAME` | official name |
| `ALIAS.0`, `ALIAS.1`, … | aliases |
| `ADDRTYPE` | `AF_INET` |
| `ADDR` | first address |
| `ADDR.0`, `ADDR.1`, … | all addresses |

They hold exactly what the system's legacy host lookup (`gethostbyname`, `gethostbyaddr`) returns.
On this host `localhost` gives `ADDR.0 = 2`, both `127.0.0.1`, and `ALIAS.0 = 2`. Use the same libc
calls for agreement.

**Flag words** are blank-separated, caseless, and unknown words are ignored:

| Routine | Recognized flags |
|---|---|
| `SockSend` | `MSG_OOB`, `MSG_DONTROUTE` |
| `SockRecv`, `SockRecvFrom` | `MSG_OOB`, `MSG_PEEK` |
| `SockSendTo` | `MSG_DONTROUTE` |

**Socket options.** These names are recognized, compared caselessly, and mapped to the Linux numbers
shown:

| Name | Number |
|---|---|
| `SO_DEBUG` | 1 |
| `SO_REUSEADDR` | 2 |
| `SO_TYPE` | 3 |
| `SO_ERROR` | 4 |
| `SO_DONTROUTE` | 5 |
| `SO_BROADCAST` | 6 |
| `SO_SNDBUF` | 7 |
| `SO_RCVBUF` | 8 |
| `SO_KEEPALIVE` | 9 |
| `SO_OOBINLINE` | 10 |
| `SO_LINGER` | 13 |
| `SO_RCVLOWAT` | 18 |
| `SO_SNDLOWAT` | 19 |
| `SO_RCVTIMEO` | 20 |
| `SO_SNDTIMEO` | 21 |
| `SO_USELOOPBACK` | 64 |

`SO_USELOOPBACK` does not exist on Linux. The number 64 is passed through, which Linux knows as a
different option: set returns 0 and a later get returns 1. Any other name gives −1 with
`ENOPROTOOPT`.

| Routine | Behaviour |
|---|---|
| `SockVersion()` | the interpreter version as `major.release.modification`: `5.3.0` |
| `SockLoadFuncs([x])` | `""`. Writes no output. |
| `SockDropFuncs()` | `""` |
| `SockInit()` | `0`. Sets `errno` to `0`. |
| `SockSock_Errno()` | the thread's C `errno` now (unspecified at program start: observed 25 or 29 depending on the host and on where standard output goes); see the errno bullet under "Rulings" |
| `SockPSock_Errno([text])` | writes `RxSOCK Error: ` + text + newline to standard error and returns `""` |
| `SockSocket(domain, type, protocol)` | Blanks are stripped and comparison is caseless. `domain` must be `AF_INET`. `type` must be `SOCK_STREAM`, `SOCK_DGRAM` or `SOCK_RAW`. `protocol` must be `IPPROTO_UDP`, `IPPROTO_TCP` or the exact text `0`. Anything else (`AF_INET6`, `6`) is a routine failure. Returns the new descriptor or −1: `('AF_INET','SOCK_DGRAM','IPPROTO_TCP')` gives −1 with `EPROTONOSUPPORT`, and unprivileged `SOCK_RAW` gives −1. |
| `SockClose(s)`, `SockSoClose(s)` | `close()`. Returns 0 or −1 (`SockClose(9999)` gives −1, `errno` `9`). The interpreter's own descriptors sit at 1000 and above (`rexx_rxapi::reserved`); every rxsock and rxunixsys routine answers `errno` `9` for one of them, as for a closed number. |
| `SockShutDown(s, how)` | `shutdown()`. `how` is 0, 1 or 2. Returns 0 or −1. |
| `SockListen(s, backlog)` | `listen()`. Returns 0 or −1. |
| `SockBind(s, addrstem)`, `SockConnect(s, addrstem)` | Returns 0 or −1. A blocking connect waits. |
| `SockAccept(s [, addrstem])` | Returns the new socket or −1. If a stem is given, it receives the peer's address, even on failure. |
| `SockGetSockName(s, stem)`, `SockGetPeerName(s, stem)` | Returns 0 or −1. The stem is written even on failure (the oracle's values are unspecified; it showed `0`, `0`, `0.0.0.0`). An unconnected socket gives `ENOTCONN`. |
| `SockGetHostByName(name, stem)` | 1 and fills the host stem, or 0 if not found |
| `SockGetHostByAddr(addr, stem [, domain])` | `addr` is parsed like `ADDR`. `domain` defaults to 2 and other numbers are passed through. Returns 1 or 0. |

This crate's two host lookups read `/etc/hosts` and then ask the resolver, in that order, whatever
the host's `/etc/nsswitch.conf` says: they answer as the oracle does on a host whose `hosts:` line is
`files dns`. An answer from the resolver fills no aliases (`ALIAS.0` is 0), since the resolver call
this crate makes reports the canonical name and the addresses only.
| `SockGetHostName()` | the host name, up to 255 bytes |
| `SockGetHostId()` | dotted address of the first IPv4 interface address whose text does not begin `127`, in interface-list order. **Defect:** the last interface in the list is never examined, and if no address qualifies the result is unspecified (D9). `0.0.0.0` if the interface list cannot be read. `errno` is not set on success. |
| `SockSend(s, data [, flags])` | sends the bytes, including NULs. Returns the count or −1. Writing to a closed peer gives −1 with `errno` `32`; the process does not die. |
| `SockSendTo(s, data, addrstem)` or `SockSendTo(s, data, flags, addrstem)` | as `SockSend`, to the given address. With four arguments the third is the flags. |
| `SockRecv(s, var, len [, flags])` | receives up to `len` bytes into the variable named `var`. The variable gets `""` on error or end of file. Returns the count, 0 at end of file, or −1. `len` must be ≥ 0, and a negative `len` is a routine failure. |
| `SockRecvFrom(s, var, len, addrstem)` or `SockRecvFrom(s, var, len, flags, addrstem)` | as `SockRecv`. The stem receives the sender's address. |
| `SockGetSockOpt(s, level, option, var)` | see below |
| `SockSetSockOpt(s, level, option, value)` | see below |
| `SockIoctl(s, cmd, arg)` | see below |
| `SockSelect([reads], [writes], [excepts] [, timeout])` | see below |

`SockAccept`, `SockRecv`, `SockRecvFrom`, `SockConnect` and `SockSelect` block only the calling
activity; other activities keep running. A blocking socket's `SockAccept`, `SockRecv` and `SockRecvFrom` wait no
longer than its `SO_RCVTIMEO`, and `SockConnect` no longer than its `SO_SNDTIMEO`: then
`SockAccept` and the receives answer −1 with `errno` 11, and `SockConnect` −1 with `errno` 115,
the connection going on (a second `SockConnect` gives `errno` 114). A `SockAccept` whose
connection another accept took first waits on. A halting signal during a blocked `SockSend` or
`SockSendTo` ends it, and HALT is raised on that clause.

`SockGetSockOpt(s, level, option, var)`:

* `level` must be `SOL_SOCKET` (caseless), or the call is a routine failure.
* The variable named `var` is set even when the call fails. Its value depends on the option:

| Option | Value |
|---|---|
| `SO_LINGER` | `onoff seconds`, for example `0 0` or `1 7` |
| `SO_TYPE` | `STREAM`, `DGRAM`, `RAW` or `UNKNOWN`. A failed call gives `UNKNOWN`. |
| `SO_RCVTIMEO`, `SO_SNDTIMEO` | milliseconds, as `seconds × 1000 + microseconds ÷ 1000` truncated |
| any other | the integer. A failed call gives `0`. |

* Returns 0 or −1.

`SockSetSockOpt(s, level, option, value)`:

* `level` must be `SOL_SOCKET` (caseless), or the call is a routine failure.
* `SO_TYPE` and `SO_ERROR` return −1, leaving `errno` unchanged.
* `value` is read as a leading decimal integer, so `abc` means 0. `SO_LINGER` reads two integers
  `onoff seconds`, missing ones are 0, and each is truncated to 16 bits.
* `SO_RCVTIMEO` and `SO_SNDTIMEO` take milliseconds.
* Returns 0 or −1. Linux reports twice the requested size when `SO_RCVBUF` or `SO_SNDBUF` is read
  back.

`SockIoctl(s, cmd, arg)`, with `cmd` compared caselessly:

* `FIONBIO`: `arg` must be a 32-bit integer, or the call is a routine failure. Non-zero makes the
  socket non-blocking.
* `FIONREAD`: `arg` names a variable that receives the number of bytes readable. If the call fails
  the variable gets an unspecified number.
* Any other command returns −1 without setting `errno`.
* Returns 0 or −1.

`SockSelect([reads], [writes], [excepts] [, timeout])`:

* Each set is a stem holding `stem.0` = n and `stem.1..n` = socket numbers. A set that is omitted,
  unresolvable, or whose `stem.0` is not a whole number takes no part and is not written back.
  **Defect:** a non-numeric `stem.i` crashes the oracle (D7).
* `timeout` is in seconds and may have a fraction. Omitted means wait indefinitely. Negative means
  0.
* Returns the number of ready descriptors, 0 on timeout, or −1.
* When the result is not 0, each participating stem is rewritten with only its ready sockets, in
  their original order. `stem.0` is the new count, and elements above it are left as they were.
  When the result is 0, the stems are rewritten unchanged.

### `socket.cls`

`::requires 'socket.cls'` (it requires `rxsock` itself). It defines three public classes.

**Errors raised.** Several methods raise 93.914 with a list of the allowed values. The list in the
message is the set of allowed names joined with newlines, in hash-set iteration order, which is
unspecified on the oracle (see the 93.914 bullet under "Rulings"). In the examples below, `<list>` stands for it.

#### Class `Socket`

**Class methods**

* `gethostbyname(name)` and `gethostbyaddr(addr)` both return `.HostInfo~new(argument)`.
* `gethostid` returns `SockGetHostId()`.
* `gethostname` returns `SockGetHostName()`.
* All of them take exactly the stated arguments; others raise 93.901 or 93.902.

**`init`**

* With exactly one argument, that argument is an existing socket number and must be a positive whole
  number. Otherwise the error is 88.905 `Argument socket must be a positive whole number; found "V".`
  This includes `.Socket~new('AF_INET6')`.
* Otherwise `init(domain = 'AF_INET', type = 'SOCK_STREAM', protocol = 0)` takes at most 3
  arguments (93.902). Each argument is upper-cased and checked:
  * `domain` must be `AF_INET`.
  * `type` must be one of `SOCK_STREAM`, `SOCK_DGRAM` and `SOCK_RAW`.
  * `protocol` must be one of `0`, `IPPROTO_UDP` and `IPPROTO_TCP`; `6` is rejected.
  * A failure raises 93.914 `Method argument domain must be one of AF_INET; found "AF_INET6".`,
    with `type` and `protocol` in place of `domain` and `<list>` for the longer lists. The value
    shown is upper-cased.
  * Then `SockSocket` runs. A failure leaves the socket number −1 and is not raised.
* `errno` is the object's attribute (GET only), set by every rxsock call the object makes. A new
  object reports `0`.

**Instance methods.** `use strict` argument counts apply to all of them (93.901 and 93.902). An
explicitly omitted required argument raises 93.903 `Missing argument in method; argument 1 is
required.` (for example `setOption(, .true)`).

* `string` returns the socket number (`-1` after `close`).
* `accept()` returns a new `Socket` for the accepted connection, or `.nil` on −1. It is unguarded:
  a `close` from another activity runs while an `accept` is blocked.
* `bind(inetaddr)` and `connect(inetaddr)`: the argument must be an `InetAddress`, or the call
  raises 88.914 `Argument hostaddr must be an instance of the INETADDRESS class.` They return the
  rxsock result.
* `close()` shuts down both directions, ignoring the result, then closes. It sets the socket number
  to −1 and returns the close result, so a second `close` gives −1 with `errno` `9`.
* `getOption(name)`:
  * `name` is upper-cased and must be one of the 16 option names, or the call raises 93.914
    `Method argument 1 must be one of <list>; found "BOGUS".`
  * The result is the value from `SockGetSockOpt`, or `.nil` if the call returned −1.
  * Booleans read back as `0` and `1`, `SO_TYPE` as `STREAM`, `SO_LINGER` as `0 0`.
* `setOption(name, value)`:
  * `name` is checked as for `getOption` (93.914).
  * Unless `name` is `SO_LINGER`, `value` must be a whole number, or the call raises 93.907
    `Method argument 2 must be a positive whole number; found "V".`
  * The result is 0, or `.nil` when rxsock returned −1. `setOption('SO_TYPE', 0)` and
    `setOption('SO_ERROR', 0)` give `.nil`.
* `getPeerName()` and `getSockName()` return a new `InetAddress` built from the result, or `.nil`
  on −1. The family is converted from its number to a name: 0 `AF_UNSPEC`, 1 `AF_LOCAL`, 2
  `AF_INET`, 3 `AF_AX25`, 4 `AF_IPX`, 5 `AF_APPLETALK`, 6 `AF_NETROM`, 7 `AF_BRIDGE`, 8
  `AF_ATMPVC`, 9 `AF_X25`, 10 `AF_INET6`, 11 `AF_ROSE`, 12 `AF_DECnet`, 13 `AF_NETBEUI`, 14
  `AF_SECURITY`, 15 `AF_KEY`, 16 `AF_NETLINK`, 17 `AF_PACKET`, 18 `AF_ASH`, 19 `AF_ECONET`, 20
  `AF_ATMSVC`, 22 `AF_SNA`, 23 `AF_IRDA`, 24 `AF_PPPOX`, 25 `AF_WANPIPE`, 31 `AF_BLUETOOTH`. Other
  numbers stay numbers. Building an `InetAddress` with any family other than `AF_INET` raises
  93.914. An unbound socket's `getSockName` is `0.0.0.0` port 0 `AF_INET`.
* `ioctl(cmd, data)` returns the rxsock result, or `.nil` on −1. For `FIONREAD`, the count is not
  returned to the caller.
* `listen(backlog)`: `backlog` must be zero or a positive whole number, or the call raises 88.904
  `The backlog argument must be zero or a positive whole number; found "V".`
* `recv(len)`:
  * `len` must be a valid length, zero or more; otherwise it raises 88.911
    `Argument length is an invalid length value; found "V".`
  * Returns the data, `""` at end of file, or `.nil` on −1. `recv(0)` on an unconnected socket gives
    `.nil`.
* `recvFrom(len, inetaddr)`:
  * `len` must be a positive whole number (88.905 `Argument length must be a positive whole number`)
    and `inetaddr` an `InetAddress` (88.914 `Argument address must be an instance of the
    INETADDRESS class.`, naming `address` where `bind` and `connect` name `hostaddr`).
  * Receives up to `len` bytes, then updates `inetaddr`'s family (converted as above, which can
    raise 93.914), address and port to the sender's.
  * Returns the data, `""`, or `.nil` on −1. On −1 the address is not updated.
* `select(reads, writes, excepts, timeout)`:
  * All four arguments are required (93.901 otherwise). The three sets must be Arrays (88.914
    `Argument reads must be an instance of the Array class.`).
  * Each Array's items are the `string` values of its sockets, so they may be `Socket` objects or
    numbers.
  * Returns the `SockSelect` result, or `.nil` on −1. Otherwise each Array is emptied and refilled
    with the ready socket numbers as strings, not `Socket` objects. On timeout it is refilled with
    every original number.
* `send(data)` returns the rxsock result.
* `uninit` closes the socket unless it is already −1.

#### Class `InetAddress`

* `init(address, port [, family = 'AF_INET'])` needs at least 2 arguments (93.901). It assigns
  through the three setters in this order: address, port, family.
* The attributes `address`, `port` and `family` are public GET.
* `address=`:
  * A value other than `INADDR_ANY` (caseless) is looked up with `SockGetHostByName`. If found, the
    address becomes the host stem's `ADDR`, so `localhost` becomes `127.0.0.1`.
  * The resulting text must consist of digits and dots only, or the setter raises 93.953
    `Method argument 1 could not be converted to type dotted decimal internet address.`
  * Not found, but all digits and dots, is kept as given: `1.2.3.4.5` stays `1.2.3.4.5`.
  * `INADDR_any` is kept in the case given.
* `port=` must be a whole number from 0 to 65535, or it raises 88.907
  `Argument port must be in the range 0 to 65535; found "V".`
* `family=` is upper-cased and must be `AF_INET` (93.914 `Method argument family must be one of
  AF_INET; found "V".`).
* `makeStem` returns a stem with `!ADDR`, `!FAMILY` and `!PORT` set.

#### Class `HostInfo`

* `init(name)` needs exactly one argument and performs the lookup. `.HostInfo~new('')` raises
  93.901 `Not enough arguments for method; 1 expected.`
* The attributes `name`, `address`, `addr` (an Array) and `alias` (an Array) are public GET.
* `name` is always the text passed in. It is never replaced by the official name, so
  `.Socket~gethostbyaddr('127.0.0.1')~name` is `127.0.0.1`.
* The lookup uses `SockGetHostByAddr` if the name consists only of digits and dots, otherwise
  `SockGetHostByName`.
* `address` is the host stem's `ADDR`. **Defect:** on lookup failure, `address` is the literal text
  `HOSTINFO.!ADDR` and the Arrays are empty (D8). `addr` and `alias` copy `ADDR.1..n` and
  `ALIAS.1..n`.
* `makeStem` returns a stem with `!NAME`, `!ADDR`, `!ADDRTYPE` = `AF_INET`, `!ADDR.0..n` and
  `!ALIAS.0..n`.

### `streamsocket.cls`

`::requires 'streamsocket.cls'` (it requires `socket.cls`). Public class `StreamSocket`, a subclass
of `InputOutputStream`. A new stream's state is `READY` with description `""`. Reads arrive in
blocks of at most `bufsize` bytes (`init`'s third argument, default 4096), and lines are assembled
across reads. `string` returns `host:port`.

**Initialization**

* `init(host, port [, bufsize = 4096])` creates a new stream `Socket` that is not yet connected. If
  that socket's `errno` is not `0`, it sets state `ERROR` and raises 5.900
  `System resources exhausted: Socket <errno>`.
* `init(socket)` with exactly one argument requires a `Socket`, or it raises 93.914 `Method argument
  1 must be one of a Socket class instance; found "V".` The stream is open from the start, and host
  and port come from `getPeerName`.

**Methods**

* `open()`:
  * If the stream is already open, it sets `READY` and returns `READY:`.
  * Otherwise it connects to `.InetAddress~new(host, port)`. On failure it sets `NOTREADY` with
    description equal to the socket's `errno` and returns `NOTREADY: ECONNREFUSED` (for example).
    No condition is raised.
  * On success the stream is open and the method returns `READY:`.
* `description` returns `(state':' description)~strip`.
* `state` returns the state.
* `string` returns `host:port`, for example `127.0.0.1:18926`.
* `chars` and `lines` return 1 if open and 0 otherwise.
* `close()` closes the socket if open. If that returns −1 it sets `ERROR` and that description,
  without raising. Otherwise it sets `READY`. It returns `description`. The stream is not marked
  closed.
* `lineOut([str = '' [, line]])` sends `str` followed by CRLF and returns the byte count. `line` is
  ignored.
* `say(str)` does the same and returns nothing.
* `charOut(str [, start])` sends `str` without CRLF and returns the count.
* `lineIn([line [, count = 1]])`:
  * `count = 0` returns `""`. `count > 1` raises 93.908 `Method argument 2 must not exceed 1; found
    "&3".`
  * If the state is `NOTREADY` it raises NOTREADY and returns `""`.
  * Otherwise it reads until a CRLF has arrived; a failed read counts as end of file. At end of
    file, buffered text without CRLF is returned as the last line; the next `lineIn` sets
    `NOTREADY` (description `NOTREADY: 0`), raises NOTREADY and returns `""`.
  * It returns the text before the first CRLF; the rest stays for later reads. A bare LF is not a
    line end.
* `charIn([start [, length = 1]])`:
  * A negative length raises 93.906 `Method argument 2 must be zero or a positive whole number;
    found "V".`
  * It reads until the buffer holds `length` bytes, with the same NOTREADY rules as `lineIn`, then
    returns and removes them. `charIn(, 0)` returns `""`.
* `arrayIn`, `arrayOut` and `position` raise 93.963 `Call to unsupported or unimplemented method.`
* Any of `lineIn`, `charIn`, `lineOut` and `charOut` on a stream that is not open first calls
  `open`. If that does not return exactly `READY:`, `lineIn` and `charIn` return `""` and `lineOut`
  and `charOut` return 0, all without raising.

**Defect:** when the send in `lineOut` or `charOut` fails, the method raises
`Error 97.1: Object "S" does not understand message "ERRNO".` instead of NOTREADY, and the stream's
state is unchanged (D10).

### `mime.cls`

`::requires 'mime.cls'`. No dependencies. CRLF below means `'0D0A'x`. The constant `crlf` is public
on both classes.

#### Class `mimepart`

* `init([type = 'text/plain'])`. The attributes `type`, `encoding`, `id`, `description` and
  `disposition` are public GET and SET, and all but `type` start as `""`. `content` is GET only and
  starts as `""`.
* `addContent(line)` appends `line` CRLF to the content. A line that is exactly `.` is stored as
  `..`; `..` is stored unchanged.
* `string` returns the following, byte for byte. Each header line appears only when its value is not
  `""`, including `Content-Type`:

```
MIME-Version: 1.0 CRLF
Content-Type: <type> CRLF
Content-Transfer-Encoding: <encoding> CRLF
Content-ID: <id> CRLF
Content-Description: <description> CRLF
Content-Disposition: <disposition> CRLF
CRLF
<content> CRLF
```

The default part's string is therefore
`MIME-Version: 1.0␍␊Content-Type: text/plain␍␊␍␊␍␊`.

#### Class `mimemultipart`

* `init()` takes no arguments. The attributes are as for `mimepart`, with `type` starting as
  `multipart/mixed`. There is no `content` attribute.
* `addPart(part)` appends a `mimepart`. Anything else raises 93.948 `Method argument MIMEPART must be
  of the &2 class.`
* `string` builds the boundary `===ooRexx=Mime=Part===` || `c2x(date('B') || time('L'))` || `==`,
  for example `===ooRexx=Mime=Part===37333939323230323A32353A35372E313739373733==`. It returns:

```
MIME-Version: 1.0 CRLF
Content-Type: <type>; boundary="<boundary>" CRLF
(Content-Transfer-Encoding, Content-ID, Content-Description and Content-Disposition lines as for mimepart)
CRLF
```

  If there is at least one part, the string continues with, for each part in order,
  `--<boundary> CRLF <part~string> CRLF`, and ends with `--<boundary>-- CRLF`.

### `smtp.cls`

`::requires 'smtp.cls'` (it requires `mime.cls` and `streamsocket.cls`).

#### Class `smtpmsg`

* `init()`. The attributes `from` and `subject` are public GET and SET and start as `""`.
* `content` (GET) returns the stored content's `string`.
* `content=` accepts a `mimepart`, `mimemultipart` or String. Anything else raises 93.948
  `Method argument MIMEPART must be of the MIMEMULTIPART class.`
* `addRecipient(r)` appends `r`. `recipients` returns the Array of recipients.

#### Class `smtp`

* `init()` takes no arguments and resets the following defaults. `logoff` resets them again.
  * `authid`, `password`, `response` = `""`
  * `cmdresponse` = new Array
  * `debug` = `.false`
  * remote host `""`, port 25
  * `localhost` = `.Socket~gethostbyaddr('127.0.0.l')`. The last character is the letter `l`, so this
    is a DNS name lookup of `127.0.0.l`. It normally fails, giving a `HostInfo` whose `name` is
    `127.0.0.l`. See D8.
* `smtperrno` is not reset. Before the first command it is the unset variable's name, `SMTPERRNO`.
* Public attributes: `authid`, `cmdresponse`, `debug`, `password`, `response` and `smtperrno` (GET
  and SET), and `localhost` (GET). `localhost=` accepts a `HostInfo`, after which `localhost`
  returns its `name`, or a string. `connect` then fails: after `localhost = 'myhost'`, `connect`
  raises 97.1 `Object "myhost" does not understand message "NAME"`.
* Debug output is printed with `SAY` when `debug \= .false`.

**A transaction.** A transaction sends one command and checks its reply:

1. Append the command text to `cmdresponse`, then send it with `lineOut`.
2. Read a reply line with `lineIn` and append it. While the 4th character of the latest line is
   `-`, read and append another line.
3. `smtperrno` becomes the first 3 characters of the last line.
4. A reply whose first digit differs from the expected digit (2 for EHLO, MAIL, RCPT, password; 3
   for AUTH LOGIN, authid, DATA) fails the step. In debug mode each reply line of a failed step is
   printed as `Error: unexpected response: <line>`.

**`connect(host [, authid [, password]])`**

1. `host` may be `name:port`. A port that is not a whole number returns −1.
2. Open a `StreamSocket` to host and port. A failed open does not stop `connect`:
   `connect('127.0.0.1:1')` (refused) returns −1 with `cmdresponse` = `''`, `'EHLO 127.0.0.l'`,
   `''` and `smtperrno` = three blanks (the empty reply's first three characters, blank-padded).
3. Read the greeting, which may span several lines, into `cmdresponse`.
4. Run the transaction `EHLO <localhost~name>` expecting `2`. On failure return −1.
5. Scan `cmdresponse` from the start for the first line whose characters 5 to 8 are `AUTH`, and take
   the text from character 5 as the authorization types.
6. If those contain `LOGIN`, log in and return its result. Otherwise return 0.

To log in, run three transactions: `AUTH LOGIN` expecting `3`, then base64(authid) expecting `3`,
then base64(password) expecting `2`. Each failure returns −1, after a debug line naming the failed
step and the host.

**`send(msg)`**

`msg` must be an `smtpmsg`, or the method raises 93.948 with `(1, 'SMTPMsg')`. It sets `response` to
a new empty Array, then runs these transactions in order, returning −1 on any failure:

1. `MAIL FROM:<from>` (no blank after the colon) expecting `2`.
2. For each recipient, `RCPT TO: <rcpt>` (one blank) expecting `2`.
3. `DATA` expecting `3`.

It then sends these lines without reading replies:

1. `From: <from>`
2. `To: <r1>`, followed by `,` if more recipients follow
3. for each further recipient, four blanks then `<rn>`, with `,` on all but the last
4. `Subject: <subject>`
5. the content string followed by CRLF
6. `.`

Finally it reads the reply (possibly several lines). It returns 0 if the last line starts with `2`.
Otherwise it prints a debug message and returns −1.

The complete client byte stream observed against a scripted server, CRLF shown as `<CRLF>`:

```
EHLO 127.0.0.l<CRLF>
AUTH LOGIN<CRLF>
dXNlckB4LmNvbQ==<CRLF>
c2VjcmV0<CRLF>
MAIL FROM:<a@x.com><CRLF>
RCPT TO: <b@y.com><CRLF>
RCPT TO: <c@z.com><CRLF>
DATA<CRLF>
From: <a@x.com><CRLF>
To: <b@y.com>,<CRLF>
    <c@z.com><CRLF>
Subject: Hello there<CRLF>
MIME-Version: 1.0<CRLF>
Content-Type: text/plain<CRLF>
<CRLF>
Body line<CRLF>
..<CRLF>
<CRLF>
<CRLF>
.<CRLF>
QUIT<CRLF>
```

After that session, `cmdresponse` held, in order:

* the greeting
* `EHLO 127.0.0.l` and its three reply lines
* `AUTH LOGIN`, `334 …`, the base64 authid, `334 …`, the base64 password, `235 ok`
* `MAIL FROM:<a@x.com>`, `250 ok`
* each `RCPT TO:` line and its `250 ok`
* `DATA`, `354 go`
* `250 queued`

`smtperrno` was `354`, because the reply to the message body is not stored there.

**`logoff()`** sends `QUIT`, closes the stream without reading the reply, resets the defaults and
returns 0.

## Resolutions

1. **rxregexp language versus grammar.** The grammar above is the intended language. The oracle
   deviates in two pattern families, D4 and D5. Their exact oracle languages are confirmed by a
   differential run of 37 patterns against every string over `abcd` up to length 5, compared with
   an independent reference. All other tested patterns agree with the grammar for `match`,
   MINIMAL `match` positions, and `pos` with its positions. The decision is the regexp-defects
   bullet under "Rulings".
2. **`RegularExpression~new` without a pattern.** The source leaves the compiled pattern undefined,
   and the oracle crashes on `match` or `pos` (D1). `position` is `!POS`. The closest defined
   behaviour is the post-error state: match everything, position 0.
3. **rxmath "Argument 2" comments.** The test group's comments say units are "Argument 2". The
   oracle and source say `Argument 3`. The tests check only the code, 88.916.
4. **SysListxattr failure value.** A test comment says it "returns -1". The oracle returns `""`.
5. **SysGettid.** The documented meaning is "numerical thread id". The oracle returns the
   `pthread_self()` value, an address. Tests check only `datatype` NUM.

## Oracle defects

| Id | Defect | Minimal reproduction | Effect |
|---|---|---|---|
| D1 | match on an object with no pattern | `.RegularExpression~new~match('x')` | SIGSEGV, every run |
| D2 | set range ending at `'7F'x` | `p~parse('['||'01'x||'-'||'7F'x||']')` | never returns, memory grows |
| D3 | use after explicit `uninit` | `p = .RegularExpression~new('a'); p~uninit; p~match('a')` | SIGSEGV |
| D4 | alternative starting with a starred factor | `a*b\|c` matches `ac`; `a*\|b` matches `ab`; `[ab]*\|c` matches `ac`; `d(a*b\|c)` matches `dac` | wrong result. Observed language: `X*R\|Z` behaves as `X*(R\|Z)`. |
| D5 | `X+` immediately followed by a starred factor | `a+c*` fails `aa`; `(a\|b)+c*` fails `aa`; `(ab)+c*` fails `abab` | wrong result. Observed language: `X+Y*` behaves as `XY*`. |
| D6 | unknown group | `SysGetgrnam('nosuchgroup','n')`, `SysGetgrgid(99999,'n')` | SIGSEGV |
| D7 | non-numeric socket in a select stem | `w.0=2; w.1=1; w.2='z'; call SockSelect ,'w.',,0` | SIGSEGV |
| D8 | failed lookups leak unset compound names | `.HostInfo~new('no.such.invalid')~address` gives `HOSTINFO.!ADDR`. `.smtp~new` does a DNS lookup of `127.0.0.l`. | wrong value |
| D9 | `SockGetHostId` skips the last interface, and the result is garbage if none qualifies | inspection | wrong or unspecified value |
| D10 | `streamsocket.cls` send-error path | a `lineOut` whose send fails | error 97.1 instead of NOTREADY |
| D11 | repetition of a sub-expression that can match empty | `(a*)*`, `(a*)+`, `(a*b*)*`, `((a)*)*` with any `match` or `pos` (parse succeeds) | never returns, memory grows until killed |

Two related observations are not defects of these libraries. `SockSock_Errno` and `SysGeterrno`
report errno values that the interpreter's own console I/O produced. A blocking test program whose
server activity is still in `recv` keeps the process alive after the main program ends.

## Rulings

Moritz ruled four questions on 2026-11-03. The controller settled the rest; those calls are marked.

* **Regexp defects D2, D4, D5 and D11 (Moritz: grammar, Deviation 87).** Implement the pattern
  language as this document states it, with standard meaning for alternation and repetition, and
  always terminate. `a*b|c` does not match `ac`; `a+c*` matches `aa`; `(a*)*` and a range ending at
  `'7F'x` answer in bounded time and memory.
* **Crash defects D1, D3, D6 and D7 (Moritz: raise a Rexx error, Deviation 88).** Each raises a
  defined condition instead of crashing or inventing a result. A RegularExpression method with no
  compiled pattern, or after `uninit`, raises 93.900 with a message naming the cause. SysGetgrnam or
  SysGetgrgid for an unknown group, and SockSelect with a non-numeric stem entry, raise 40.1 for the
  routine, as any other routine failure in this document.
* **Line numbers in the rewritten packages (Moritz: accept, Deviation 89).** Tracebacks through the
  rewritten `.cls` files show their own line numbers. Messages, codes and results match the oracle.
  The packages are found the way this crate finds `::requires` files; where they are placed is set
  in the PR 4 plan.
* **OS access (Moritz: a safe crate).** Lookups and calls that `rustix` does not cover (users and
  groups, host and address resolution, interface lists, ioctl and select on sockets) go through
  established safe crates such as `nix`, `dns-lookup` and `if-addrs`. No new `unsafe` in this
  repository.
* **SysCrypt (controller).** Not provided, as on the oracle; `test_SysCrypt` fails on both, and its
  row is settled with the other host-dependent SysUnix rows in PR 6.
* **errno (controller).** `SysGeterrno` and `SockSock_Errno` report the errno of the last call of
  their own library on this thread, or 0 if none. The interpreter's own I/O does not change it.
* **93.914 option lists (controller).** Fixed order: the order the options are listed in this
  document. Hash order is unspecified (Moritz, 2026-10-12).
* **D8, D9 and D10 (controller).** Reproduce the oracle where it is deterministic: the
  `HOSTINFO.!ADDR` value after a failed lookup, `SockGetHostId` skipping the last interface, and
  `Error 97.1: Object "S" does not understand message "ERRNO".` on a failed `streamsocket.cls`
  send, with the stream state unchanged. When no interface qualifies for `SockGetHostId`, raise
  40.1, Deviation 88's rule for an undefined result.
