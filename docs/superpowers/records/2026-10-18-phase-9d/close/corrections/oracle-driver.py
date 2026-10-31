# Copyright (c) 2026 Rexx Language Association. All rights reserved.
# This program and the accompanying materials are made available under the
# terms of the Common Public License v1.0 which accompanies this distribution.
# A copy is also available at https://www.oorexx.org/license.html.

from pathlib import Path
import os, subprocess, tempfile, resource, json, shutil, sys
root=Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
records=root/'docs/superpowers/records/2026-10-18-phase-9d/close/corrections'
names=sys.argv[1:] or ['constant_result_cleanup','raise_additional_items','define_methods_supplier_override']
text=(root/'rust/crates/rexx-parse/tests/sourceline_oracle.rs').read_text()
block=text.split('//! ```rexx\n',1)[1].split('//! ```',1)[0]
driver='\n'.join(line.removeprefix('//! ').removeprefix('//!') for line in block.splitlines())+'\n'
def cap():resource.setrlimit(resource.RLIMIT_AS,(1073741824,1073741824))
for name in names:
 cwd=Path(tempfile.mkdtemp(prefix='p9d-correction-oracle-'))
 source=cwd/'probe.rex';shutil.copyfile(root/'rust/corpus/lang'/f'{name}.rex',source)
 env=os.environ.copy();env['LD_LIBRARY_PATH']='/home/moritz/dev/repos/ooRexx/build/lib'
 for key in list(env):
  if key.startswith('REXX_'):del env[key]
 for mode in ['oracle','sourceline']:
  out=records/(name+os.environ.get('ORACLE_RECORD_SUFFIX',''))/mode;out.parent.mkdir(exist_ok=True)
  assert not out.with_suffix('.status').exists()
  if mode=='sourceline':
   (cwd/'source.rex').write_text(driver)
   args=[str(cwd/'source.rex'),str(source)]
  else:args=[str(source)]
  cmd=['timeout','-k','5','30','/home/moritz/dev/repos/ooRexx/build/bin/rexx']+args
  run=subprocess.run(cmd,cwd=cwd,env=env,preexec_fn=cap,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
  out.with_suffix('.stdout').write_bytes(run.stdout);out.with_suffix('.stderr').write_bytes(run.stderr);out.with_suffix('.status').write_text(str(run.returncode)+'\n');out.with_suffix('.json').write_text(json.dumps({'command':cmd,'cwd':str(cwd),'status':run.returncode},indent=2)+'\n')
  print(name,mode,run.returncode,run.stdout.decode(errors='replace'))
  assert run.returncode==0
  if mode=='sourceline':
   lines=run.stdout.splitlines();count=[x[len(b'%SRCG%COUNT '):] for x in lines if x.startswith(b'%SRCG%COUNT ')];assert len(count)==1
   output=b'count '+count[0]+b'\n'+b'\n'.join(x[len(b'%SRCG%L'):] for x in lines if x.startswith(b'%SRCG%L'))+b'\n'
   (root/'rust/crates/rexx-parse/tests/sourceline_oracle'/f'{name}.txt').write_bytes(output)
