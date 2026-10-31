# Copyright (c) 2026 Rexx Language Association. All rights reserved.
# This program and the accompanying materials are made available under the
# terms of the Common Public License v1.0 which accompanies this distribution.
# A copy is also available at https://www.oorexx.org/license.html.
from pathlib import Path
import json, os, subprocess
ROOT=Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
OUT=Path(__file__).resolve().parent
HEAD=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT).decode().strip()
env=dict(os.environ,CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='4',CARGO_TARGET_DIR='/tmp/codex-p9d-t88-target')
assert env.get('REXX_TEST_THREAD_SANITIZER') != '1'
cases=[('normal',['cargo','test','-p','rexx-exec','--lib','--test','rexx_command','--test','signals','--test','stdin_contention','--test','program_end','--test','concurrency_tests','--test','close_corrections','--test','dispatch_seam','--test','environment_seam','--test','refusal_sites','--test','refusal_dispositions','--test','closed_phases','-j4']),('depth-release',['cargo','test','-p','rexx-exec','--release','--lib','eval::tests::a_parse_pattern_and_an_expression_share_the_eval_depth_limit','-j4','--','--exact']),('unsafe',['cargo','test','-p','rexx-core','--test','unsafe_sites','-j4']),('fmt',['cargo','fmt','--all','--check']),('clippy',['cargo','clippy','-p','rexx-exec','--all-targets','-j4','--','-D','warnings'])]
for name,args in cases:
 path=OUT/name;assert not path.with_suffix('.status').exists()
 command=['timeout','-k','5','580',*args]
 with path.with_suffix('.stdout').open('wb') as stdout,path.with_suffix('.stderr').open('wb') as stderr:
  result=subprocess.run(command,cwd=ROOT/'rust',env=env,stdin=subprocess.DEVNULL,stdout=stdout,stderr=stderr)
 path.with_suffix('.status').write_text(str(result.returncode)+'\n')
 path.with_suffix('.json').write_text(json.dumps({'command':command,'cwd':str(ROOT/'rust'),'source_commit':HEAD,'status':result.returncode,'outer_cap':'8G','marker':env.get('REXX_TEST_THREAD_SANITIZER')},indent=2)+'\n')
 print(name,result.returncode,flush=True)
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT).decode().strip()==HEAD
 if result.returncode:raise SystemExit(result.returncode)
