# List slicing and local pipelines

Tested clean source: `fff765af30bf3ee4bf4b5632a2c73962917586fa`. No production change was necessary. Both original
fixtures already execute correctly; this checkpoint adds durable edge coverage.

| Program | Observed stdout | Five executable lanes, twice each |
| --- | --- | --- |
| base/list_slice | 22 original output lines | exact |
| base/list_ops | `12\n` | exact |
| slicing edge companion | 16 output lines | exact |
| pipeline edge companion | 16 output lines | exact |

All **40 executions** match complete stdout, empty stderr and exit 0 in native
Rust, port C/JS and pinned upstream C/JS. There are no timeouts. The originals are
byte-identical to upstream e6676b080f25b1bc1bf5b5b7d7a17e22f8022599 and retain
Apache-2.0 attribution. [Fixtures, goldens and scope](../tests/fixtures/cpu-list-slice-pipelines/README.md)
and [per-case evidence](cpu-list-slice-pipelines-evidence.csv) are preserved in Git.

The [slicing original](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/base/list_slice.bend)
exercises bundled list length, append/concat/reverse, head/tail/last/get/set,
take/drop, zip, range and replicate. Its companion adds empty lists, zero counts,
updates to empty lists and first positions, and unequal zip lengths. The
[pipeline original](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/base/list_ops.bend)
defines local first-order recursive workers. Its companion checks empty/default
results, odd filtering, out-of-range updates, short zip, counts and empty any/all.
These are concrete fixture-local behaviors, not universal higher-order List claims.

## Applicability and validation

The main matrix contains **88 observations**: 40 exact executions, 16 generations,
eight C builds, eight direct-Node host failures and 16 strict check/eval refusals.
Direct upstream Node fails with `TypeError: require is not a function`; generated
JS succeeds. The separate **24 phase probes** show upstream acceptance and port
executable acceptance, while strict proof checking refuses IO. These are IO mains:
no strict-proof or normal-form qualification is inferred from execution. Exact
[before/current failure streams](cpu-list-slice-pipelines-failures.json) remain
separate. Both originals' pre-checkpoint native outputs already matched.

The clean-source standard gate passes **1,169 / 0 / 49**. It includes format,
production strict Clippy, all-feature build, Node provider and all repository tests,
including the new list integration test and earlier stack/sharing, aggregate work,
literal-pattern, deep-input and cancellation controls. The new integration test's
targeted strict Clippy also passes. Its preliminary focused run passed one test
before the clean commit; that dirty-source receipt is retained separately, not
reported as the full gate. No additional broad repeat was needed after the gate.

Independent review found no blocker. The 49 ignored hardware/profiler cases remain
unrun; historical supplementary all-target Clippy failures are not cleared. No
dependencies, tools, upstream references, resource limits, demand or cancellation
contracts changed. No desktop/GPU work, main merge or release occurred.

## Remaining parity and next substantive family

Full CPU parity remains incomplete. Coverage is **422 linked / 880 unlinked**:
525 interpreter-and-compiled candidates, 341 checker negatives, 13 interpreter
profiles and one checking-only module remain unlinked. A link is not universal
qualification. The full checking census was not rerun: its last measurement at
`d224e79d33b4a0ab26f701b57f448056b63b790a` remains **300 differences beyond 385
historical IO exclusions plus six upstream stack failures**. Production-tree
identity is verified; these historical counts are not re-dated as this run.
The historical residual triage is 149 diagnostic candidates / 109 acceptance /
42 other. F32.read, competing failure order, erased-hole/TODO, host/foreign and
printer/oracle/resource disagreements, external API caveats and the incomplete
688-vector exploration remain unwaived.

The [string-pattern design](cpu-compact-patterns.md) remains blocked on a complete
compact-lowering/normalization design. Its original still refuses at load; list
controls do not close it. The next independent substantive candidate is
[`base/read_bounds.bend`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/base/read_bounds.bend):
Nat48/U32 lexical bounds, overflow rejection and empty-path File.open error mapping.
It was source-inspected, **not executed in this checkpoint**. No mismatch or new
numeric parsing policy is asserted in advance.

## Reproduction and preservation

Activate the provisioned environment, then run:

```bash
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test cpu_list_slice_pipelines
cargo clippy -p teamy-bend --locked --all-features --test cpu_list_slice_pipelines -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Host debug/incremental settings control build-cache size, not Bend optimization.
For every original/positive file, set `fixture`, `reference` (the pinned checkout)
and a fresh `artifacts` directory; repeat executable commands twice:

```bash
teamy-bend run "$fixture"
teamy-bend check "$fixture"
teamy-bend eval "$fixture"
node "$reference/bend2/main.ts" "$fixture"
node "$reference/bend2/main.ts" "$fixture" -o "$artifacts/upstream.c"
node "$reference/bend2/main.ts" "$fixture" -o "$artifacts/upstream.js"
"$TEAMY_BEND_CC" -std=c11 -O3 "$artifacts/upstream.c" -lpthread -lm -o "$artifacts/upstream"
teamy-bend compile "$fixture" --executable --target c --output "$artifacts/port.c"
teamy-bend compile "$fixture" --executable --target javascript --output "$artifacts/port.js"
"$TEAMY_BEND_CC" -std=c11 -O3 -Wall -Wextra -Werror -pedantic "$artifacts/port.c" -pthread -lm -ldl -o "$artifacts/port"
"$artifacts/upstream" --threads 1 --gpu off
"$artifacts/port" --threads 1 --gpu off
node "$artifacts/upstream.js"
node "$artifacts/port.js"
```

Matrix children use 30-second bounds; the integration harness uses strict
unoptimized C and 20-second generated-runtime bounds. Repeated invocation of one
checked Book verifies isolation, not allocator reclamation or mid-normalization
cancellation. [Validation](cpu-compact-patterns-validation.json) records exact
commands, hashes, frozen inputs and raw receipt identities.

Disk preflight started at 11.934 GiB free; minimum workspace
headroom was 11.908 GiB, above the unchanged 10 GiB stop floor.
The prior verified relocation of rebuildable dependency artifacts to temporary
storage was retained; minimum temporary headroom was 5.081 GiB.
No unique evidence was deleted. Git preserves tests, research source and compact
sanitized outcomes. Bulk raw logs, generated artifacts, private phase runners and
binary archives remain local and are **not preserved by pushing Git**.
