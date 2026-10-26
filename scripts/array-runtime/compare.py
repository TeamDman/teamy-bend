# SPDX-License-Identifier: MPL-2.0
"""Bounded, exact CPU array observations; residual refusals stay unqualified."""
from pathlib import Path
import argparse, base64, hashlib, json, os, signal, shutil, subprocess, time

p = argparse.ArgumentParser()
p.add_argument('--repo', type=Path, required=True)
p.add_argument('--reference', type=Path, required=True)
p.add_argument('--binary', type=Path, required=True)
p.add_argument('--manifest', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
pin = 'e6676b080f25b1bc1bf5b5b7d7a17e22f8022599'
assert subprocess.check_output(['git','rev-parse','HEAD'], cwd=a.reference, text=True).strip() == pin
assert not subprocess.check_output(['git','status','--porcelain'], cwd=a.reference).strip()
cases = json.loads(a.manifest.read_text())
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
for case in cases:
    source = a.repo / case['path']
    assert sha(source) == case['source_sha256']
    if 'upstream_path' in case:
        assert source.read_bytes() == (a.reference / case['upstream_path']).read_bytes()
a.output.mkdir()
result = {'source_commit': subprocess.check_output(['git','rev-parse','HEAD'], cwd=a.repo, text=True).strip(), 'dirty': bool(subprocess.check_output(['git','status','--porcelain'], cwd=a.repo)), 'upstream': pin, 'binary_sha256': sha(a.binary), 'manifest_sha256': sha(a.manifest), 'rows': [], 'cases': [], 'completed': False}
receipt = a.output / 'results.json'
with receipt.open('x') as f: json.dump(result, f)
def save():
    receipt.write_text(json.dumps(result, separators=(',',':'))+'\n')
def streams(row):
    return (row['exit'], base64.b64decode(row['stdout_base64']), base64.b64decode(row['stderr_base64']))
def run(case, source, lane, cmd, repeat=0):
    assert shutil.disk_usage(a.repo).free >= 10*2**30
    assert shutil.disk_usage('/tmp').free >= 2**30
    row = dict(case=case, source_sha256=sha(source), lane=lane, command=list(map(str,cmd)), repeat=repeat, state='started')
    result['rows'].append(row); save()
    started=time.monotonic()
    child=subprocess.Popen(row['command'], cwd=a.repo, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
    try: out, err=child.communicate(timeout=60); timeout=False
    except subprocess.TimeoutExpired:
        os.killpg(child.pid,signal.SIGKILL); out,err=child.communicate(); timeout=True
    row.update(exit=child.returncode, stdout_base64=base64.b64encode(out).decode(), stderr_base64=base64.b64encode(err).decode(), seconds=time.monotonic()-started, timed_out=timeout, state='completed')
    save(); return row
for index, case in enumerate(cases):
    name=case['case']; source=a.repo / case['path']; folder=a.output / str(index); folder.mkdir()
    phases=folder/'upstream-phase.json'
    up=run(name,source,'upstream-check',['node',a.repo/'scripts/checking-census/upstream.mjs',a.reference/'bend2/bend.ts',source,phases])
    if phases.exists(): up['phase']=json.loads(phases.read_text()); save()
    port=run(name,source,'port-strict-check',[a.binary,'check',source])
    ev=run(name,source,'port-strict-eval',[a.binary,'eval',source])
    programs={'native':[a.binary,'run',source]}
    artifacts={}
    for side in ['upstream','port']:
        for target, ext in [('c','c'),('javascript','js')]:
            artifact=folder/(side+'.'+ext)
            command=['node',a.reference/'bend2/main.ts',source,'-o',artifact] if side=='upstream' else [a.binary,'compile',source,'--executable','--target',target,'--output',artifact]
            generated=run(name,source,side+'-generate-'+ext,command)
            if generated['exit'] or not artifact.exists(): continue
            generated['artifact_sha256']=sha(artifact); artifacts[side+'-'+ext]=generated['artifact_sha256']; save()
            if ext=='c':
                exe=folder/(side+'.bin')
                flags=[] if side=='upstream' else ['-Wall','-Wextra','-Werror','-pedantic']
                libraries=['-lpthread','-lm'] if side=='upstream' else ['-pthread','-lm','-ldl']
                built=run(name,source,side+'-compile-O3',[os.environ['TEAMY_BEND_CC'],'-std=c11','-O3',*flags,artifact,*libraries,'-o',exe])
                if built['exit']==0: programs[side+'-C-O3']=[exe,'--threads','1','--gpu','off']
            else: programs[side+'-JS']=['node',artifact]
    runs={}
    for lane, cmd in programs.items():
        runs[lane]=[run(name,source,lane,cmd,n) for n in [1,2]]
    comparisons={lane:streams(rows[0])==streams(rows[1]) for lane,rows in runs.items()}
    expected=case.get('stdout')
    result['cases'].append(dict(case=name,kind=case['kind'],deterministic=comparisons,all_five_executable_lanes=len(runs)==5,executions={lane:dict(exit=rows[0]['exit'],expected_stdout=(base64.b64decode(rows[0]['stdout_base64']).decode()==expected if expected is not None else None),empty_stderr=not base64.b64decode(rows[0]['stderr_base64'])) for lane,rows in runs.items()},corresponding_backend={ext:streams(runs['port-'+ext][0])==streams(runs['upstream-'+ext][0]) if 'port-'+ext in runs and 'upstream-'+ext in runs else None for ext in ['C-O3','JS']},artifacts=artifacts))
    save(); print(json.dumps(result['cases'][-1]),flush=True)
assert sha(a.binary) == result['binary_sha256']
assert all(not row['timed_out'] for row in result['rows'])
assert all(all(c['deterministic'].values()) for c in result['cases'])
residuals=[]
for case in cases:
    name=case['case']; rows=[r for r in result['rows'] if r['case']==name]
    by_lane={lane:[r for r in rows if r['lane']==lane] for lane in {r['lane'] for r in rows}}
    up=by_lane['upstream-check'][0]
    if case['kind']=='negative':
        assert up['phase']=={'phase':'check','outcome':'language_rejection'}
        expected=(1,b'',case['stderr'].encode())
        assert all(streams(row)==expected for row in rows), name
        assert len(rows)==9 and not any('artifact_sha256' in row for row in rows)
        continue
    assert up['phase']=={'phase':'accepted','outcome':'accepted'} and streams(up)==(0,b'',b'')
    # Strict-proof refusal is a separately observed exclusion, not an execution.
    for lane in ['port-strict-check','port-strict-eval']:
        row=by_lane[lane][0]
        assert row['exit']==1 and streams(row)[1]==b'' and b'a defined name' in streams(row)[2]
    assert len(rows)==19
    for lane in ['upstream-generate-c','upstream-generate-js','port-generate-c','port-generate-js','upstream-compile-O3','port-compile-O3']:
        assert streams(by_lane[lane][0])==(0,b'',b''), (name,lane)
    for lane in ['native','upstream-C-O3','upstream-JS','port-C-O3','port-JS']:
        if case['kind']=='positive':
            expected=(0,case['stdout'].encode(),b'')
        else:
            error=case['port_c_residual_stderr'] if lane=='port-C-O3' else case['stderr']
            expected=(1,b'',error.encode())
        assert all(streams(row)==expected for row in by_lane[lane]), (name,lane)
    if case['kind']=='runtime-negative':
        residuals.append({'case':name,'lane':'port-C-O3','classification':'unwaived resource-refusal diagnostic difference'})
result.update(completed=True,observations=len(result['rows']),timeouts=0,unwaived_residuals=residuals,full_parity=False)
save()
