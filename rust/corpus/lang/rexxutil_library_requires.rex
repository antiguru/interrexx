/* ::REQUIRES of the internal rexxutil and REXX packages as libraries: their
   routines join this package's imported routines, are found there before the
   security manager is asked. The name a call wrote shows only in the trace
   line of a built-in level, which is not specified
   (r[class.stackframe.builtin]), so a failing call prints its code only. */
p = .context~package
say 'call' SysSleep(0)
ir = p~importedRoutines
say 'imported' ir~items
say 'has' ir~hasIndex('SYSSLEEP') ir~hasIndex('SysSleep') ir~hasIndex('SYSADDREXXMACRO') ir~hasIndex('FILESPEC')
say 'imported packages' p~importedPackages~items
say 'routines' p~routines~items 'public' p~publicRoutines~items
r = p~findRoutine('SysSleep')
say 'found' (r \== .nil) 'package' r~package~name
say 'stable' (p~findRoutine('syssleep') == r) (ir['SYSSLEEP'] == r)
say 'public' (p~findPublicRoutine('SysSleep') == r)
say 'routine call' r~call(0)
say 'version' p~findRoutine('SysVersion')~call~word(1)
say 'filespec' p~findRoutine('Filespec')~call('n', '/a/b.c')
say 'loadLibrary' p~loadLibrary('rexxutil') p~loadLibrary('REXX') p~loadLibrary('REXXUTIL')

say '-- the manager is not asked for an imported routine'
p~setSecurityManager(.collector~new)
call managed

say '-- a failing call, written as a string and as a symbol'
call quoted
call symbol
exit

quoted:
  signal on syntax name blamed
  call 'SysSleep' 'abc'
  return
symbol:
  signal on syntax name blamed
  call SysSleep 'abc'
  return
blamed:
  c = condition('o')
  say c~code
  return

::routine managed
say 'managed' SysSleep(0) SysVersion()~word(1)

::class collector
::method unknown
  use arg name, args
  say 'event' name args[1]~name
  return 0

::requires 'rexxutil' library
::requires 'REXX' library
