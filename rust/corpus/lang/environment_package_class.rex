/* A class this file declares resolves by name from inside this file, with no
 * PUBLIC keyword on the directive.
 *
 * Class lookup consults the installed classes first, which are the
 * package's own class table and is not the environment: nothing installs a
 * ::CLASS into .environment. The embedded core library is this case, naming
 * classes its own file declares further down.
 *
 * An unquoted directive name is a symbol, which the scanner upcases, so the
 * class id is ZORK and defaultName renders "The ZORK class". A quoted one
 * keeps its spelling, which is what the second class here shows.
 *
 * Both routes reach the same table.
 *
 * Measured, rc 0.
 */

say .Zork
say value('.zork')
say .Quux

::class Zork
::class 'Quux'
