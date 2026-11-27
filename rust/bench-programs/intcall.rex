/* Internal package routines called n times each: SysUtilVersion and SysFileExists
   of rexxutil, which run no activation. */
n = 150000
t = 0
do i = 1 to n
    t = t + length(SysUtilVersion()) + SysFileExists('/nonexistent/liba.so')
end
say t
