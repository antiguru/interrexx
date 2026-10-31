set pagination off
set confirm off
set debuginfod enabled off
set disable-randomization off
handle SIGINT nostop noprint pass
handle SIGTERM nostop noprint pass
handle SIGHUP nostop noprint pass
handle SIGPIPE nostop noprint pass
handle SIGALRM nostop noprint pass
handle SIGUSR1 nostop noprint pass
handle SIGUSR2 nostop noprint pass
handle SIGCHLD nostop noprint pass
handle SIGSEGV stop print nopass
python
import json
observed_stop_signal = None
def record_diagnostic_stop(event):
    global observed_stop_signal
    observed_stop_signal = getattr(event, 'stop_signal', None)
gdb.events.stop.connect(record_diagnostic_stop)
end
run
python
result = {'stop_signal': observed_stop_signal, 'inferior_pid': gdb.selected_inferior().pid,
          'fault_intercepted_by_debugger': observed_stop_signal == 'SIGSEGV'}
print('DIAGNOSTIC_INFERIOR_STATUS=' + json.dumps(result))
commands = ['info program']
if gdb.selected_inferior().pid:
    commands += ['info registers', 'thread apply all bt 24', 'info proc mappings',
                 'x/8i $pc', 'set language c',
                 'set $diagnostic_tsan_state = *(unsigned long long*)($fs_base - 0x958)',
                 'printf "DIAGNOSTIC_TSAN_STATE=0x%llx\n", $diagnostic_tsan_state',
                 'x/gx $diagnostic_tsan_state + 0x10',
                 'x/2gx $diagnostic_tsan_state + 0x2d8',
                 'printf "DIAGNOSTIC_SHADOW_BEGIN=0x%llx\\n", *(unsigned long long*)($diagnostic_tsan_state + 0x2d8)',
                 'printf "DIAGNOSTIC_SHADOW_TOP=0x%llx\\n", *(unsigned long long*)($diagnostic_tsan_state + 0x10)',
                 'printf "DIAGNOSTIC_SHADOW_END=0x%llx\\n", *(unsigned long long*)($diagnostic_tsan_state + 0x2e0)',
                 'printf "DIAGNOSTIC_SHADOW_BYTES=%llu\n", *(unsigned long long*)($diagnostic_tsan_state + 0x10) - *(unsigned long long*)($diagnostic_tsan_state + 0x2d8)',
                 'printf "DIAGNOSTIC_SHADOW_ENTRIES=%llu\n", (*(unsigned long long*)($diagnostic_tsan_state + 0x10) - *(unsigned long long*)($diagnostic_tsan_state + 0x2d8)) / 8']
for command in commands:
    print('DIAGNOSTIC_READ_COMMAND=' + command)
    try:
        gdb.execute(command)
    except gdb.error as error:
        print('DIAGNOSTIC_READ_ERROR=' + str(error))
if gdb.selected_inferior().pid:
    print('DIAGNOSTIC_CLEANUP=kill stopped inferior after read-only capture')
    gdb.execute('kill')
end
