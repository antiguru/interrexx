/* The code of a directive error when the source reaches the parser through
   .Package~new, beside controls that raise none. */
call show '::method'
call show '::routine'
call show '::requires'
call show '::method m external nostring'
call show '::attribute a external nostring'
call show '::routine r external nostring'
call show '::class c metaclass'
call show '::class c subclass +'
call show '::class c inherit'
call show '::class c inherit a +'
call show '::class c mixinclass'
call show '::attribute'
call show '::constant'
call show '::constant c *'
call show '::constant c -abc'
call show '::constant c -5x'
call show '::options digits -1'
call show '::options fuzz -1'
call show '::options trace +'
call show '::resource'
call show '::resource r end'
call show '::annotate'
call show '::annotate "package"'
call show '::options form'
call show '::options form "x"'
call show '::method m delegate "p"'
call show '::attribute a delegate'
call show '::options numeric'
call show '::options numeric 5'
call show '::annotate package a *'
call show '::annotate package a -abc'
call show '::annotate package a -5x'
call show '::class'
call show '::class c subclass rexx:'
call show '::requires x namespace "n"'
call show '::junk'
call show '::annotate package a'
call show '::requires x namespace rexx'
call show '::class c junk'
exit

show: procedure
  parse arg src
  signal on syntax
  p = .package~new('x', .array~of(src))
  say src '-> no error'
  return
syntax:
  c = condition('o')
  say src '->' c~code
  return
