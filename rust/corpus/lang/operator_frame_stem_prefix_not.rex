/* The same forwarded frame reached through prefix
 * `\`, whose own check is textual rather than numeric -- `s.`'s default
 * "abc" is not a logical value. Measured, rc 222, stderr opening
 * the frame of method "\" with scope String (101.20).
 */

s. = 'abc'
say \s.
