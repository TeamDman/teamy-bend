# SPDX-License-Identifier: MPL-2.0
"""Research-only manual core factoring; successful probes do not qualify source lowering."""
from pathlib import Path
import argparse,hashlib,json,os,signal,subprocess,shutil,time
p=argparse.ArgumentParser();p.add_argument('--reference',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
repo=Path(__file__).resolve().parents[2];out=a.output.resolve();assert not out.exists();assert shutil.disk_usage(repo).free>10*2**30
assert not subprocess.check_output(['git','status','--porcelain'],cwd=repo)
assert not subprocess.check_output(['git','status','--porcelain'],cwd=a.reference)
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=a.reference,text=True).strip()=='e6676b080f25b1bc1bf5b5b7d7a17e22f8022599'
out.mkdir(parents=True);h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();c=[]
for fp in (repo/'target/debug/.fingerprint').glob('teamy-bend-*/lib-teamy_bend.json'):
 meta=json.loads(fp.read_text());artifact=repo/'target/debug/deps'/('libteamy_bend-'+fp.parent.name.removeprefix('teamy-bend-')+'.rlib')
 if artifact.exists() and json.loads(meta['features'])==['default','extended_observability']:c.append((artifact.stat().st_mtime,artifact,meta))
_,library,meta=max(c);source=repo/'scripts/compact-patterns/prototype.rs';binary=out/'prototype'
d=dict(tested_source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),source_sha256=h(source),library_sha256=h(library),library_fingerprint=meta,observations=[],completed=False,production_adopted=False)
def run(lane,cmd,rep=1):
 cmd=list(map(str,cmd));t=time.monotonic();proc=subprocess.Popen(cmd,cwd=repo,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
 try:o,e=proc.communicate(timeout=30);timed=False
 except subprocess.TimeoutExpired:os.killpg(proc.pid,signal.SIGKILL);o,e=proc.communicate();timed=True
 x=dict(lane=lane,rep=rep,command=cmd,exit=proc.returncode,stdout=o.decode(),stderr=e.decode(),timed_out=timed,seconds=time.monotonic()-t);d['observations'].append(x);(out/'receipt.json').write_text(json.dumps(d,indent=2)+'\n');assert not timed and x['exit']==0,x;return x
run('rustc',['rustc','--edition=2024','-C','debuginfo=0',source,'--extern','teamy_bend='+str(library),'-L','dependency='+str(repo/'target/debug/deps'),'-o',binary]);d['binary_sha256']=h(binary)
fixtures=repo/'tests/fixtures/cpu-compact-patterns';d['fixture_hashes']={p.name:h(p) for p in fixtures.glob('*.bend')}
for rep in [1,2]:
 for lane,extra in [('direct-cut',[]),('explicit-call',['--call'])]:
  x=run(lane,[binary,fixtures/'helpers.bend',*extra],rep)
  assert 'check=accepted\n' in x['stdout'] and 'normalize:main:ok:True{}\n' in x['stdout']
  assert 'normalize:is_hi:error:kernel nesting limit exhausted; proof not accepted\n' in x['stdout']
 for kind in ['original-open']:
  run('upstream-normalization',['node',repo/'scripts/compact-patterns/upstream-normalized.mjs',a.reference/'bend2/bend.ts',fixtures/(kind+'.bend')],rep)
for lane in ['direct-cut','explicit-call','upstream-normalization']:
 xs=[x for x in d['observations'] if x['lane']==lane];assert all(xs[0][k]==xs[1][k] for k in ['stdout','stderr','exit'])
up={x['name']:x for x in map(json.loads,next(x['stdout'] for x in d['observations'] if x['lane']=='upstream-normalization').splitlines())}
cut=next(x['stdout'] for x in d['observations'] if x['lane']=='direct-cut');call=next(x['stdout'] for x in d['observations'] if x['lane']=='explicit-call')
line=lambda text,prefix:next(l.removeprefix(prefix) for l in text.splitlines() if l.startswith(prefix))
expanded=line(cut,'expanded_canonical=');opened=line(cut,'canonical:open:');called=line(call,'canonical:open:')
assert expanded==up['is_hi']['canonical'] and opened==up['open']['canonical'] and called!=opened
for value,expected in [('hi','True{}'),('ho','False{}'),('h','False{}'),('','False{}'),('xi','False{}'),('hii','False{}')]:
 for text in [cut,call]:
  for method in ['evaluate','data']:assert f'{method}:"{value}":ok:{expected}\n' in text
assert 'compact_metrics=(72, 616)\n' in cut and 'expanded_metrics=(138, 610)\n' in cut
d.update(completed=True,results=dict(compact_depth=72,compact_tree_nodes=616,expanded_depth=138,expanded_tree_nodes=610,expanded_full_alpha_equal=True,expanded_canonical_sha256=hashlib.sha256(expanded.encode()).hexdigest(),open_full_alpha_equal=True,explicit_call_open_mismatch=True,whole_function_normalization='refused by unchanged128 kernel normalization guard',original_source_execution='not implemented by this manual Book prototype',closed_inputs=6,repetitions=2,methods=['evaluate','evaluate_data']))
(out/'receipt.json').write_text(json.dumps(d,indent=2)+'\n');print(json.dumps(d['results'],indent=2))
