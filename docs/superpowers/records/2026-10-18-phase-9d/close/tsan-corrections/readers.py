# Copyright (c) 2026 Rexx Language Association. All rights reserved.
# This program and the accompanying materials are made available under the
# terms of the Common Public License v1.0 which accompanies this distribution.
# A copy is also available at https://www.oorexx.org/license.html.
from pathlib import Path
import hashlib,json,os,select,signal,subprocess,sys,time
ROOT=Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
OUT=Path(__file__).resolve().parent
binary=Path(sys.argv[1]); label=sys.argv[2]
BASE="call on halt name h\ns.0 = 5; s.1 = 'old'\nsay 'a'\naddress system '{command}' with output stem s.{error}\nsay 'b' rc s.0 s.1\nexit\nh: say 'halted' sigl; return\n"
cases=[('no-op-stderr','echo first; sleep 5; echo late',''),('real-stderr-eof','printf err >&2; exec 2>&-; echo first; sleep 5; echo late',' error stem e.'),('real-stdout-eof','echo first; exec 1>&-; printf err >&2; sleep 5; printf late >&2',' error stem e.')]
rows=[]
for name,command,error in cases:
 case=OUT/(label+'-'+name);case.mkdir(exist_ok=False)
 program=case/'program.rex';program.write_text(BASE.format(command=command,error=error))
 env=dict(os.environ,REXX_TEST_THREAD_SANITIZER='1')
 env['TSAN_OPTIONS']=f'log_path={case}/tsan suppressions={ROOT}/rust/tsan.supp allocator_may_return_null=1 second_deadlock_stack=1'
 argv=['env','--default-signal=INT,TERM,HUP',str(binary),str(program)]
 start=time.monotonic();child=subprocess.Popen(argv,cwd=ROOT/'rust',env=env,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
 captured=b''
 try:
  while not captured.endswith(b'a\n') and time.monotonic()-start<15:
   if select.select([child.stdout],[],[],0.1)[0]:
    data=os.read(child.stdout.fileno(),4096)
    if not data:break
    captured+=data
  assert captured.endswith(b'a\n'),captured
  time.sleep(.5);os.kill(child.pid,signal.SIGINT)
  status=child.wait(timeout=20);elapsed=time.monotonic()-start
  stdout,stderr=child.communicate(timeout=12)
 finally:
  if child.poll() is None:child.kill();child.wait()
  try:os.killpg(child.pid,signal.SIGTERM)
  except ProcessLookupError:pass
 stdout=captured+stdout
 (case/'stdout').write_bytes(stdout);(case/'stderr').write_bytes(stderr);(case/'status').write_text(str(status)+'\n')
 reports=[p.name for p in case.glob('tsan.*')]
 row={'name':name,'command':argv,'status':status,'process_seconds':elapsed,'reports':reports,'TSAN_OPTIONS':env['TSAN_OPTIONS'],'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'stdout_expected':stdout==b'a\nb -4 1 first\nhalted 5\n','stderr_expected':stderr==b''}
 (case/'metadata.json').write_text(json.dumps(row,indent=2)+'\n');rows.append(row)
 print(label,name,status,reports,flush=True)
 if label=='after':assert status==0 and not reports and row['stdout_expected'] and row['stderr_expected']
(OUT/(label+'-readers.json')).write_text(json.dumps(rows,indent=2)+'\n')
if label=='before':assert all(row['reports'] for row in rows),rows
