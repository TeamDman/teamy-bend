# SPDX-License-Identifier: MPL-2.0
"""Bounded executable and checking observations; unresolved cells remain failures."""
from pathlib import Path
import argparse, hashlib, json, os, shutil, signal, subprocess, time
p=argparse.ArgumentParser()
for name in ['reference','binary','phase-probe','output','build-receipt']:p.add_argument('--'+name,type=Path,required=True)
a=p.parse_args(); repo=Path(__file__).resolve().parents[2];root=a.output.resolve()
assert not root.exists();assert shutil.disk_usage(repo).free>10*2**30
assert subprocess.check_output(['git','-C',str(a.reference),'rev-parse','HEAD'],text=True).strip()=='e6676b080f25b1bc1bf5b5b7d7a17e22f8022599'
assert not subprocess.check_output(['git','status','--porcelain'],cwd=repo)
assert not subprocess.check_output(['git','status','--porcelain'],cwd=a.reference)
root.mkdir(parents=True);fixtures=repo/'tests/fixtures/cpu-string-pattern-depth'
d=dict(tested_source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),phase_probe_sha256=hashlib.sha256(a.phase_probe.read_bytes()).hexdigest(),rows=[],unrun=[],dispositions=[],completed=False,full_parity=False)
build=json.loads(a.build_receipt.read_text())
assert all(build[k]==d[k] for k in ['tested_source','binary_sha256','phase_probe_sha256']), 'Build receipt does not match clean source and artifacts'
d['build_receipt_sha256']=hashlib.sha256(a.build_receipt.read_bytes()).hexdigest()
def save(): (root/'receipt.json').write_text(json.dumps(d,indent=2)+'\n')
def run(case,lane,cmd,rep=1):
 assert shutil.disk_usage(repo).free>10*2**30
 cmd=list(map(str,cmd));start=time.monotonic();proc=subprocess.Popen(cmd,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
 try:o,e=proc.communicate(timeout=30);timed=False
 except subprocess.TimeoutExpired:os.killpg(proc.pid,signal.SIGKILL);o,e=proc.communicate();timed=True
 row=dict(case=case.relative_to(fixtures).as_posix(),source_sha256=hashlib.sha256(case.read_bytes()).hexdigest(),lane=lane,rep=rep,command=cmd,exit=proc.returncode,stdout=o.decode(),stderr=e.decode(),timed_out=timed,seconds=time.monotonic()-start)
 d['rows'].append(row);save();assert not timed,row;return row
for case in sorted(fixtures.rglob('*.bend')):
 label=case.relative_to(fixtures).as_posix();work=root/case.stem;work.mkdir()
 for rep in [1,2]:
  phase=work/f'phase-{rep}.json'; assert not phase.exists()
  row=run(case,'upstream-check',['node',repo/'scripts/checking-census/upstream.mjs',a.reference/'bend2/bend.ts',case,phase],rep);row.update(json.loads(phase.read_text()))
  for lane,args in [('port-strict-phase',[]),('port-executable-phase',['--executable'])]:
   row=run(case,lane,[a.phase_probe,case,*args],rep);row['phase']=row['stdout'].strip()
  run(case,'port-check',[a.binary,'--output-format','json','check',case],rep)
 # Run actual native, evaluator and upstream interpreter independently.
 for rep in [1,2]:
  for lane,cmd in [('port-native',[a.binary,'run',case]),('port-eval',[a.binary,'eval',case]),('upstream-interpreter',['node',a.reference/'bend2/main.ts',case])]:run(case,lane,cmd,rep)
 for who in ['port','upstream']:
  for target,ext in [('c','c'),('javascript','js')]:
   artifact=work/(who+'.'+ext)
   cmd=[a.binary,'compile',case,'--executable','--target',target,'--output',artifact] if who=='port' else ['node',a.reference/'bend2/main.ts',case,'-o',artifact]
   row=run(case,who+'-emit-'+target,cmd)
   if row['exit']!=0:
    assert not artifact.exists();d['unrun'].append(dict(case=label,lane=who+'-'+target,reason='emission refused; no artifact'));continue
   row['artifact_sha256']=hashlib.sha256(artifact.read_bytes()).hexdigest()
   if ext=='c':
    exe=work/(who+'.bin');cmd=[os.environ['TEAMY_BEND_CC'],'-std=c11','-O3',artifact,'-lpthread','-lm','-o',exe]
    if who=='port':cmd[1:1]=['-Wall','-Wextra','-Werror']
    build=run(case,who+'-C-build-O3',cmd)
    if build['exit']!=0:d['unrun'].append(dict(case=label,lane=who+'-C',reason='C build failed'));continue
    command=[exe,'--threads','1','--gpu','off']
   else:command=['node',artifact]
   for rep in [1,2]:run(case,who+'-'+('C-O3' if ext=='c' else 'JS'),command,rep)
 for lane in {x['lane'] for x in d['rows'] if x['case']==label}:
  xs=[x for x in d['rows'] if x['case']==label and x['lane']==lane]
  if len(xs)==2:assert all(xs[0][k]==xs[1][k] for k in ['exit','stdout','stderr']), (label,lane,'nondeterminism')
 if case.parent.name=='positive':
  expected=case.with_suffix('.stdout').read_text()
  for x in [x for x in d['rows'] if x['case']==label]:
   if x['lane'] in ['port-native','port-C-O3','port-JS','upstream-C-O3','upstream-JS','upstream-interpreter','port-eval']:assert (x['exit'],x['stdout'],x['stderr'])==(0,expected,''),x
   else:assert x['exit']==0,x
 else:
  rows=[x for x in d['rows'] if x['case']==label]
  for x in rows:
   if x['lane'].startswith('port-'):
    assert x['exit']==1,x
    if x['lane'].endswith('-phase'):
     assert x['phase']==('check' if case.stem=='wrong-type' else 'load'),x
    elif case.stem!='wrong-type':assert 'resource limit of 64 levels' in x['stderr'],x
   elif case.stem in ['string_pattern_sugar','nat65']:
    assert x['exit']==0,x
    if x['lane'] in ['upstream-interpreter','upstream-C-O3','upstream-JS']:assert (x['stdout'],x['stderr'])==('True{}\n',''),x
   else:
    assert x['exit']==1,x
    if x['lane']=='upstream-check':assert x['phase']=='check',x
 current=[x for x in d['rows'] if x['case']==label]
 up=next(x for x in current if x['lane']=='upstream-check')
 port=next(x for x in current if x['lane']=='port-check')
 port_phase=next(x for x in current if x['lane']=='port-strict-phase')['phase']
 classification='qualified_positive' if case.parent.name=='positive' else 'unresolved_literal_depth' if case.parent.name=='original' else 'retained_resource_refusal' if case.stem in ['nat65','columns65'] else 'wrong_type_rejected'
 exact_rejection=up['exit']==port['exit']==1 and up['phase']==port_phase and all(up[k]==port[k] for k in ['stdout','stderr'])
 d['dispositions'].append(dict(case=label,classification=classification,upstream_phase=up['phase'],port_phase=port_phase,exact_rejection=exact_rejection))
 print(label,classification,flush=True)
 save()
d['completed']=True;save();print('observations',len(d['rows']),'unrun',len(d['unrun']),flush=True)
