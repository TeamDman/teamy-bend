# SPDX-License-Identifier: MPL-2.0
from pathlib import Path
import subprocess, concurrent.futures, csv, json, hashlib, base64, os, signal, time, collections
import argparse, shutil
parser=argparse.ArgumentParser(description="Checking-only full pinned inventory census; never execute or normalize main.")
parser.add_argument('--reference', type=Path, required=True)
parser.add_argument('--binary', type=Path, required=True)
parser.add_argument('--phase-probe', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--build-receipt', type=Path, required=True)
args=parser.parse_args()
r=Path(__file__).resolve().parent;repo=r.parent.parent
ref=args.reference.resolve();binary=args.binary.resolve();probe=args.phase_probe.resolve();root=args.output.resolve()
git=lambda path,*parts: subprocess.check_output(['git','-C',str(path),*parts],text=True).strip()
pin='e6676b080f25b1bc1bf5b5b7d7a17e22f8022599'
assert git(ref,'rev-parse','HEAD')==pin and not git(ref,'status','--porcelain')
assert not git(repo,'status','--porcelain'), 'Use a clean tested source checkpoint'
assert not root.exists() and not root.is_relative_to(ref)
assert shutil.disk_usage(repo).free>=10*2**30, 'Preserve the recorded workspace disk floor'
root.mkdir(parents=True)
snap=dict(source_commit=git(repo,'rev-parse','HEAD'),binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),phase_probe_sha256=hashlib.sha256(probe.read_bytes()).hexdigest())
build=json.loads(args.build_receipt.read_text())
assert all(build[k]==snap[k] for k in snap), 'Build provenance does not match the clean source and executables'
inventory=list(csv.DictReader((repo/'docs/cpu-applicability.csv').open()));assert len(inventory)==1302
assert {x['fixture'] for x in inventory}=={p.relative_to(ref/'tests').as_posix() for p in (ref/'tests').rglob('*.bend')}
rows=[];env={**os.environ,'BEND_GPU':'off','CARGO_NET_OFFLINE':'true'}
def run(cmd,timeout):
 start=time.monotonic();p=subprocess.Popen(list(map(str,cmd)),stdout=subprocess.PIPE,stderr=subprocess.PIPE,stdin=subprocess.DEVNULL,start_new_session=True,env=env);timed=False
 try:o,e=p.communicate(timeout=timeout)
 except subprocess.TimeoutExpired:timed=True;os.killpg(p.pid,signal.SIGKILL);o,e=p.communicate()
 return dict(command=list(map(str,cmd)),exit=p.returncode,timed_out=timed,timeout_seconds=timeout,elapsed_ms=round(1000*(time.monotonic()-start)),stdout_base64=base64.b64encode(o).decode(),stderr_base64=base64.b64encode(e).decode())
def observation(row):
 path=ref/'tests'/row['fixture'];assert hashlib.sha256(path.read_bytes()).hexdigest()==row['source_sha256'];phase=root/(hashlib.sha256(row['fixture'].encode()).hexdigest()+'.phase.json')
 up=run(['node',r/'upstream.mjs',ref/'bend2/bend.ts',path,phase],30)
 if phase.exists():up.update(json.loads(phase.read_text()))
 port=run([binary,'--output-format','json','check',path],60)
 phase_result=run([probe,path],60);phase_out=base64.b64decode(phase_result['stdout_base64']).decode().strip();port['phase']=phase_out
 valid=not any(x['timed_out'] or x['exit'] not in [0,1] for x in [up,port,phase_result]) and up.get('phase') in ['load','check','accepted'] and phase_out in ['load','check','accepted'] and phase_result['exit']==port['exit'] and not phase_result['stderr_base64'] and (up['phase']=='accepted')==(up['exit']==0) and (phase_out=='accepted')==(port['exit']==0) and up.get('outcome') in ['accepted','language_rejection','incomplete_proof']
 exact=valid and port['exit']==up['exit']==1 and up['phase']==port['phase'] and all(up[k]==port[k] for k in ['stdout_base64','stderr_base64'])
 kind='oracle_or_infrastructure_failure' if not valid else 'both_accept' if up['exit']==port['exit']==0 else 'exact_rejection' if exact else 'different_rejection' if up['exit']==port['exit']==1 else 'acceptance_disagreement'
 err=base64.b64decode(port['stderr_base64']).decode(errors='replace');io=valid and up['exit']==0 and port['exit']==1 and port['phase']=='check' and '- expected : a defined name\n- observed : IO\n' in err
 return dict(fixture=row['fixture'],source_sha256=row['source_sha256'],inventory_category=row['category'],classification=kind,missing_IO_candidate=io,upstream=up,port=port,port_phase_probe=phase_result)
with (root/'observations.jsonl').open('w') as stream:
 with concurrent.futures.ThreadPoolExecutor(max_workers=4) as ex:
  for row in ex.map(observation,inventory):
   rows.append(row);stream.write(json.dumps(row,separators=(',',':'))+'\n');stream.flush()
   if len(rows)%100==0:print('Measured',len(rows),'of 1302',flush=True)
assert hashlib.sha256(binary.read_bytes()).hexdigest()==snap['binary_sha256']
assert hashlib.sha256(probe.read_bytes()).hexdigest()==snap['phase_probe_sha256']
counts=collections.Counter(x['classification'] for x in rows);io=sum(x['missing_IO_candidate'] for x in rows)
summary=dict(completed=True,full_parity=False,tested_source=snap['source_commit'],binary_sha256=snap['binary_sha256'],phase_probe_sha256=snap['phase_probe_sha256'],build_receipt_sha256=hashlib.sha256(args.build_receipt.read_bytes()).hexdigest(),reference=pin,total=len(rows),counts=dict(counts),missing_IO_candidates=io,measured_differences=sum(x['classification'] in ['different_rejection','acceptance_disagreement'] for x in rows),scope='Checking-only. Both-accept is status agreement, not output/normalization/runtime parity. Missing-IO annotations are symptom subcounts, not exclusions or additional categories. Phase probes are separate from raw CLI streams. Non-exact and oracle failures remain unqualified.')
(root/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(summary,indent=2),flush=True)
