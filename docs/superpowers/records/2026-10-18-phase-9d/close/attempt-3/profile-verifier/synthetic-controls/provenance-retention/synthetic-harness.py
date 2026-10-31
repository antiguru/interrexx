from pathlib import Path
import hashlib,json,re,statistics,subprocess,tempfile
root=Path('/home/moritz/dev/repos/ooRexx-rust-rewrite');script=Path('/tmp/p9d-close-verify-profiles.py');base=Path(tempfile.mkdtemp(prefix='p9d-profile-verifier-controls-'));profiles=base/'profiles';profiles.mkdir()
pin=base/'pin';head=base/'head';pin.write_bytes(b'pin synthetic');head.write_bytes(b'head synthetic');sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
instrument=(root/'rust/bench-programs/callgrind.sh').read_text();ping=re.search(r'^PINGPONG="([^"]+)"',instrument,re.M).group(1).split();programs=re.search(r'^PROGRAMS="([^"]+)"',instrument,re.M).group(1).replace('$PINGPONG',' '.join(ping)).split()
(profiles/'binaries.txt').write_text(f'# synthetic rounds=3 REXX_LIB_DIR=synthetic\n# valgrind-3.27.1\n# pin {pin} {sha(pin)}\n# head {head} {sha(head)}\n')
rows=[];table=['program\tpin\thead\thead d%\tpin -libc\thead -libc\thead -libc d%\tpin spread%\thead spread%']
for program in programs:
 source=root/'rust/bench-rexxcps/rexxcps.rex' if program=='rexxcps' else root/'rust/bench-programs/pingpong'/(program+'.rex') if program in ping else root/'rust/bench-programs'/(program+'.rex')
 for r in range(1,4):
  for binary,exe,n in [('pin',pin,100),('head',head,101)]:
   pre=profiles/f'{program}.{binary}.r{r}';whole=n;libc=10;ld=5;own=n-15
   cg=f'# callgrind format\ncreator: callgrind-3.27.1\ncmd: {exe} {source}\ndesc: Trigger: Program termination\npositions: line\nevents: Ir\nsummary: {whole}\nob={exe}\nfl=synthetic\nfn=synthetic\n1 {own}\ncalls=1 1\n* 999\nob=/usr/lib/libc.so.6\n2 10\nob=/usr/lib/ld-linux-x86-64.so.2\n3 5\ntotals: {whole}\n'
   Path(str(pre)+'.cg').write_text(cg)
   Path(str(pre)+'.vg').write_text(f'==1== Using Valgrind-3.27.1 and LibVEX\n==1== Command: {exe} {source}\n==1== Collected : {whole}\n==1== I refs: {whole}\n')
   Path(str(pre)+'.out').write_bytes(b'correct\n');Path(str(pre)+'.err').write_bytes(b'');Path(str(pre)+'.rc').write_text('0\n')
   row=f'{binary}\t{program}\tr{r}\t{whole}\t{libc}\t{ld}\t{own}\trc=0';rows.append(row);Path(str(pre)+'.row').write_text(row+'\n')
 table.append(f'{program}\t100\t101\t+1.0000\t85\t86\t{100/85:+.4f}\t0.0000\t0.0000')
(profiles/'summary.tsv').write_text('\n'.join(rows)+'\n');(profiles/'table.txt').write_text('\n'.join(table)+'\n')
args=['python3','-B',str(script),'--root',str(root),'--profiles',str(profiles),'--pin-binary',str(pin),'--pin-sha256',sha(pin),'--head-binary',str(head),'--head-sha256',sha(head)]
results=[]
def run(name,expected):
 out=base/name;result=subprocess.run(args+['--output',str(out)],capture_output=True);(base/(name+'.stdout')).write_bytes(result.stdout);(base/(name+'.stderr')).write_bytes(result.stderr);assert result.returncode==expected,(name,result.returncode,result.stderr.decode());return json.loads((out/'validation.json').read_text())
valid=run('valid-over',0);assert valid['valid'] and valid['validated_profiles']==198 and len(valid['budget_diagnostics'])==33;results.append('valid OVER stays exit 0 and is not accepted')
ref=profiles/'alloc.head.r1.out';ref.write_bytes(b'wrong\n');bad=run('stdout-invalid',2);assert any('stdout mismatch' in e['error'] for e in bad['errors']);ref.write_bytes(b'correct\n');results.append('semantic mismatch rejected')
ref=profiles/'alloc.head.r1.cg';old=ref.read_text();ref.write_text(old.replace('totals: 101\n',''));bad=run('truncated-invalid',2);assert not bad['valid'];ref.write_text(old);results.append('truncated profile rejected')
ref=profiles/'alloc.head.r1.rc';ref.write_text('66\n');assert not run('status-invalid',2)['valid'];ref.write_text('0\n');results.append('nonzero status rejected')
ref=profiles/'alloc.head.r1.err';ref.write_bytes(b'diagnostic\n');bad=run('stderr-invalid',2);assert any(d['stderr_preview']=='diagnostic\n' for d in bad['diagnostics']);ref.write_bytes(b'');results.append('semantic stderr retained and rejected')
# Existing output must remain untouched rather than silently overwritten.
result=subprocess.run(args+['--output',str(base/'valid-over')],capture_output=True);assert result.returncode!=0;results.append('existing output refused')
(base/'results.json').write_text(json.dumps(results,indent=2)+'\n');print(base);print('\n'.join(results))

pre=profiles/'alloc.head.r3'
originals={suffix:Path(str(pre)+'.'+suffix).read_text() for suffix in ['cg','vg','row']}
old_summary=(profiles/'summary.tsv').read_text();old_table=(profiles/'table.txt').read_text()
Path(str(pre)+'.cg').write_text(originals['cg'].replace('summary: 101','summary: 102').replace('totals: 101','totals: 102').replace('1 86\n','1 87\n'))
Path(str(pre)+'.vg').write_text(originals['vg'].replace('Collected : 101','Collected : 102').replace('I refs: 101','I refs: 102'))
newrow=originals['row'].replace('\t101\t10\t5\t86\t','\t102\t10\t5\t87\t');Path(str(pre)+'.row').write_text(newrow)
(profiles/'summary.tsv').write_text(old_summary.replace(originals['row'].strip(),newrow.strip()))
(profiles/'table.txt').write_text(old_table.replace('alloc\t100\t101\t+1.0000\t85\t86\t+1.1765\t0.0000\t0.0000','alloc\t100\t101\t+1.0000\t85\t86\t+1.1765\t0.0000\t0.9901'))
bad=run('spread-invalid',2);assert any('SPREAD' in e['error'] for e in bad['errors']);results.append('SPREAD rejected from coherent raw counts')
for suffix,value in originals.items():Path(str(pre)+'.'+suffix).write_text(value)
(profiles/'summary.tsv').write_text(old_summary);(profiles/'table.txt').write_text(old_table)
ref=profiles/'alloc.head.r1.row';old=ref.read_text();ref.unlink();bad=run('tuple-invalid',2);assert not bad['valid'];ref.write_text(old);results.append('missing tuple artifact rejected')
(base/'results.json').write_text(json.dumps(results,indent=2)+'\n');print('Additional SPREAD and missing-artifact controls passed.')
