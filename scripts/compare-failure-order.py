# SPDX-License-Identifier: MPL-2.0
"""Bounded failure-order research; records differences without granting exceptions."""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
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
    reference = args.reference.resolve()
    output = args.output.resolve()
    git = lambda root, *parts: subprocess.check_output(['git', '-C', str(root), *parts], text=True).strip()
    if git(reference, 'rev-parse', 'HEAD') != PIN or git(reference, 'status', '--porcelain=v1'):
        raise ValueError('Reference must be clean at the pin')
    if output.exists() or output.is_relative_to(reference):
        raise ValueError('Choose fresh evidence outside the reference')
    output.mkdir(parents=True)
    binary = output / 'teamy-bend'
    shutil.copy2(PORT / 'target/debug/teamy-bend', binary)
    revision = git(PORT, 'rev-parse', 'HEAD')
    dirty = bool(git(PORT, 'status', '--porcelain=v1'))
    node = os.environ.get('TEAMY_BEND_NODE', 'node')
    compiler_version = subprocess.check_output([args.compiler, '--version'], text=True)
    cases = [
        ('nat_first', 'cpu-demand-review'),
        ('request_first', 'cpu-demand-review'),
        ('nat_first_indirect', 'cpu-failure-order'),
        ('nat_first_bound', 'cpu-failure-order'),
        ('word_live_tail', 'cpu-demand-review'),
    ]
    rows = []
    for name, folder in cases:
        directory = output / name
        directory.mkdir()
        source = directory / 'main.bend'
        shutil.copyfile(PORT / 'tests/fixtures' / folder / (name + '.bend'), source)
        row = dict(fixture=name + '.bend', origin='authored_control', source_sha256=digest(source),
                   reference=PIN, port_revision=revision, port_dirty=dirty,
                   port_binary_sha256=digest(binary), timeout_seconds=60, repetitions=2,
                   compiler_version=compiler_version, generated_sha256={},
                   c_arguments=['--threads', '1', '--gpu', 'off'], steps={})
        rows.append(row)

        def run(label, command, cwd=directory, phase='execution'):
            command = list(map(str, command))
            start = time.monotonic()
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
            row['steps'][label] = dict(command=command, cwd=str(cwd), phase=phase,
                                      exit=child.returncode, timed_out=timed_out,
                                      elapsed_seconds=time.monotonic() - start,
                                      stdout_base64=base64.b64encode(stdout).decode(),
                                      stderr_base64=base64.b64encode(stderr).decode())
            (directory / (label + '.stdout')).write_bytes(stdout)
            (directory / (label + '.stderr')).write_bytes(stderr)
            (output / 'receipt.json').write_text(json.dumps(rows, indent=2) + '\n')
            return child.returncode == 0 and not timed_out

        for implementation in ['upstream', 'port']:
            for target in ['c', 'js']:
                generated = directory / (implementation + '.' + target)
                command = [node, 'main.ts', source, '-o', generated] if implementation == 'upstream' else [
                    binary, 'compile', '--executable', source, '--target',
                    'c' if target == 'c' else 'javascript', '--output', generated]
                cwd = reference / 'bend2' if implementation == 'upstream' else PORT
                if not run(f'{implementation}-generate-{target}', command, cwd, 'generation'):
                    continue
                row['generated_sha256'][implementation + '.' + target] = digest(generated)
                if target == 'js':
                    for repeat in [1, 2]:
                        run(f'{implementation}-js-{repeat}', [node, generated])
                    continue
                profiles = ['O0', 'O1', 'O2', 'O3'] if implementation == 'upstream' else ['strict']
                for profile in profiles:
                    executable = directory / f'{implementation}-{profile}'
                    flags = ['-std=c11', '-' + profile] if implementation == 'upstream' else [
                        '-std=c11', '-Wall', '-Wextra', '-Werror', '-pedantic']
                    # O3 follows pinned main.ts cli_build's Linux CPU flags;
                    # the other upstream profiles change only optimization.
                    if run(f'{implementation}-compile-{profile}', [args.compiler, *flags, generated,
                            '-lpthread', '-lm', '-o', executable], phase='compilation'):
                        for repeat in [1, 2]:
                            run(f'{implementation}-c-{profile}-{repeat}',
                                [executable, '--threads', '1', '--gpu', 'off'])
        for repeat in [1, 2]:
            run(f'native-{repeat}', [binary, 'run', source])
        row['deterministic'] = all(
            all(step[key] == row['steps'][label[:-1] + '2'][key]
                for key in ['exit', 'stdout_base64', 'stderr_base64', 'timed_out'])
            for label, step in row['steps'].items()
            if step['phase'] == 'execution' and label.endswith('-1'))
        (output / 'receipt.json').write_text(json.dumps(rows, indent=2) + '\n')
        print(name, {label: (step['exit'], base64.b64decode(step['stdout_base64']).decode(),
                            base64.b64decode(step['stderr_base64']).decode())
                     for label, step in row['steps'].items() if step['phase'] == 'execution'}, flush=True)
    # Research completion is not parity acceptance. Every raw discrepancy and
    # unavailable execution remains in the receipt, including compiler failures.


if __name__ == '__main__':
    main()
