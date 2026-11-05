/* A real shared library, not LIBRARY REXX. The two forms share one row --
   a row is (directive, keyword, position) -- and only this one has a probe,
   so the LIBRARY REXX form has no row of its own here. */

say 'main'
::routine r external 'LIBRARY zzznolib zzzr'
