# SPDX-License-Identifier: MPL-2.0
"""Classify pinned fixture applicability; never execute Bend or waive a lane."""
import argparse
import collections
import csv
import hashlib
import json
from pathlib import Path
import re
import subprocess

PIN = 'e6676b080f25b1bc1bf5b5b7d7a17e22f8022599'
ROOT = Path(__file__).resolve().parent.parent


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_csv(path):
    with path.open(newline='', encoding='utf-8-sig') as stream:
        return list(csv.DictReader(stream))


def require(condition, message):
    if not condition:
        raise ValueError(message)


def code_only(source):
    # Preserve line breaks while excluding strings and line comments from API
    # reference hints. Hints do not prove reachability or supply lane waivers.
    return re.sub(r'"(?:\\.|[^"\\])*"|#[^\r\n]*', lambda m: '\n' * m[0].count('\n'), source)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reference', type=Path, required=True)
    parser.add_argument('--baseline-ledger', type=Path, required=True)
    parser.add_argument('--audit', type=Path, required=True)
    parser.add_argument('--ledger', type=Path, default=ROOT / 'docs/cpu-runtime-evidence.csv')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    reference = args.reference.resolve()
    tests = reference / 'tests'
    git = lambda *parts: subprocess.check_output(['git', '-C', str(reference), *parts], text=True).strip()
    require(git('rev-parse', 'HEAD') == PIN and not git('status', '--porcelain=v1'), 'Reference must be clean at the pin')
    output = args.output.resolve()
    require(not output.exists() and not output.is_relative_to(reference), 'Choose a fresh output outside the reference')
    inventory_path = ROOT / 'docs/cpu-parity-inventory.csv'
    inventory = read_csv(inventory_path)
    metadata = json.loads((ROOT / 'docs/cpu-parity-inventory.json').read_text())
    require(metadata['upstream'] == PIN and metadata['inventory_sha256'] == digest(inventory_path)
            and metadata['upstream_gate_sha256'] == digest(reference / 'gates/test.ts'),
            'Inventory provenance changed')
    baseline = {r['fixture']: r for r in read_csv(args.baseline_ledger)}
    ledger = {r['fixture']: r for r in read_csv(args.ledger)}
    names = {r['fixture'] for r in inventory}
    require(len(names) == len(inventory) == 1302, 'Unexpected inventory')
    require(names == {str(p.relative_to(tests)) for p in tests.rglob('*.bend')}, 'Incomplete inventory')
    require(set(baseline) <= set(ledger) <= names, 'Ledger must preserve baseline fixture rows')
    require(len(baseline) == len(read_csv(args.baseline_ledger)) and len(ledger) == len(read_csv(args.ledger)),
            'Duplicate ledger rows')
    audit = json.loads(args.audit.read_text())
    totals = collections.Counter()
    for counts in audit['categories'].values():
        totals.update(counts)
    require(totals['fixtures'] == len(inventory) and totals['abnormal_exit'] == 0, 'Incomplete or abnormal strict audit')
    require(totals['expected_failure_checked'] == 0, 'Audit acceptance requires explicit review')
    findings = {r['fixture']: r for r in audit['findings']}
    require(len(findings) == totals['expected_success_rejected'], 'Unexpected audit findings')
    require(all(r['outcome'] == 'expected-success-rejected' for r in findings.values()), 'Unexpected audit outcome')
    # Literal include_str sources only. An inlined/adapted workload, a filename
    # mention or a related test name is not exact upstream-fixture coverage.
    exact_tests = collections.defaultdict(set)
    for rust in sorted((ROOT / 'tests').rglob('*.rs')):
        for name in re.findall(r'include_str!\(\s*"([^"]+)"\s*\)', rust.read_text()):
            source = rust.parent / name
            if source.is_file() and source.suffix == '.bend':
                exact_tests[digest(source)].add(str(rust.relative_to(ROOT)))
    rows = []
    for item in inventory:
        fixture = item['fixture']
        path = tests / fixture
        source = path.read_text()
        require(digest(path) == item['source_sha256'], 'Source hash changed: ' + fixture)
        expected = '\n'.join(re.findall(r'^#\|([^\r\n]*)', source, re.M))
        require(hashlib.sha256(expected.encode()).hexdigest() == item['expectation_sha256'], 'Expectation changed: ' + fixture)
        for linked in [baseline, ledger]:
            if fixture in linked:
                require(linked[fixture]['source_sha256'] == item['source_sha256'], 'Ledger hash changed: ' + fixture)
        rejected = item['expects_check_error'] == 'True'
        runnable = item['has_main'] == 'True' and not rejected
        compiled = item['candidate_js'] == 'True' or item['candidate_c'] == 'True'
        category = ('checker_rejection' if rejected else 'checking_without_main' if not runnable
                    else 'interpretation_and_compiled_candidates' if compiled else 'interpretation_only')
        code = code_only(source)
        refs = set(re.findall(r'\b(?:IO|File|TCP|UDP|Listener|Socket|Window|Audio|App)\.[A-Za-z_][\w.]*', code))
        foreign = bool(re.search(r'^\s*import "[^"\r\n]+\.(?:js|c)"', source, re.M))
        imports = []
        for target in re.findall(r'^import\s+([^\s#]+)', source, re.M):
            if target != 'Base':
                # At this pin, one runnable fixture imports another module.
                dependency = (path.parent / target).resolve()
                require((dependency.is_relative_to(tests) and dependency.is_file()) or not runnable,
                        'Runnable import needs review: ' + fixture)
                if runnable:
                    imports.append(str(dependency.relative_to(tests)))
                    foreign |= bool(re.search(r'^\s*import "[^"\r\n]+\.(?:js|c)"', dependency.read_text(), re.M))
        profiles = []
        if runnable:
            for prefix, profile in [('Window.', 'window_surface'), ('Audio.', 'audio_surface'),
                                    ('File.', 'filesystem'), ('TCP.', 'network'), ('UDP.', 'network'),
                                    ('Listener.', 'network'), ('Socket.', 'network')]:
                if any(ref.startswith(prefix) for ref in refs):
                    profiles.append(profile)
            if 'IO.get_env' in refs:
                profiles.append('environment')
            if refs & {'IO.sleep', 'IO.now'}:
                profiles.append('clock_or_timer')
            if foreign:
                profiles.append('foreign_companion')
            if 'App.run' in refs:
                profiles.append('app_device_review')
        status = ('positive_refused' if fixture in findings else 'nonzero_expectation_refused'
                  if any(code != '0' for code in item['exit_marker'].split(';') if code) else 'accepted')
        rows.append(dict(fixture=fixture, source_sha256=item['source_sha256'], category=category,
                         required_check='reject' if rejected else 'accept',
                         required_interpretation=runnable, candidate_c=item['candidate_c'], candidate_js=item['candidate_js'],
                         runtime_exit_marker=item['exit_marker'] if runnable else '',
                         profile_references=';'.join(sorted(set(profiles))) if runnable else 'not_executed_by_gate',
                         api_references=';'.join(sorted(refs)), imported_fixture=';'.join(imports),
                         cpu_offload_review=runnable and '!(' in code,
                         baseline_runtime_link=fixture in baseline, current_runtime_link=fixture in ledger,
                         strict_audit_status_only=status,
                         exact_fixture_gate_sources=';'.join(sorted(exact_tests[item['source_sha256']]))))
    output.mkdir(parents=True)
    with (output / 'cpu-applicability.csv').open('w', newline='') as stream:
        writer = csv.DictWriter(stream, fieldnames=list(rows[0]), lineterminator='\n')
        writer.writeheader()
        writer.writerows(rows)
    def counts(selected):
        return dict(rows=len(selected), categories=dict(collections.Counter(r['category'] for r in selected)),
                    profiles=dict(collections.Counter(p for r in selected for p in r['profile_references'].split(';') if p and p != 'not_executed_by_gate')),
                    cpu_offload_review=sum(r['cpu_offload_review'] for r in selected),
                    strict_audit_status_only=dict(collections.Counter(r['strict_audit_status_only'] for r in selected)),
                    exact_fixture_gate_references=sum(bool(r['exact_fixture_gate_sources']) for r in selected),
                    examples_or_other=0)
    summary = dict(schema=1, upstream=PIN, inventory_sha256=digest(inventory_path),
                   baseline_ledger_sha256=digest(args.baseline_ledger), current_ledger_sha256=digest(args.ledger),
                   strict_audit_sha256=digest(args.audit), upstream_gate_sha256=digest(reference / 'gates/test.ts'),
                   all=counts(rows), baseline_unlinked=counts([r for r in rows if not r['baseline_runtime_link']]),
                   current_unlinked=counts([r for r in rows if not r['current_runtime_link']]),
                   qualification='Applicability and existing evidence only. Every fixture is a gate-selected test, not an example waiver. Profiles are source references, not reachability or platform qualification. Strict audit is proof-mode status only, never exact diagnostic/executable parity. Literal include_str references are test coverage, not differential qualification.')
    (output / 'cpu-applicability.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    main()
