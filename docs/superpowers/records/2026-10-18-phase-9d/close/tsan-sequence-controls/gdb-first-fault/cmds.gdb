set pagination off
set confirm off
handle SIG32 SIG33 SIG34 SIG35 SIG36 SIG37 SIGPIPE nostop noprint pass
handle SIGSEGV stop print nopass
run
echo === siginfo\n
p $_siginfo._sifields._sigfault.si_addr
echo === regs\n
info registers rip rsp rbp rax rcx rdx rdi rsi
x/8i $pc-16
echo === thread\n
info threads
echo === bt\n
bt 40
echo === mappings\n
info proc mappings
kill
quit
