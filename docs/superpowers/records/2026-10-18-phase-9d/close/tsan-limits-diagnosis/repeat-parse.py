import json,os,pathlib,subprocess,time
root=pathlib.Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
base=root/'docs/superpowers/records/2026-10-18-phase-9d/close/tsan-limits-diagnosis'
binary=json.loads((base/'provenance.json').read_text())['binary']
for index in range(1,4):
    case=base/f'parse-repeat-{index}'
    case.mkdir()
    options=f'log_path={case}/tsan suppressions={root}/rust/tsan.supp allocator_may_return_null=1 second_deadlock_stack=1'
    command=['timeout','--kill-after=5s','90s',binary,'--exact','eval::tests::a_parse_pattern_and_an_expression_share_the_eval_depth_limit','--nocapture']
    env=dict(os.environ,TSAN_OPTIONS=options)
    record={'command':command,'TSAN_OPTIONS':options,'start':time.time()}
    (case/'command.json').write_text(json.dumps(record,indent=2)+'\n')
    with (case/'stdout').open('xb') as stdout,(case/'stderr').open('xb') as stderr:
        result=subprocess.run(command,cwd=root/'rust',env=env,stdout=stdout,stderr=stderr)
    record.update(status=result.returncode,end=time.time(),reports=[p.name for p in case.glob('tsan.*')])
    (case/'status.json').write_text(json.dumps(record,indent=2)+'\n')
    print(index,result.returncode,record['reports'],flush=True)
