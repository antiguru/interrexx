/* A ::CONSTANT directive's parenthesised expression evaluates at install
 * time and a well-formed one changes nothing observable.
 *
 * Were the expression form accepted and never evaluated, a well-formed
 * expression and an ignored
 * one would produce the same bytes -- this program alone could not have told the
 * two apart, which is exactly why `directive_constant_expression_fails.rex`
 * beside it exists. Measured, rc 0, stdout "prolog" on both sides; the
 * constant's own value is discarded here, since nothing in this crate can
 * read it back without a message send.

 */

say 'prolog'
exit

::class K
::constant c (2+3)
