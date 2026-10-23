# SPDX-License-Identifier: MPL-2.0
"""Bounded Linux F32.read comparison; upstream disagreement keeps exit 1."""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time

PIN = 'e6676b080f25b1bc1bf5b5b7d7a17e22f8022599'
PORT = Path(__file__).resolve().parent.parent


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reference', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--compiler', default=os.environ.get('TEAMY_BEND_CC', 'clang'))
    args = parser.parse_args()
    if sys.platform != 'linux':
        raise ValueError('This probe qualifies the Linux POSIX CPU profile')
    reference = args.reference.resolve()
    output = args.output.resolve()
    git = lambda root, *parts: subprocess.check_output(
        ['git', '-C', str(root), *parts], text=True, timeout=60).strip()
    if git(reference, 'rev-parse', 'HEAD') != PIN or git(reference, 'status', '--porcelain=v1'):
        raise ValueError('Reference must be clean at the pin')
    if output.exists() or output.is_relative_to(reference):
        raise ValueError('Choose fresh evidence outside the reference')
    output.mkdir(parents=True)
    binary = output / 'teamy-bend'
    source = output / 'main.bend'
    shutil.copy2(PORT / 'target/debug/teamy-bend', binary)
    shutil.copyfile(PORT / 'tests/fixtures/cpu-float-text/float-read-boundaries.bend', source)
    node = os.environ.get('TEAMY_BEND_NODE', 'node')
    row = dict(fixture='float-read-boundaries.bend', origin='authored_control',
               source_sha256=digest(source), reference=PIN,
               port_revision=git(PORT, 'rev-parse', 'HEAD'),
               port_dirty=bool(git(PORT, 'status', '--porcelain=v1')),
               port_binary_sha256=digest(binary), timeout_seconds=60, repetitions=2,
               upstream_c_flags=['-std=c11', '-O3'], upstream_c_link_flags=['-lpthread', '-lm'],
               c_arguments=['--threads', '1', '--gpu', 'off'], steps={},
               generated_sha256={}, outcome='incomplete')

    def save():
        (output / 'receipt.json').write_text(json.dumps(row, indent=2) + '\n')

    def run(label, command, cwd=output, phase='execution'):
        command = list(map(str, command))
        started = time.monotonic()
        child = subprocess.Popen(command, cwd=cwd, env={**os.environ, 'BEND_GPU': 'off'},
                                 stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                 stderr=subprocess.PIPE, start_new_session=True)
        timed_out = False
        try:
            stdout, stderr = child.communicate(timeout=60)
        except subprocess.TimeoutExpired:
            timed_out = True
            os.killpg(child.pid, signal.SIGKILL)
            stdout, stderr = child.communicate()
        result = dict(command=command, cwd=str(cwd), phase=phase, exit=child.returncode,
                      timed_out=timed_out, elapsed_seconds=time.monotonic() - started,
                      stdout_base64=base64.b64encode(stdout).decode(),
                      stderr_base64=base64.b64encode(stderr).decode())
        row['steps'][label] = result
        (output / (label + '.stdout')).write_bytes(stdout)
        (output / (label + '.stderr')).write_bytes(stderr)
        save()
        return result

    save()
    for implementation in ['upstream', 'port']:
        for target in ['c', 'js']:
            generated = output / (implementation + '.' + target)
            command = [node, 'main.ts', source, '-o', generated] if implementation == 'upstream' else [
                binary, 'compile', '--executable', source, '--target',
                'c' if target == 'c' else 'javascript', '--output', generated]
            result = run(implementation + '-generate-' + target, command,
                         reference / 'bend2' if implementation == 'upstream' else PORT, 'generation')
            if result['exit'] or result['timed_out']:
                row['outcome'] = 'generation_failed'
                save()
                return 1
            row['generated_sha256'][implementation + '-' + target] = digest(generated)
    for implementation in ['upstream', 'port']:
        flags = ['-std=c11', '-O3'] if implementation == 'upstream' else [
            '-std=c11', '-Wall', '-Wextra', '-Werror', '-pedantic']
        links = ['-lpthread', '-lm'] if implementation == 'upstream' else ['-pthread', '-lm', '-ldl']
        result = run(implementation + '-compile', [args.compiler, *flags,
                     output / (implementation + '.c'), *links, '-o', output / implementation],
                     phase='compilation')
        if result['exit'] or result['timed_out']:
            row['outcome'] = 'compilation_failed'
            save()
            return 1
    lanes = ['upstream-c', 'port-c', 'upstream-js', 'port-js', 'native']
    for repeat in [1, 2]:
        for lane in lanes:
            if lane == 'native':
                command = [binary, 'run', source]
            elif lane.endswith('-js'):
                command = [node, output / lane.replace('-js', '.js')]
            else:
                command = [output / lane.replace('-c', ''), '--threads', '1', '--gpu', 'off']
            run(lane + '-' + str(repeat), command)

    def observation(lane, repeat=1):
        result = row['steps'][lane + '-' + str(repeat)]
        return tuple(result[key] for key in ['exit', 'stdout_base64', 'stderr_base64', 'timed_out'])

    row['deterministic'] = all(observation(lane) == observation(lane, 2) for lane in lanes)
    row['executions_ok'] = all(
        result['exit'] == 0 and not result['stderr_base64'] and not result['timed_out']
        for result in row['steps'].values() if result['phase'] == 'execution')
    row['upstream_agreement'] = observation('upstream-c') == observation('upstream-js')
    # These are observed backend matches, not a waiver of the upstream split.
    row['backend_matches'] = {
        'native_to_upstream_c': observation('native') == observation('upstream-c'),
        'port_c_to_upstream_c': observation('port-c') == observation('upstream-c'),
        'port_js_to_upstream_js': observation('port-js') == observation('upstream-js'),
    }
    if not row['deterministic'] or not row['executions_ok']:
        row['outcome'] = 'execution_failure_or_nondeterminism'
    elif not all(row['backend_matches'].values()):
        row['outcome'] = 'port_backend_difference'
    elif not row['upstream_agreement']:
        row['outcome'] = 'unresolved_upstream_read_contract'
    else:
        row['outcome'] = 'exact_agreement'
    save()
    print(row['outcome'], json.dumps(row['backend_matches']), flush=True)
    return 0 if row['outcome'] == 'exact_agreement' else 1


if __name__ == '__main__':
    raise SystemExit(main())
