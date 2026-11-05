/* A message send that is a whole clause, and what it does that the
 * expression form does not: it settles RESULT.
 *
 * The ~~ form replaces the send's result with the target BEFORE RESULT is
 * set, so the second RESULT read is the receiver and not the length --
 * measured, "abc" rather than 3.
 */

'abc'~length
say result
'abc'~~length
say result
