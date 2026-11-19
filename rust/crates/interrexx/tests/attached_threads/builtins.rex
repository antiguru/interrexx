/* A wait on an event semaphore that nothing posts, in a program that loaded only libraries built
   into the interpreter: RexxUtil through RxFuncAdd, and rxmath. The wait is refused.
   spec: conc.activities.deviation.20 */
call RxFuncAdd 'SysLoadFuncs', 'rexxutil', 'SysLoadFuncs'
call SysLoadFuncs
say RxCalcSqrt(4)
h = SysCreateEventSem()
say 'waiting'
say SysWaitEventSem(h)
say 'never'
::requires 'rxmath' LIBRARY
