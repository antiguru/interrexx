set pagination off
set confirm off
handle SIG32 SIG33 SIG34 SIG35 SIG36 SIG37 SIGPIPE nostop noprint pass
tbreak rexx_exec::dispatch::hash::stem::forward_to_value
run
set language c
echo === first forward\n
thread
eval "tbreak *0x555555930ee2 thread %d", $_thread
continue
echo === push site\n
p/x $rsi
p/x $rax
python
import gdb,struct
pos=int(gdb.parse_and_eval('(unsigned long)$rax'))
inf=gdb.selected_inferior()
a=pos-8
while True:
    v=struct.unpack('<Q',inf.read_memory(a,8).tobytes())[0]
    if not 0x555555554000<=v<0x555558000000: break
    a-=8
base=a+8
e=base+0x80000
print('stop value',hex(v),'at',hex(a),'base',hex(base),'used entries',(pos-base)//8,'end',hex(e))
gdb.set_convenience_variable('end', e)
end
p/x $end
echo === content at end before\n
x/4gx $end
watch -l *(long*)$end
continue
echo === watch hit\n
thread
p/x $rax
p/x $rsp
info registers rip
x/2i $pc
bt 6
info proc mappings
delete
continue
echo === after\n
info program
quit
