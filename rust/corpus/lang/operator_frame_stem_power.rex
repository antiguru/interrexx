/* The same forwarded frame as operator_frame_stem_plus.rex,
 * with the operator's shape varied to `**` so the frame's method name is
 * read off the failing operator rather than assumed to always be "+".
 * Measured, rc 215, stderr opening
 * the frame of method "**" with scope String (101.20).
 */

say b. ** 2
