import json,os,pathlib,subprocess,time
root=pathlib.Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
base=root/'docs/superpowers/records/2026-10-18-phase-9d/close/tsan-limits-diagnosis'
binary=json.loads((base/'provenance.json').read_text())['binary']
names=['spawn_failure_on_stdin_reaches_syntax_without_reading','spawn_failure_preserves_partial_input_and_requeues_other_posts','stdin_recovers_with_a_pooled_worker_after_spawn_failure','stdin_recovers_in_configured_inline_mode_after_spawn_failure']
for short in names:
    name='input::tests::'+short
    case=base/('direct-child-'+short)
    case.mkdir()
    options=f'log_path={case}/tsan suppressions={root}/rust/tsan.supp allocator_may_return_null=1 second_deadlock_stack=1'
    command=['timeout','--kill-after=5s','35s',binary,'--exact',name,'--nocapture']
    env=dict(os.environ,TSAN_OPTIONS=options,REXX_POOL_FAILURE_CHILD=name)
    record={'command':command,'TSAN_OPTIONS':options,'REXX_POOL_FAILURE_CHILD':name,'start':time.time(),'stdin':'success\\n on open pipe for recovery cases, empty open pipe otherwise','scope':'Execute existing child preflight and complete test body without Python launcher RLIMIT_AS'}
    (case/'command.json').write_text(json.dumps(record,indent=2)+'\n')
    with (case/'stdout').open('xb') as stdout,(case/'stderr').open('xb') as stderr:
        process=subprocess.Popen(command,cwd=root/'rust',env=env,stdout=stdout,stderr=stderr,stdin=subprocess.PIPE)
        if short.startswith('stdin_recovers'):
            process.stdin.write(b'success\n')
            process.stdin.flush()
        status=process.wait()
        process.stdin.close()
    record.update(status=status,end=time.time(),reports=[p.name for p in case.glob('tsan.*')])
    (case/'status.json').write_text(json.dumps(record,indent=2)+'\n')
    print(short,status,record['reports'],flush=True)
