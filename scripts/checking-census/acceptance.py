# SPDX-License-Identifier: MPL-2.0
"""Renew a recorded acceptance subset without executing any Bend program."""
from pathlib import Path
import argparse
import base64
import collections
import concurrent.futures
import csv
import hashlib
import json
import os
import shutil
import signal
import subprocess
import tempfile
import time

parser = argparse.ArgumentParser(description=__doc__)
for name in ['reference', 'binary', 'phase-probe', 'census', 'output', 'build-receipt']:
    parser.add_argument('--' + name, required=True, type=Path)
args = parser.parse_args()
for key, value in vars(args).items():
    setattr(args, key, value.resolve())
here = Path(__file__).resolve().parent
repo = here.parent.parent
git = lambda path, *parts: subprocess.check_output(['git', '-C', str(path), *parts], text=True).strip()
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
pin = 'e6676b080f25b1bc1bf5b5b7d7a17e22f8022599'
assert git(args.reference, 'rev-parse', 'HEAD') == pin
assert not git(args.reference, 'status', '--porcelain')
assert not git(repo, 'status', '--porcelain'), 'Use a clean tested checkpoint'
assert not args.output.exists()
assert not args.output.resolve().is_relative_to(args.reference.resolve())
snapshot = dict(source_commit=git(repo, 'rev-parse', 'HEAD'),
                binary_sha256=sha(args.binary), phase_probe_sha256=sha(args.phase_probe))
build = json.loads(args.build_receipt.read_text())
assert all(build[k] == v for k, v in snapshot.items()), 'Build receipt mismatch'
with args.census.open() as stream:
    selected = [row for row in csv.DictReader(stream)
                if row['classification'] == 'acceptance_disagreement'
                and row['renewed_historical_IO_member'] == 'false']
assert selected and len({row['fixture'] for row in selected}) == len(selected)
assert shutil.disk_usage(repo).free >= 11 * 2**30
args.output.mkdir(parents=True)


def run(command, timeout=60):
    assert shutil.disk_usage(repo).free > 10 * 2**30
    assert shutil.disk_usage(tempfile.gettempdir()).free > 2**30
    command = list(map(str, command))
    start = time.monotonic()
    child = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                             stdin=subprocess.DEVNULL, start_new_session=True)
    timed = False
    try:
        out, err = child.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        timed = True
        os.killpg(child.pid, signal.SIGKILL)
        out, err = child.communicate()
    return dict(command=command, exit=child.returncode, timed_out=timed,
                timeout_seconds=timeout, seconds=time.monotonic() - start,
                stdout_base64=base64.b64encode(out).decode(),
                stderr_base64=base64.b64encode(err).decode())


def observe(row):
    source = args.reference / 'tests' / row['fixture']
    assert sha(source) == row['source_sha256']
    key = hashlib.sha256(row['fixture'].encode()).hexdigest()
    phase = args.output / (key + '.phase.json')
    upstream = run(['node', here / 'upstream.mjs', args.reference / 'bend2/bend.ts', source, phase], 30)
    if phase.exists():
        upstream.update(json.loads(phase.read_text()))
    strict = run([args.binary, '--output-format', 'json', 'check', source])
    executable = run([args.phase_probe, source, '--executable'])
    result = dict(fixture=row['fixture'], source_sha256=sha(source), upstream=upstream,
                  strict=strict, executable_phase=executable)
    if executable['exit'] == 1 and not executable['timed_out']:
        # Renew the executable error without invoking an unknown effectful main.
        artifact = args.output / (key + '.js')
        result['executable_refusal'] = run([args.binary, 'compile', source, '--executable',
                                          '--target', 'javascript', '--output', artifact])
        result['artifact_emitted'] = artifact.exists()
    return result


rows = []
with (args.output / 'observations.jsonl').open('w') as stream:
    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
        for row in pool.map(observe, selected):
            rows.append(row)
            stream.write(json.dumps(row, separators=(',', ':')) + '\n')
            stream.flush()
            if len(rows) % 20 == 0:
                print('Measured', len(rows), 'of', len(selected), flush=True)
assert sha(args.binary) == snapshot['binary_sha256']
assert sha(args.phase_probe) == snapshot['phase_probe_sha256']
valid = all(row['upstream']['exit'] == 0 and row['upstream'].get('phase') == 'accepted'
            and row['strict']['exit'] == 1
            and not any(observation['timed_out'] for observation in [row['upstream'], row['strict'], row['executable_phase']])
            and ((row['executable_phase']['exit'], row['executable_phase']['stdout_base64'])
                 in [(0, 'YWNjZXB0ZWQK'), (1, 'bG9hZAo='), (1, 'Y2hlY2sK')])
            and not row['executable_phase']['stderr_base64']
            and ('executable_refusal' not in row or
                 (row['executable_refusal']['exit'] == 1
                  and not row['executable_refusal']['timed_out'] and not row['artifact_emitted']))
            for row in rows)
summary = dict(**snapshot, reference=pin, selected=len(selected), completed=True,
               acceptance_observations_renewed=valid, census_sha256=sha(args.census),
               phase_counts=dict(collections.Counter(base64.b64decode(row['executable_phase']['stdout_base64']).decode().strip() for row in rows)),
               scope='Checking and refusal generation only. No program executed; no observation waived.')
(args.output / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
print(json.dumps(summary, indent=2), flush=True)
raise SystemExit(0 if valid else 1)
