<!-- SPDX-License-Identifier: MPL-2.0 -->
# Compile-time reference rebinding diagnostics

Tested source `d0176c3a104688fbe21bb79d9e05330bdc9ce08f`, against unchanged upstream
[`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
`comptime/err_rebind` now matches the pinned invalid-pattern diagnostic and
original source location. Its load-time rejection was already correct. CPU
parity remains incomplete under the [acceptance criterion](cpu-parity-acceptance.md).

## Correction and API boundary

Pinned [`parse_patt`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1690)
rejects a reference in pattern position. Pinned
[`parse_var`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1639)
gives a substituted bare-name argument its use-site span; qualified/imported
references retain the argument's caller span. The port's early pointer-identity
refusal already prevented rebinding, but bypassed structured diagnostic rendering.
Pattern conversion also dropped the occurrence source beside the shared term.

The correction carries that optional source through top-level patterns and the
corresponding constructor-field edges. A real occurrence excerpt takes priority,
then falls back to the shared node's source. The existing early refusal attaches
the existing invalid-pattern diagnostic only for reference values. No shared
reference is copied or globally relabeled. Rejection order, token/cursor state,
binder allocation, core terms, sharing, evaluation and work/depth limits stay
unchanged. Lambda-binder cases and constructor-valued compile bindings remain
separate, unqualified behavior; this batch does not infer upstream agreement for them.

Public ParseError fields and Display, public signatures and plain loader Err
behavior remain unchanged. Optional diagnostic metadata and CLI output improve;
derived Debug can reflect private metadata. Newly formatted CLI `invoke` failures
use the existing `Ok(CliOutput)` boundary with `exit_code = 1`. External callers
remain unaudited and must inspect that field. Entry cancellation, failed writers,
flush errors and host fallback stay under regression; no mid-normalization
cancellation is claimed.

## Exact diagnostics, execution and retained mismatch

The [matrix](cpu-comptime-rebind-results.json) records **346 commands: 318 matching, eight strict-IO observations, two retained port-normalization mismatches and 18 lambda-binder diagnostic observations**. There are zero unexpected outcomes or timeouts; all 155 repeated lane pairs are deterministic. This includes **144 exact negative stream comparisons plus 32 separate port load-phase probes**, **52 actual executions** (48 generated C/JS and four native IO) and **16 pure-normalization observations**. All eight reference-pattern negatives now match exactly. Retained discrepancies have explicit expected observations; that is not a waiver or a parity pass.

Two originals, `err_rebind` and `shadow_arg`, remain byte-identical with Apache
attribution. Eight exact-negative roots cover the original, assignment/match
patterns, multiline/imported/qualified arguments, nested fields and an earlier
use of the same shared argument. All eight reject during loading; rejected
programs produce no artifact and are not executed. The old template-occurrence
pattern test is strengthened to exact pinned stderr. Its lambda companion still
has different diagnostic text, preserved through separate phase-only observations.

Four pure controls and two IO controls cover valid shadowing, repeated template
specialization, imported functions and actual execution. All twelve emitted port
C/JS artifacts are byte-identical to the pre-fix baseline. C uses the pinned O3
profile, one worker and GPU off; port C retains strict Clang warnings. Direct
upstream IO interpretation remains unrun. Strict IO proof refusals remain
exclusions, not runtime successes.

The [local positive control](../tests/fixtures/cpu-comptime-rebind/positive/local.bend)
exposed a pre-existing substantive normalization difference:

| Observation | Pinned upstream | Current port |
| --- | --- | --- |
| Pure normalization / pure CLI run | `12n`, exit 0 | `Nat.add(11n, 1n)`, exit 0 |
| Generated C and JS execution | `12n`, exit 0 | `12n`, exit 0 |
| Explicit IO wrapper execution | Generated C/JS print `12`, exit 0 | Native IO and generated C/JS print `12`, exit 0 |

The baseline port produced the same unreduced application. Pre-fix generated
artifacts were emitted but not separately executed; their bytes match the
currently executed artifacts. The port's pure `run_main` path also uses
normalization and therefore retains the application. Complete graph comparison preserves that application and both Nat
arguments; it does not replace structural comparison with a top-level tag or
pretend agreement with upstream's scalar. The separate
[IO wrapper](../tests/fixtures/cpu-comptime-rebind/positive/local_io.bend)
contains the same `good(~inc, 10n)(1n)` computation. No normalizer or evaluator fix,
new exception, resource refusal or output tolerance is introduced.

[Compact pre-fix receipts](../tests/fixtures/cpu-comptime-rebind/before.json)
preserve baseline identities and the discovered mismatch. Initial authored import
spellings were malformed, and an initial nested control rejected an already-built
constructor scrutinee before the intended binding; their source and observations
remain archived. Only those authored controls were corrected. Original fixtures
were unchanged. Failed intermediate test runs remain recorded, including the
incorrect assumed normalization result, a missing test-helper import, and the
pure run_main assumption. Final corrected expectations keep the demonstrated
normalization mismatch explicit rather than counting it as parity.

## Validation and reproduction

The standard gate passes **1,244 / 0 / 49** (passed / failed / ignored), 184 result groups, exit 0, in 975.17 seconds. Formatting, strict production Clippy, all-feature build and real Node TCP/UDP validation pass. Pre-commit corrected focused tests pass **22 / 0 / 0**, provenance/state tests **2 / 0 / 0**, and the cancellation/writer/fallback boundary control **1 / 0 / 0**. The full gate renews them at the clean tested source. Targeted strict Clippy passes. Three earlier partial runs each ended **1 / 1 / 0**, and one compilation failure ran no tests; these remain failures at their recorded candidate sources. Counts from separate runs are not additive unique-test coverage.

Host build overrides remain frozen and are distinct from Bend's C O3 profile:

```bash
source "$TOOLS/activate.sh"
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --offline --features extended_observability --test cpu_comptime_rebind --test cpu_comptime_type --test cpu_template_occurrences --test parser_templates
cargo test --locked --offline --features extended_observability --lib template_rebind_metadata
cargo test --locked --offline --features extended_observability --lib pattern_load_boundary_preserves
pwsh -NoLogo -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo clippy --locked --offline --features extended_observability --test cpu_comptime_rebind -- -D warnings
```

[Validation receipts](cpu-comptime-rebind-validation.json) retain exact commands,
source/fixture/binary hashes, profiles, bounded deadlines and outcomes. New controls
compare plain/tracked public errors, cursor/tokens/fresh/depth/scope state, strong
ownership and pointer identity, occurrence/node fallback, and earlier deep-input
refusal. Successful controls compare checking/normalization work under tracking.
Existing shared-reference, host-stack, aggregate-work, packed-word and cancellation
regressions remain in the full gate. No guard, test assertion, ignore or budget was
relaxed. Source, evidence and final report received independent read-only review.

## Census and remaining work

The complete [checking census](cpu-comptime-rebind-census.json) renews all **1,302** originals: **366 both accept, 378 exact rejections, 494 acceptance disagreements, 58 different rejections and six oracle failures**. Both-accept establishes checking status only. All **385 historical IO members renew exactly**; outside that same subset the measured residual is **167 = 109 acceptance disagreements + 58 rejection differences**, down from 168. Historical triage labels for those residuals remain 109 acceptance/rejection mismatches, 29 diagnostic-only candidates and 29 other rejection differences; the labels do not waive observations or establish semantic parity.

Only `comptime/err_rebind` changes, and only its port stderr: it now exactly matches upstream. Every acceptance status, failure phase and upstream stream is unchanged. The six upstream host-stack failures are unchanged; there is no oracle-variation adjustment in this census. The fresh missing-IO symptom count remains 386 and does not enlarge the historical 385-member subset. The authored local normalizer discrepancy is outside this inventory census and remains separately unresolved. Historical receipts retain their original source identities.

The [family ledger](cpu-comptime-rebind-evidence.csv) qualifies two originals. Overall evidence linkage is **482 linked / 820 unlinked**: 285 checker negatives, 521 interpreter/compiled candidates, 1 checking-only module, 13 interpreter-only cases. Linked evidence is not universal parity; unlinked entries are not all missing runtime tests. Historical receipts retain their actual tested commits.

The next substantive candidate is the retained closed partial-application
normalization gap above: determine why `good(~inc, 10n)(1n)` reconstructs the
closed `Nat.add` call instead of reducing it. Preserve named-call reconstruction
for genuinely open terms, shared values and all work/depth controls. This
checkpoint only identifies and reproduces that gap.

The Base callback source-coordinate mismatch remains **390 versus 1017**, as
recorded at the prior checkpoint; its existing regression is retained, but no
new exact coordinate agreement is claimed. `nat_first` remains pending and
unwaived, compact Nat/string lowering unadopted, and external invoke auditing
outstanding. Earlier source-hole/TODO, F32.read, foreign, host/printer, resource
and error-order disagreements remain. Supplementary all-target lint failures
remain historical and unrenewed. No new parity exception is added.

Minimum recorded free space is **11.024 GiB workspace / 3.542 GiB temporary**, with the 11 GiB build preflight and 10/1 GiB stop floors enforced. Existing verified cache relocations remain in use; no cache was moved or deleted in this task. No timeout or disk-floor stop occurred.

Compact fixtures, original failures, expected diagnostics and sanitized receipts
are committed. Bulk logs, binary/gzip snapshots, generated files and full census
JSONL remain local under `$EVIDENCE`; Git publication does not preserve them.
Unique historical evidence is retained. No dependency/toolchain change,
installation, desktop/GPU work, main merge or release occurred.
