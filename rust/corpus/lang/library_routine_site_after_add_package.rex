/* A call site that found a library routine finds a public routine of the
   same name that addPackage made callable in front of it, on its next pass. A
   routine of the REXX package is
   library_routine_site_after_add_package_filespec's: FILESPEC is one only at
   language level 6.06 (Deviation 179). */
x = .context~package~loadLibrary('rxmathc')
do i = 1 to 2
  say 'library' i RxCalcSqrt(16)
  if i = 1 then .context~package~addPackage(.context~package~loadPackage('sqrt.cls'))
end
