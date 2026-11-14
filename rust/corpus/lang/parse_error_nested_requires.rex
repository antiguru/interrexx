/* A ::REQUIRES file that does not parse, or whose prologue raises, inside
   a file Routine~newFile, Method~newFile, Package~new or loadPackage loads:
   the directive's level and the loading method's level both report. A
   built-in level is not printed: whether a build has one is not specified
   (r[class.stackframe.builtin]), and its trace line has no line number. */
do kind over .array~of('parse', 'run')
  call try .routine, 'newFile', 'mid_rnf_'kind'.cls'
  call try .method, 'newFile', 'mid_mnf_'kind'.cls'
  call try .package, 'new', 'mid_pkg_'kind'.cls'
  call try .context~package, 'loadPackage', 'mid_lp_'kind'.cls'
end
exit
try: signal on syntax
  r = arg(1)~send(arg(2), arg(3))
  say 'not reached'
  return
syntax:
  o = condition('O')
  say o~code o~position rc
  do l over o~traceback
    if word(l, 1) \== '*-*' then say '  tb:' l
  end
  do f over o~stackframes
    if word(f~traceLine, 1) \== '*-*' then say '  sf:' f~type f~name f~line
  end
  return
