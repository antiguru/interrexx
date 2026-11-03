/* rxregexp.cls: the RegularExpression class over the rxregexp library's
   native methods, written for this interpreter from
   docs/superpowers/specs/extensions-behaviour.md ("rxregexp").

   init, uninit, parse, match and pos are the library's. Each of parse, match
   and pos leaves the position it computed in the object variable !POS, which
   position answers; until one of them has run, !POS is unassigned and
   position answers its name. */

::class "RegularExpression" public

::method init external "LIBRARY rxregexp RegExp_Init"
::method uninit external "LIBRARY rxregexp RegExp_Uninit"
::method parse external "LIBRARY rxregexp RegExp_Parse"
::method match external "LIBRARY rxregexp RegExp_Match"
::method pos external "LIBRARY rxregexp RegExp_Pos"

::method position
  expose !POS
  use strict arg
  return !POS
