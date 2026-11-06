# SPDX-License-Identifier: MPL-2.0
"""Compare the bounded glibc NaN payload witness against pinned Bend C and JS."""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import signal
import subprocess
import sys
import time

PIN = 'e6676b080f25b1bc1bf5b5b7d7a17e22f8022599'
ROOT = Path(__file__).resolve().parent.parent


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reference', type=Path, required=True)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--compiler', default=os.environ.get('TEAMY_BEND_CC', 'clang'))
    args = parser.parse_args()
    reference = args.reference.resolve()
    binary = args.binary.resolve()
    output = args.output.resolve()
    if sys.platform != 'linux' or platform.machine() != 'x86_64' or platform.libc_ver()[0] != 'glibc':
        raise ValueError('This witness qualifies Linux x86-64 glibc only')
    git = lambda root, *parts: subprocess.check_output(
        ['git', '-C', str(root), *parts], text=True, timeout=60).strip()
    if git(reference, 'rev-parse', 'HEAD') != PIN or git(reference, 'status', '--porcelain=v1'):
        raise ValueError('Reference must be clean at the pin')
    if git(ROOT, 'status', '--porcelain=v1'):
        raise ValueError('Port source must be clean')
    if output.exists() or output.is_relative_to(reference) or not binary.is_file():
        raise ValueError('Choose a fresh output outside the reference and an existing binary')
    if shutil.disk_usage(ROOT).free < 10 * 2**30 or shutil.disk_usage('/tmp').free < 2**30:
        raise ValueError('Active disk floor not met')
    output.mkdir(parents=True)
    fixture = ROOT / 'tests/fixtures/cpu-float-text/nan-payload-overflow.bend'
    result = dict(source_commit=git(ROOT, 'rev-parse', 'HEAD'), reference=PIN,
                  binary_sha256=digest(binary), fixture_sha256=digest(fixture),
                  platform='Linux x86-64 glibc', libc_version=platform.libc_ver()[1],
                  observations=[], completed=False, cross_backend_split=True)
    receipt = output / 'receipt.json'

    def save():
        receipt.write_text(json.dumps(result, indent=2) + '\n')

    def run(lane, command, cwd):
        if shutil.disk_usage(ROOT).free < 10 * 2**30 or shutil.disk_usage('/tmp').free < 2**30:
            raise RuntimeError('Active disk floor crossed')
        started = time.monotonic()
        child = subprocess.Popen(list(map(str, command)), cwd=cwd, stdin=subprocess.DEVNULL,
                                 stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                 env={**os.environ, 'BEND_GPU': 'off'}, start_new_session=True)
        try:
            stdout, stderr = child.communicate(timeout=60)
            timed_out = False
        except subprocess.TimeoutExpired:
            os.killpg(child.pid, signal.SIGKILL)
            stdout, stderr = child.communicate()
            timed_out = True
        row = dict(lane=lane, exit=child.returncode, timed_out=timed_out,
                   stdout_base64=base64.b64encode(stdout).decode(),
                   stderr_base64=base64.b64encode(stderr).decode(),
                   seconds=time.monotonic()-started)
        result['observations'].append(row)
        save()
        if timed_out:
            raise RuntimeError('Child timed out: ' + lane)
        return child.returncode, stdout, stderr

    save()
    programs = {'native': [binary, 'run', fixture]}
    for side in ['upstream', 'port']:
        for target in ['c', 'js']:
            artifact = output / (side + '.' + target)
            command = (['node', reference / 'bend2/main.ts', fixture, '-o', artifact]
                       if side == 'upstream' else
                       [binary, 'compile', fixture, '--executable', '--target',
                        'c' if target == 'c' else 'javascript', '--output', artifact])
            status, stdout, stderr = run(side + '-generate-' + target, command,
                                         reference / 'bend2' if side == 'upstream' else ROOT)
            if status != 0 or stderr or not artifact.is_file():
                raise RuntimeError('Generation failed: ' + side + '-' + target)
            if side == 'port':
                metadata = json.loads(stdout)
                expected_target = 'c' if target == 'c' else 'javascript'
                provider = metadata['socket_provider']
                if (metadata['output'] != str(artifact)
                        or metadata['target'] != expected_target
                        or metadata['bytes'] != artifact.stat().st_size
                        or metadata['gpu_artifact'] is not None
                        or (target == 'c' and provider is not None)
                        or (target == 'js' and
                            (provider != str(output / 'teamy-bend-sys.node')
                             or not Path(provider).is_file()))):
                    raise RuntimeError('Generated artifact metadata mismatch')
            elif stdout:
                raise RuntimeError('Unexpected upstream generation output')
            result[side + '_' + target + '_sha256'] = digest(artifact)
            if target == 'c':
                executable = output / (side + '-c')
                flags = [] if side == 'upstream' else ['-Wall', '-Wextra', '-Werror', '-pedantic']
                libraries = ['-lpthread', '-lm'] if side == 'upstream' else ['-pthread', '-lm', '-ldl']
                built = run(side + '-compile-c', [args.compiler, '-std=c11', '-O3', *flags,
                            artifact, *libraries, '-o', executable], ROOT)
                if built != (0, b'', b''):
                    raise RuntimeError('C compilation failed: ' + side)
                programs[side + '-C'] = [executable, '--threads', '1', '--gpu', 'off']
            else:
                programs[side + '-JS'] = ['node', artifact]
    expected_c = b'2147483647\n2147483647\n4294967295\n2143289344\n'
    expected_js = b'none\nnone\nnone\nnone\n'
    for lane, command in programs.items():
        expected = expected_js if lane.endswith('-JS') else expected_c
        for repeat in [1, 2]:
            if run(lane + '-' + str(repeat), command, ROOT) != (0, expected, b''):
                raise RuntimeError('Mismatch: ' + lane)
    if digest(binary) != result['binary_sha256'] or git(ROOT, 'status', '--porcelain=v1'):
        raise RuntimeError('Binary or source changed during comparison')
    result.update(completed=True, executions=10, timeouts=0,
                  qualification='Native and C match each other; JS lanes match each other and reject these spellings. Cross-backend behavior remains different.')
    save()
    print(json.dumps({'completed': True, 'observations': len(result['observations']),
                      'executions': result['executions'], 'cross_backend_split': True}))


if __name__ == '__main__':
    main()
