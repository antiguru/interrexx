import hashlib,json,os,pathlib,resource,subprocess,time
root=pathlib.Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
base=root/'docs/superpowers/records/2026-10-18-phase-9d/close/tsan-limits-diagnosis'
binary=root/'.superpowers/sdd/2026-10-18-phase-9d/task10-targets/9922e56f3bf6c72442d796f8a43f631a86621abd/tsan/x86_64-unknown-linux-gnu/debug/build/rexx-exec/6c80e238cfcc855d/out/rexx_exec-6c80e238cfcc855d'
names=['eval::tests::a_parse_pattern_and_an_expression_share_the_eval_depth_limit','input::tests::spawn_failure_on_stdin_reaches_syntax_without_reading','input::tests::spawn_failure_preserves_partial_input_and_requeues_other_posts','input::tests::stdin_recovers_with_a_pooled_worker_after_spawn_failure','input::tests::stdin_recovers_in_configured_inline_mode_after_spawn_failure']
provenance={'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'binary':str(binary),'binary_sha256':hashlib.file_digest(binary.open('rb'),'sha256').hexdigest(),'rlimit_as':resource.getrlimit(resource.RLIMIT_AS),'rlimit_stack':resource.getrlimit(resource.RLIMIT_STACK),'cgroup':pathlib.Path('/proc/self/cgroup').read_text(),'start':time.time(),'cwd':str(root/'rust')}
(base/'provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')
for number,name in enumerate(names,1):
    case=base/f'{number:02d}-{name.split("::")[-1]}'
    case.mkdir()
    env=os.environ.copy()
    options=f'log_path={case}/tsan suppressions={root}/rust/tsan.supp allocator_may_return_null=1 second_deadlock_stack=1'
    env['TSAN_OPTIONS']=options
    command=['timeout','--kill-after=5s','90s',str(binary),'--exact',name,'--nocapture']
    record={'command':command,'TSAN_OPTIONS':options,'start':time.time(),'timeout_seconds':90}
    (case/'command.json').write_text(json.dumps(record,indent=2)+'\n')
    with (case/'stdout').open('xb') as stdout,(case/'stderr').open('xb') as stderr:
        result=subprocess.run(command,cwd=root/'rust',env=env,stdout=stdout,stderr=stderr)
    record.update(status=result.returncode,end=time.time(),reports=[p.name for p in case.glob('tsan.*')])
    (case/'status.json').write_text(json.dumps(record,indent=2)+'\n')
    print(name,result.returncode,record['reports'],flush=True)
