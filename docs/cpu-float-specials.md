# Float comparisons, special values and an open checker resource gap

Historical checkpoint: the later [checker-depth fix](cpu-checker-depth.md),
`2f349c8`, now qualifies the unchanged combined program with the same limits.
The findings and receipt identities below describe `9b18fbb`.

Coverage source `9b18fbbede02e8653b99644db881699470d251b1`, local branch
`qualify/cpu-float-specials-20261023`, based on `49a37bc`.
Target: [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
The requested originals and a factored observation control match five primary
runtimes twice. No numeric runtime defect was demonstrated. A valid combined
version of the control exposes an unresolved port checker resource failure;
its source and failed cells are retained. F32.read disagreements also remain
unresolved. No production, parsing-policy, budget, dependency, toolchain, pin or
approved-demand change. No new exception, installation or push.

## Original results and applicability

| Exact original | Exact stdout |
| --- | --- |
| [run/float_text_roundtrip.bend](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/run/float_text_roundtrip.bend) | `233\n` |
| [compile/float_compare.bend](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/compile/float_compare.bend) | `1999985\n` |
| [run/float_specials.bend](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/run/float_specials.bend) | `12\n` |

All have Base/IO mains. Upstream C/JS generation actually succeeds. Native Rust,
port C, port JS, upstream C and emitted upstream JS produce the exact output,
empty stderr and exit 0 twice: **30 exact primary executions**. Original source
bytes, comments and Apache-2.0 attribution remain unchanged.

Despite its name, float_text_roundtrip performs arithmetic and to_u32, not
F32.read. Its three results are 3, 3 and 2, weighted into 233. float_compare
performs twenty rounded arithmetic steps, scales/truncates the result and adds
five successful comparisons. float_specials sums twelve Boolean checks involving
NaN, infinities, signed zero and bounded U32 conversions. That digest establishes
those Boolean outcomes; it does not compare NaN payload bits.

All three direct upstream IO entries remain host-blocked twice: exit 1, empty
stdout, `TypeError: require is not a function\n`. The pinned
[driver](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1271)
uses import.meta.require, absent on this Node host. Emitted CommonJS succeeds.
These six direct failures are not erased by generated-JS agreement. Strict proof
checking remains a separate lane from execution of axiomatic F32 operations.

## Exact observations, without a NaN payload claim

The factored authored control adds ten exact executions with seventeen stdout
lines, exit 0 and empty stderr. Selected results print bits and text together:

| Observation | Binary32 bits | Text |
| --- | --- | --- |
| 1.5 * 2; 0.25 + 2.75 (two lines) | 1077936128 | 3 |
| 10 / 4 | 1075838976 | 2.5 |
| Twenty-step arithmetic result | 1073741660 | 1.9999804 |
| Result multiplied by 1000000 | 1240736612 | 1999980.5 |
| 1 / 0 | 2139095040 | inf |
| -1 / 0 | 4286578688 | -inf |
| 0 * -1 | 2147483648 | -0 |

Separate integer lines are `2\n` for truncating 2.5 and `5\n` for the comparison
count. NaN prints `nan\n`; its payload/sign bits are **not read**. The following
rows print six Boolean digits in order: eq, ne, lt, le, gt, ge.

| Operands | Exact predicate row |
| --- | --- |
| NaN, NaN | `0 1 0 0 0 0\n` |
| NaN, 1 | `0 1 0 0 0 0\n` |
| +infinity, 1 | `0 1 0 0 1 1\n` |
| -infinity, 1 | `0 1 1 1 0 0\n` |
| -0, +0 | `1 0 0 1 0 1\n` |

The final conversion row is `0 0 0 0 0 0 1 4294967040 0\n`, corresponding to
NaN, +infinity, -infinity, -2.5, -0, 0.5, 1, the largest binary32 integer below
2^32, and U32.to_f32(4294967295), which rounds to 2^32. Numeric bits, rendered
text and nonfinite classification are separate observations. No tolerance or
unobserved NaN payload guarantee is introduced. The CLI regression also preserves
raw program output under `--output-format json`.

Pinned [comparison/arithmetic lowering](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L220)
uses C float operations and JS Math.fround/comparisons. The
[C U32 conversion](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L427)
guards the range before casting; [JS conversion](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L236)
uses its range check and Math.floor. Pinned
[C text](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L457)
and [JS text](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L538)
explicitly render NaN as nan. This qualifies the tested values/profile, not all
inputs, other hosts, GPU execution or arbitrary NaN representations.

## Combined-control checking failure remains open

[float-specials-combined-depth.bend](../tests/fixtures/cpu-float-specials/float-specials-combined-depth.bend)
preserves the exact valid combined control. Pinned upstream C and JS generate
and execute it twice, matching the seventeen lines above. Port C/JS generation
and native execution instead refuse it during checking, before producing any
program output, with status 1 and the diagnostic cause:

```text
main: kernel nesting limit exhausted; proof not accepted
```

The full raw CLI stderr includes the existing host error formatting and source
location. Exact bytes/hashes remain in receipts and the separate
[depth ledger](cpu-float-specials-depth-evidence.csv); a substring is not promoted
to exact diagnostic parity. No generated port executable runs. Native checking
also prevents the first IO operation. The repeated probe records four successful
upstream executions and six port checking refusals. Its runner exits **1**.

The [checker guard](../src/kernel/check.rs#L331) rejects nesting at 128. There is
no evidence here of a float arithmetic failure: factoring the same observations
into named IO helpers produces identical upstream output and passes the port.
The helper program is a distinct source; its success **does not qualify** the
combined one. This is an unresolved resource compatibility cell under the
acceptance criterion, not a new policy exception. No limit is raised, no checker
is bypassed and no production correction is claimed. A future fix should isolate
and reduce checker traversal/expansion depth while retaining fail-closed checks.

`-Suite float-specials-depth` reproduces the port-C generation failure and keeps
exit 1. Because it stops at the first failed generation, it does not itself
execute all runtimes or reach its requested repetitions. The separately retained
bounded full-depth probe visits both upstream runtimes and all three port paths
twice, with explicit phase/status/stream receipts. The positive suite includes
the factored program; this separation is documented and does not waive the
combined source. Both authored controls add no upstream inventory rows.

An initial authored spelling omitted reusable Nat binders on step/count. Both
implementations rejected that invalid control. Restoring the originals' `+n`
contract produced the valid combined reproducer above. Those preliminary
failures remain recorded; they are not port defects or passing exact diagnostic
comparisons. No original fixture was edited to obtain agreement.

## Validation and retained gaps

The clean coverage commit passes the standard Linux gate: **815 passed, zero
failed, 49 ignored**, exit 0. The 47 GPU and two optional-profiler ignores remain
unrun. All **33 focused tests**, strict all-target/all-feature Clippy and clean
build pass. The positive matrix covers 62 fixtures (53 originals, nine authored
controls), 1,040 commands: **608/620 primary executions exact**, twelve existing
Bun-FFI host failures. Forty-eight separate direct observations give four exact
and 44 import.meta.require failures. Its runner exits 0 with those classified
host limits; they are not passes. Repeats are stable, zero timeouts, longest
child 2.134 seconds.

The full-depth probe records 18 commands, four exact upstream C/JS executions,
six port checking refusals and two direct-IO host failures. Repeats are stable,
zero timeouts, longest child 0.627 seconds; runner exit 1. The checked-in
generation-only probe also exits 1. The read probe records sixteen commands,
ten successful executions and the same three cross-backend differences, with
zero timeouts and runner exit 1. Neither failed probe is hidden in the positive
matrix's pass count or treated as an accepted exception.

The renewed F32.read probe still returns **1** for the three
[read-contract disagreements](cpu-float-read-evidence.csv). Native/port C match
upstream C; port JS matches upstream JS. Decimal bits remain 1065353217 versus
1065353216; hexadecimal and embedded-NUL inputs remain Some versus None. The
final-newline input returns None everywhere. All executions exit 0 with empty
stderr. Those corresponding-backend matches are not cross-backend passes.
The [documented-contract investigation](cpu-float-arithmetic.md) still applies;
no parsing policy or additional exception is chosen.

Unchanged production means negative/demand/normalization/comptime/audit matrices
retain `bd6a228`, without rerunning or relabelling them. Negative 44/56 exact
retains eight resource-policy, two native-foreign and two Bun-host gaps; demand
92/100 exact retains six approved wildcard exceptions and two unwaived Nat-first
error-order differences. Comptime 156/160 primary exact retains four default-JS
stack failures plus four exact stack-4096 supplements. Eighteen normalization
originals retain exact structure/text agreement. The strict audit remains 366
accepted positives, 487 refused, all 439 checker negatives rejected. Historical
profiles and downstream KernelError/run_main API caveats remain intact.

## Reproduction and provenance

Activate the existing environment and use `CARGO_NET_OFFLINE=true BEND_GPU=off`:

```sh
cargo test --features extended_observability --test cpu_float_specials --test cpu_float_arithmetic --test cpu_float_text --test cpu_float_math --test cpu_float_runtime --test cpu_parse_floats --test runtime_numeric --test parser_numeric
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-features --bin teamy-bend
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference "$REFERENCE" -OutputDirectory "$EVIDENCE/positive" -Suite all -Repetitions 2 -OracleOptimization O3
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference "$REFERENCE" -OutputDirectory "$EVIDENCE/depth" -Suite float-specials-depth -Repetitions 2 -OracleOptimization O3
python3 scripts/compare-float-read.py --reference "$REFERENCE" --output "$EVIDENCE/read"
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

REFERENCE names the clean pin; EVIDENCE is a fresh private directory. Focus the
positive family with `-Suite float-specials`. Upstream C retains pinned Linux
O3/C11/link flags and `--threads 1 --gpu off`; port C keeps strict warnings.
Rust 1.96.0, nightly-2026-05-24 formatting, Clang 19.1.7 and Node 24.19.0 are
unchanged. Every comparison child has a 60-second deadline. Exact argv, stream
bytes, statuses, identities and timings remain in private receipts.

Preflight/control exploration used the preserved all-feature `8ad7f43` binary;
the dirty positive candidate used its gate-feature binary. Final comparisons
snapshot clean `9b18fbb` before the full gate; no other Cargo build, source edit
or commit overlaps the gate. Failed preliminary controls, raw logs and all old
receipts remain. Only completed root test executables were reclaimed; identical
immutable snapshots were deduplicated with every path/hash retained.

Private evidence SHA256 hashes:

| Receipt | SHA256 |
| --- | --- |
| preflight | `0914591ec83912ecc0388ee12524a89c7660459fb858b676881d1725a940fd9c` |
| control-preflight | `dacd3ad8a255b008fb3854bbb465b85afc1180dee7ef0e43b45a6bca725ab1d1` |
| control-candidate | `edd6b15798e0514f6155cf195d3b5e91aca5a21d7d005f8654c5ac54a17cf227` |
| control-factored | `325a2dda7335cafe259436e3ed956dda72f1e9adfb38eb91279e2594aeb904d4` |
| candidate | `238780694d2d3c3c65092401c8b887c5d8c18d9bb92d4cd3390b81b068628b61` |
| positive | `c9dc0ad0f49ebcd1ee45a9a40f29b5ff5bf189993cfcb19e26abc15e7231b64e` |
| depth | `9824036eb74ef190f80c2000c5d77f71170f40c0f0eeddc408542ab05ff2e284` |
| depth-full | `258fad0471291f12fe22cc8f8b52e699d4b4a32381f1801c7c06f2e06d0940cd` |
| read | `3be1903ddc4c1e2f0b7667e559074cea4199f2bc94604bede0c598d810430443` |
| Clean all-feature binary | `1149df32aee8d016f208fc13da2fac02caa621e101ecc6729727597bd832ffdf` |
| Unchanged Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |

## Acceptance and next work

Runtime evidence links **75 originals**: 53 positive originals renewed here,
22 negative/comptime rows retained. With eighteen interpretation originals,
**93** have these receipts and **1,209** lack them: 439 checker negatives, twelve
check-only modules, 153 interpreter-only cases and 605 compiled candidates.
These are distinct required modes, not 1,209 missing executable tests. Nine
positive authored controls and the separate read/depth probes add no inventory
rows. Twenty-four parse/compile/base/run-float direct entries are renewed;
earlier collection direct observations retain their actual provenance.

Full CPU parity remains incomplete. The combined checker-depth failure, read
contracts, competing-error order, host/resource/foreign cells, complete lane/API
review and consumer caveats remain open. The next targeted correction is to
minimize and investigate the combined source's checker-depth growth without
raising limits. An independent next family is `parse/chain_suffix_f32_f`, `_g`
and `_h`: source-inspected function-valued arithmetic/comparison mains requiring
normalized structure/text qualification and actual printer-applicability review.
Their compiled-candidate hints are not execution passes. Those cases, fresh
Windows/macOS, additional worker profiles and GPU execution are unrun.
