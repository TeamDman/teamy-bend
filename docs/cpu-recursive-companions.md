# Recursive companion normalization checkpoint

Historical checkpoint follows. The later [Base-function qualification](cpu-base-function-companions.md)
completes the three next cases proposed here; these receipts remain at `6e5bd61`.

Implementation: `6e5bd61b5f2cf7e1e4de7c398f17cfe04a0150cb`, local branch
`qualify/cpu-recursive-companions-20261023`, based on `d74f6a6`.
Target remains [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
The approved-demand implementation and prior checkpoints remain intact.
Dependencies, lockfile, tools, reference/docs sources and safety limits are
unchanged; nothing was pushed.

## Three originals qualified, one presentation correction

The unchanged upstream `_002_sel`, `_003_flip` and `_004_sel` companions now
match both normalized structure and raw text twice. Each exits 0 with empty
stderr; `run` emits the following term followed by one LF, and `eval` retains
its Facet envelope with that exact decoded value:

| Fixture under `flatten/` | Exact upstream and port normalized term |
| --- | --- |
| `computed_recursion_002_sel.bend` | `\{S: \{Z: Z{}; (_3 => S{Z{}})}; (r => S{Z{}})}` |
| `computed_recursion_003_flip.bend` | `\{T: F{}; F: T{}}` |
| `computed_recursion_004_sel.bend` | `\{Z: S{Z{}}; (r => Z{})}` |

The prior binary already agreed structurally but printed `_5` instead of `_3`
in `_002_sel`. Pinned [parse_book](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2532)
resets its fresh counter per declaration;
[parse_var/parse_open](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1612)
count binder openings and unresolved simple names, and
[match_flatten](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2719)
uses that counter in generated pattern names. The port had used its globally
unique semantic identity counter for these displayed names.

The parser now tracks local display numbering separately. Semantic binder IDs
remain globally unique; substitution, binder scope, normalization and resource
policies are unchanged. Regressions cover nested/default applications at zero,
one and two constructors, Boolean flipping, and an unrelated preceding datatype
and function that must not change the displayed name or collide semantically.
The structural comparator remains unchanged. Original fixture bytes and Apache
attribution are retained. NoBase function mains are interpreter lanes under the
pinned gate; compiled passes are not claimed for these three originals.

## Validation

- Standard gate: **801 passed, 0 failed, 49 ignored**, exit 0; formatting, strict
  Clippy, all-feature build and real Node provider also pass.
- Strict all-target/all-feature Clippy: exit 0. Focused parser/template/normalization
  tests: **50 passed**. The initial Clippy function-length finding was fixed by
  extracting the existing flatten/error wrapper, without suppressing a lint.
- Full normalization matrix: **16 cases**, twelve original and four authored,
  exact structure/text twice. **132 commands**: 128 successful observations and
  four expected upstream function-printer refusals. No timeout or byte instability.
- Positive matrix: **308/320 exact executions**, twelve known Bun-host failures,
  32 fixtures twice; runner exit 0 with blocked cells explicitly retained.
- Negative matrix: **44/56 exact executions**, eight resource-policy differences,
  two native-foreign gaps and two Bun-host failures, six fixtures twice; required
  parity runner exit **1**. No new failure category or timeout occurred.
- The approved demand matrix was renewed: **98 executions**, 90 exact, six
  accepted upstream-C wildcard exceptions, two unresolved competing-error
  diagnostics; two Word executions unavailable after one C compilation failure.
  Its required-parity runner exits **1**. The
  [separate ordering/profile experiment](cpu-failure-order.md) explains that
  failure and establishes exact Word behavior under upstream's O3 build profile.

The full gate was run on the source content then committed as `6e5bd61`; clean
commit CLI/probe rebuilds generated the differential receipts. Current positive
and negative matrix renewals and hashes are recorded below. Older comptime,
multiworker, direct-CLI, checking and Windows/macOS receipts retain their actual
source commits; this checkpoint does not relabel them as new executions. The
existing downstream `KernelError` private-field and `run_main` `Ok(1)` API caveat
remains unresolved; no new public API signature change is introduced here.

Commands used with the activated installed tools and exported
`CARGO_NET_OFFLINE=true`, `BEND_GPU=off`:

```sh
cargo test --features extended_observability --test kernel_normalization --test parser --test parser_templates
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-features --bin teamy-bend --example compare_normalization
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <reference> -OutputDirectory <fresh-normalization-dir> -Suite all -Repetitions 2 -RequireExactText
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <reference> -OutputDirectory <fresh-demand-dir> -Suite demand -Repetitions 2
python3 scripts/compare-failure-order.py --reference <reference> --output <fresh-ordering-dir>
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <reference> -OutputDirectory <fresh-positive-dir> -Suite all -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <reference> -OutputDirectory <fresh-negative-dir> -Repetitions 2
```

Candidate binary SHA-256:
`4c4d603c91a3b260a3fa88bffccb4320c0a939937f528d7f4bd09fa781492863`;
normalization probe:
`b3b74b3c17fbf464bb89ef01e57b37b25795b5a071d60f9b9816bd26e2de3c4d`.
Normalization receipt:
`1a59bf3b053dd9601e27e95f0079463c2947c6dcc20ad02918f1c9b99c331bbc`.
Demand receipt:
`5a6159c2a51744339b2b90d1057b632eddc397dc59f31abba2aa7748cfb2ecbc`.
Positive receipt:
`dc8f03d2677643e2ae4debc4959bac736d194a77b0249fa05b00b7bd3cc8e1ca`.
Negative receipt:
`f97615cbd79a09e861bccab61d3b67cf6a9566373aac17ae22446138eaeaf4a1`.
Private `cpu-recursive-companions` evidence retains preflight, build/test logs,
all receipts, binary snapshots and exact generated programs. Existing evidence
was preserved; disk headroom was checked before builds.

## Remaining acceptance work

The [acceptance criterion](cpu-parity-acceptance.md) is unchanged. Runtime and
interpretation ledgers now link **63 originals**: 51 runtime and twelve
interpretation rows, not necessarily all their applicable lanes. **1,239** lack
either receipt kind: 439 checker negatives, twelve check-only modules, 153
interpreter-only cases and 635 compiled candidates. A separate checking ledger
still has one exact acceptance and three phase/construct agreements with raw
diagnostic differences. Authored controls add no inventory rows.

Competing-error diagnostic selection remains an unresolved cross-oracle contract,
not part of the approved wildcard exception. Bun-host lanes, native foreign
support, explicit resource-policy failures, broader corpus/API coverage and
host/multiworker qualification remain required. GPU checks were not run and
are not a prerequisite for independent CPU progress.

The next independent bounded group is `computed_recursion_001_main_dec`,
`_001_not` and `_001_par`: Base-importing function results with nested decisions
and recursion. They need interpreter structure/text qualification and actual
upstream printer applicability checks. Their Base imports alone cannot establish
that compiled printers support them. No new results are claimed for that group.
