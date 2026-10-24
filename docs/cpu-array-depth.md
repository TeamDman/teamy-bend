# Array normalization depth correction

Source `e9410bfc9eda4971ba9819651482440fcdc7b126` closes the native pure CLI and strict-eval failures recorded
in the [partial array checkpoint](cpu-array-syntax.md). The test-only follow-up
`6644368cd2759c54500f9544e8a2e367072e7b43` distinguishes tail-fuel refusal from pending-depth refusal.
Production code is identical between these commits. Both unchanged originals
and the four-slot regression now agree with the pinned interpreter and generated
C/JavaScript. This is a bounded compatibility repair, not full CPU parity.

## Cause and correction

The kernel recursively called `whnf` after unfolding a definition even when the
only remaining action was to return the next reduction's result. These tail
frames accumulated alongside pending match/argument reductions until
the 128-level guard refused a four-slot read. The earlier candidate removing a
selected-arm reduction alone did not fix the original; its patch, failed output
and trace remain preserved with the previous checkpoint.

The reducer now loops through tail transitions (unfolded applications, aliases,
annotations, lets and selected rewrite/quantity bodies). Every transition still
charges the work step previously charged by the recursive call. Reductions with
pending work still enter the ordinary depth guard; the outer frame unwinds on
success and failure. Match-arm reduction, structural normalization, freshening,
checking and evaluator dispatch retain their existing behavior.

The [pinned weak-head contract and machine](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2807)
keep arguments, fields and unselected arms weak, and use loop transitions for
unfolding. That proof-normalizer behavior is distinct from the approved strict
live-argument executable contract. This patch changes neither demand contract
and does not add upstream-style sharing or a replacement evaluator.

The 128-level pending-depth guard, 2,000,000-step budget and input limits are
unchanged. Production changes are confined to `kernel/reduce.rs`; additions in
`kernel/check.rs` are tests. No dependency, toolchain, public API or CLI stream
envelope change occurred. No installation or push occurred.

## Exact observations

| Program | Exact stdout, followed by LF |
| --- | --- |
| Original array_literal | `([3, 3, 3, 3], [True{}, True{}])` |
| Original array_write_stmt | `([7, 1, 0, 0, 0, 42, 0, 0], 42)` |
| Authored four-slot read | `([0, 0, 0, 0], 0)` |

The completed repeated matrix has **36/36 exact executions**: three programs,
six lanes (native pure CLI, port/upstream C, port/upstream JavaScript, direct
pinned interpretation), twice each. **6/6 strict JSON eval reports** also match
the expected entry/value. Every execution exits 0, stderr is empty, all repeats
are deterministic and no timeout occurs. C uses the upstream CLI's O3 profile
with `--threads 1 --gpu off`; port C retains strict warning flags. The public
`parse-arrays` runner now exits 0.

The six earlier one/two/four/eight-slot read/write minimizers all agree with
upstream, as do four isolated word-expression controls. The original eight-slot
write program is unchanged; it was not replaced by a smaller passing example.
Tests check actual ALeaf/ANode core structure independently of shared list/array
pretty notation. Constructor-runtime API checks remain separate from CLI checks.

An existing `U32.div(17, 5)` test expected the old depth refusal. Tail-frame reuse
now computes quotient 3 and remainder 2, matching a direct pinned interpreter
control. Its assertion was strengthened to those exact results. Dedicated guard
checks retain bounded refusal coverage rather than preserving accidental failure.

## Guard and regression evidence

Three new kernel tests verify exact fuel charging (200 tail transitions require
201 steps including the leaf), cyclic-alias budget refusal, depth refusal for
128 pending reductions, first-error ordering, recovery after failure, and weak
demand of unused arguments/branches/fields/annotations and stuck rewrites.
Full normalization still demands a delayed field. Unsafe tail recursion still
fails closed but now reports evaluation-budget exhaustion; constructor-producing
recursion still reports nesting exhaustion. Both retain cancellation and empty
output assertions. The first full gate exposed the stale tail diagnostic assertion
and stopped; that failed receipt is preserved. Existing inference-depth,
128-Nat CLI refusal, input-depth and false-proof controls also pass.

| Check | Result |
| --- | --- |
| Standard repository gate | **831 passed, 0 failed, 49 ignored**; exit 0 |
| Focused integration checks | **89 passed, 0 failed, 0 ignored** |
| Focused internal guard checks | **5 passed**, including three new tests |
| Executable guard and contract checks | **10 passed**, with tail/pending recursion and cancellation assertions |
| Pinned formatting, strict all-target Clippy, build | Passed |
| Array matrix and strict eval | 36/36 executions plus 6/6 reports; exit 0 |
| Five-fixture baseline | 50/50 exact executions |
| Normalization | 42 fixtures; 330 successful commands, 108 actual printer refusals, six retained F32 proof exclusions; exit 0 |
| Strict checking audit | 1,302 processed; 366 expected positives accepted, 487 refused, all 449 nonzero-expectation fixtures refused including all 439 checker negatives; no abnormal exit/timeout |
| Six runtime-negative originals | Classifications unchanged; runner exit 1 |
| Ten executable demand controls | Classifications unchanged; runner exit 1 |

The final full gate and renewed array/strict-eval matrix ran at `6644368`.
The audit, baseline, negative/demand and normalization matrices ran at `e9410bf`;
the only subsequent source change is the executable guard test. Their receipts
and ledger rows keep those exact identities.

The strict audit has no acceptance-status changes from its recorded historical
baseline. It does not establish diagnostic or runtime equivalence. Negative
lanes still include four resource refusals, one native foreign gap and one
unavailable upstream Bun lane; two C cells are inapplicable to a JS-only fixture.
The demand matrix retains the three approved upstream-C wildcard exceptions
and one unresolved upstream-C competing-error diagnostic order. Those latter
observations are not promoted to exact agreement. Raw diagnostic discrepancies,
F32.read differences, resource policies and platform/API gaps remain open.

The 49 ignored tests remain 47 CUDA checks and two optional profilers. Full
positive-runtime and read/comptime/checking-diagnostic differential matrices,
GPU and fresh Windows/macOS validation were not renewed. Earlier receipts keep
their actual source identities. Initial failed focused/Clippy runs, the corrected
assertion and lint, and all final logs remain available as private evidence.

## Reproduction and provenance

Activate the existing frozen environment, with `CARGO_NET_OFFLINE=true` and
`BEND_GPU=off`. Commands (evidence destinations are fresh directories):

```sh
cargo +nightly-2026-05-24 fmt --all -- --check
cargo test --features extended_observability --lib depth_tests --quiet
cargo test --features extended_observability --test executable_check --quiet
cargo test --features extended_observability --test cpu_array_syntax --test kernel_normalization --test kernel_arrays --test kernel_core --test kernel_descent_alias --test parser --test parser_numeric --test cli_proofs --quiet
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-features --bin teamy-bend --example compare_normalization
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <arrays> -Suite parse-arrays -Repetitions 2
python3 <private-evidence>/complete-matrix.py
python3 <private-evidence>/minimize-depth.py
python3 <private-evidence>/minimize-word-depth.py
python3 <private-evidence>/audit.py
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <clean-pin> -OutputDirectory <negatives> -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <clean-pin> -OutputDirectory <demand> -Suite demand -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <baseline> -Suite baseline -Repetitions 2
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <clean-pin> -OutputDirectory <normalization> -Suite all -Repetitions 2 -RequireExactText
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Individual comparisons use 60-second deadlines; audit concurrency is four.
Negative/demand runner exits remain 1 for the explicitly open observations.
All source and binary hashes, commands, exact streams/statuses and repetitions
are retained. Original fixture bytes and attribution remain unchanged.

Initial disk preflight recovered 3.48 to 12.94 GiB by removing inactive
rebuildable test products. Each full gate had a fresh preflight. Minimum final
gate headroom was 3.38 GiB workspace and 4.77 GiB temporary space; the gate took
170.82 seconds. No exhaustion or deadline stop occurred.
The 12-GiB pre-build reserve and 2-GiB workspace / 1-GiB temporary monitoring
floors remain in force. Source, evidence, libraries and immutable binaries are
protected; retained snapshots and relinking still account for disk growth.

| Receipt / input | SHA256 |
| --- | --- |
| arrays | `a0603765f9a03dc3e4b6ab3f0b3fa80bc250d389242cd9e0c52319dbad3a4e36` |
| complete-matrix | `0fb019d9e845366fa176a8b2bd777ff265afca1de5565d11631386bdc142bd78` |
| depth-controls | `a92dd793b00586d27da58ae61f9b60f2e6ff611f11a2b7c03a22ff06d35ce99e` |
| word-controls | `b83569393024ebdb0873405b1bcc0b62d7968067b85a025bd77a6ef63b935d8c` |
| audit | `dfef7cd6e93f67d8b4a397a6fdebd0d7f94e03088be83bcfc2e34240ddc57f53` |
| negatives | `ad99c0869fc6348c2658ff84a79b91797e5dc6fcedcb299611ec5528f5abcc64` |
| demand | `4245c4a9a261cb2f8339604ab36a312c31b6dba93896bf43b5985554cfb00288` |
| baseline | `6175e33aed72b335283ab41a6aff33063a0011948d1aa85c676c306cf1885bf6` |
| normalization | `c6fbc350791011232ec83e69870ee8d22f983add1712d6c0d20668c5140e6385` |
| gate | `6ee592c0f0c6b30ff793a193ab892a3fb8f3acc73e5bef04cd0f1efff47b80fb` |
| initial-failed-gate | `d372d37ac1a39ccb7729dccfc3ac5be3221eb509f9e33adc722576bd29422320` |
| renewed-arrays | `8c586d97e9812fdb33fc2d70ea49eabeda196067f51abbb4195a9422bd24016e` |
| renewed-complete-matrix | `650f23ab283c30ac54e152d4fbd59ad5f8d726763c9ed68e57a8db85958ffc15` |
| division-control | `32bdf7c640c24a65996f15ecc1486a8999861741f2e29573010d5fdee9b86b97` |
| cache-before-build | `373ca11526c6f855b67fda5b5fa35661e4a6c80c4343e74305b9ab8b6e3dc25e` |
| cache-before-gate | `ea6c9a00cffbc48cdd7b1223fb741b2dbe71c959098a02e7a39cb4e46a50ecb5` |
| Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |
| prior-rejected-candidate | `ea9678938038af4c1772d174ea3d47079ec99ff1fb2fbb354aef89dc1c99ff86` |
| Tested CLI | `ea603e91afbeec33ccee0d89a2a4a285ee5fe4f1183108f806e1a5a3774035a9` |

The runtime ledger remains 77 originals: eleven rows renewed here, 66 retaining
historical receipts. All 38 interpretation and ten demand rows are renewed;
O2 demand evidence keeps its separate historical identity. The runtime/interpretation
union remains 115 originals; the 1,187 absent entries retain their distinct
checking/interpretation/compiled applicability categories. An authored minimizer
does not add an upstream inventory row.

Next independent family: parse/array_read_annotation and parse/array_read_write,
for annotated reads and chained read/write extraction. Their actual printer and
runtime applicability still need qualification; this audit is only checking
status. Full CPU parity remains incomplete. There is no new executor/dependency
blocker or resource-policy decision required for this correction.
