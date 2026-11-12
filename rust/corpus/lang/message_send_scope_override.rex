/* target~name:scope refuses a scope that is not a class object, before the
 * arguments are evaluated.
 *
 * SUPER outside a method is an ordinary uninitialised variable, so the scope
 * expression comes to the string "SUPER" and the isInstanceOf check in
 * the message expression refuses it.
 * Measured, rc 168.
 */

say 'abc'~length:super
