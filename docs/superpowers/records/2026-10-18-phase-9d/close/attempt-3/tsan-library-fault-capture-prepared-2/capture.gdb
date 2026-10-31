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
# The interpreter's native-call interrupt is SIGRTMIN + 3, observed as SIG37.
# Pass the Linux realtime signal range, including the threading signals.
handle SIG32 SIG33 SIG34 SIG35 SIG36 SIG37 SIG38 SIG39 SIG40 SIG41 SIG42 SIG43 SIG44 SIG45 SIG46 SIG47 SIG48 SIG49 SIG50 SIG51 SIG52 SIG53 SIG54 SIG55 SIG56 SIG57 SIG58 SIG59 SIG60 SIG61 SIG62 SIG63 SIG64 nostop noprint pass
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
inferior_pid = gdb.selected_inferior().pid
fault = observed_stop_signal == 'SIGSEGV'
unexpected = bool(inferior_pid) and not fault
result = {'stop_signal': observed_stop_signal, 'inferior_pid': inferior_pid,
          'fault_intercepted_by_debugger': fault,
          'unexpected_diagnostic_stop': unexpected}
print('DIAGNOSTIC_INFERIOR_STATUS=' + json.dumps(result))
if unexpected:
    print('DIAGNOSTIC_UNEXPECTED_STOP=' + json.dumps(result))
    print('DIAGNOSTIC_CLEANUP=kill inferior after unexpected stop, no fault inspection')
    gdb.execute('kill')
    gdb.execute('quit 86')
elif fault:
    commands = ['info program', 'info registers', 'thread apply all bt 24',
                'info proc mappings', 'x/8i $pc', 'set language c',
                'set $diagnostic_tsan_state = *(unsigned long long*)($fs_base - 0x958)',
                'printf "DIAGNOSTIC_TSAN_STATE=0x%llx\\n", $diagnostic_tsan_state',
                'x/gx $diagnostic_tsan_state + 0x10',
                'x/2gx $diagnostic_tsan_state + 0x2d8',
                'printf "DIAGNOSTIC_SHADOW_BEGIN=0x%llx\\n", *(unsigned long long*)($diagnostic_tsan_state + 0x2d8)',
                'printf "DIAGNOSTIC_SHADOW_TOP=0x%llx\\n", *(unsigned long long*)($diagnostic_tsan_state + 0x10)',
                'printf "DIAGNOSTIC_SHADOW_END=0x%llx\\n", *(unsigned long long*)($diagnostic_tsan_state + 0x2e0)',
                'printf "DIAGNOSTIC_SHADOW_BYTES=%llu\\n", *(unsigned long long*)($diagnostic_tsan_state + 0x10) - *(unsigned long long*)($diagnostic_tsan_state + 0x2d8)',
                'printf "DIAGNOSTIC_SHADOW_ENTRIES=%llu\\n", (*(unsigned long long*)($diagnostic_tsan_state + 0x10) - *(unsigned long long*)($diagnostic_tsan_state + 0x2d8)) / 8']
    for command in commands:
        print('DIAGNOSTIC_READ_COMMAND=' + command)
        try:
            gdb.execute(command)
        except gdb.error as error:
            print('DIAGNOSTIC_READ_ERROR=' + str(error))
    print('DIAGNOSTIC_CLEANUP=kill stopped inferior after read-only capture')
    gdb.execute('kill')
else:
    print('DIAGNOSTIC_NO_FAULT=inferior exited without a captured SIGSEGV')
end
