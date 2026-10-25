# Reusable-let diagnostic locations and full checking census

Tested source: `ddd4f0a9bcc6d8b224b1c17d45fe63b91f6712ac`. The subsequent evidence checkpoint changes only
reports and compact receipts. Upstream remains `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.
The reusable-let location gap is closed. **Full CPU parity remains incomplete.**

## Correction and review

Pinned [check-let](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3452)
relocates a failure only when its expected object is the exact kind expectation
created by that let. The port now keeps that identity in private diagnostic metadata
and prefers the let's source span. An unsourced generated let falls back to the
original diagnostic location. Value-inference errors and unrelated nested failures
are not relocated. Textual or structural equality cannot trigger the relocation.

The metadata owns no terms and changes no public term fields, serialization,
reference identity, template cache keys or evaluation. The kind allocation remains
alive during comparison. The checker uses the same inference/check order, alias
cleanup, fuel, depth and fresh-ID accounting. No dependencies, toolchains, guards,
limits, approved demand semantics or cancellation boundary change.

The first candidate caused an existing 90-application default-stack test to abort.
Extracting the unchanged let rule into `check_let` removed its local state from the
recursive catch-all frame. Independent review compared the extracted body with the
saved failed candidate and confirmed identical ordering/cleanup/accounting. The
original default-stack regression and full gate now pass without a stack override
or higher Bend limits. The failed candidate and receipt remain historical evidence.

Six exact controls cover reusable fields, direct/multiline/nested/imported lets,
and a multiline inference error whose span differs from the let. New units verify
same versus distinct expected allocations, unchanged ownership/work accounting,
exhausted presentation budget, real unsourced-let fallback and cancellation.
Existing nested/reusable-field, shared-reference, parser-stack and aggregate work
regressions remain enabled. Independent review ran no tests itself; validation below
was executed on the recorded source.

## Validation

| Check | Result |
| --- | --- |
| Standard gate | **1,141 passed / 0 failed / 49 ignored** |
| Format, strict production Clippy, all-feature build, Node provider | Pass |
| Focused integration Clippy | Pass |
| Corrected core/normalization/parser/template/pattern suite | 111 passed |
| Diagnostic units / occurrence units / final pattern integration | 35 / 8 / 4 passed; overlapping gate coverage |
| Target matrix | 360 observations: 306 exact negative boundaries plus 54 positive/applicability observations |
| Prior runtime/diagnostic renewal | 246 observations, including retained failed lanes |
| Combined matrices | **606 observations**, deterministic repeated streams/statuses, no timeouts |
| Upstream phase probes | 17 cases load, then reject while checking |
| Positive/unfinished-hole controls | 14 positives; 18 observations retain three unwaived differences |
| Old ten-cell renewal | Nine exact; `unsafe_many` remains unresolved |
| Full pinned checking census | 1,302 measured; all port statuses unchanged |

Negative checking refusals prevent emission; subsequent builds/executions are
**unrun**, never successful runtime observations. Positive graph/text/native and
pure C/JS values remain distinct from executable-Base applicability. The runtime
renewal retains two repeated upstream C-generation failures and two Node direct-IO
host failures, with downstream unrun phases explicit. Supplementary all-target
Clippy was not rerun; its historical failures are not replaced by standard/focused
lint success. GPU/desktop checks were not run.

## Current census versus historical subset

The [full CSV](cpu-checking-census.csv) and [summary](cpu-checking-census.json) cover
all pinned originals, using checking only: neither adapter evaluates `main`.
Actual CLI streams and independent phase probes remain separate. Upstream host
exceptions are classified separately from Bend rejections; process deadlines are
30 seconds for upstream and 60 seconds for the port/probe. No census process timed out.

| Mutually exclusive result | Fixtures |
| --- | ---: |
| Both accept: checking-status agreement only | 366 |
| Exact rejection: phase, status and complete streams | 243 |
| Both reject with different phase/diagnostics | 193 |
| Acceptance disagreement | 494 |
| Upstream Node host-stack failure; oracle incomplete | 6 |
| Total | 1,302 |

There are **687 measured checking differences**, including the **385 previously
documented strict-IO proof exclusions**. Every one of those 385 retains its prior
fixture hash, status and raw streams; there are no new IO-based waivers. This leaves
**302 differences beyond that historical IO subset**, plus the **6 oracle failures**.
These are not 302 independent language defects: they include diagnostic, phase,
strict-proof applicability and resource-policy gaps requiring individual analysis.
Of the 109 non-IO acceptance disagreements, diagnostic symptom triage finds 38
unavailable numeric proof names, eight unavailable File-type names, 22 explicit
foreign-proof refusals, two unsafe-proof refusals, ten Nat-literal parser limits,
four pattern-depth limits and 25 other unqualified symptoms. These subcounts do not
add new exclusions or imply executable failures; several fixtures already have
separate passing executable evidence.

The old **489-case subset** still contains **32 exact / 385 IO / 72 other**. Its 72
is not the current full-inventory residual count. Counts overlap older ledgers;
do not add them together.

The six Node-oracle stack failures are `check/bystander_hypothesis_cost`,
`check/deep_hyp_rewrite`, `page/deep_list_walk`, `reg/borrow_fork_hold`,
`reg/dead_param_fork` and `rfc/share_loop`. Each port still rejects; an upstream
host failure does not establish the required language outcome. No host limit was raised.

All 1,302 port acceptance statuses are unchanged. Of 196 changed stderr streams,
195 differ only in the host Rust source-line number after the renderer edit;
this grouping is diagnostic triage, not relaxed comparison. The remaining original,
[check/cop_let_formation](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/check/cop_let_formation.bend),
now reports the exact pinned Bend diagnostic and `+x = a` source line.

The family qualification ledger remains **410 linked / 892 unlinked**: 341 checker
negatives, one checking-only, 13 interpreter-profile and 537 compiled candidates.
The new census is separate whole-inventory checking evidence, not blanket runtime
qualification or fabricated new family links. Linked receipts can be historical,
partial or blocked. The acceptance criterion remains unchanged.

## Remaining work and reproduction

The four compact adjacent exploratory failures remain unqualified: deeper-kind
source provenance (both check-reject), annotation syntax (upstream check-reject,
port load-reject), malformed comma-separated parallel binding (different load
rejections), and reusable-lambda syntax (upstream load-reject, port check-reject).
Their original streams and a renewed receipt are committed. No policy decision or
semantic workaround was applied. A useful next bounded diagnostic task is the
nested expected-kind source fallback; the independent substantive batch remains
`parse/datatype_declaration_form`, `empty_datatype`, and `empty_param_group`.

Imported-JS printer timeouts, dependent-pair/minimal-IO C failures, F32.read,
competing-error order, erased-hole/TODO behavior, host/foreign/resource/API gaps and
the incomplete 688-vector exploration remain unwaived. Existing `CliOutput::exit_code`,
emitter and `DefDecl::completion_parameters` compatibility caveats remain; external
consumers were not audited. No mid-normalization cancellation is claimed.

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo test --locked --offline --features extended_observability --test kernel_core --test kernel_normalization --test parser --test parser_numeric --test parser_templates --test cpu_template_occurrences --test cpu_pattern_locations
cargo test --locked --offline --features extended_observability --lib diagnostic
cargo test --locked --offline --features extended_observability --lib occurrence
cargo clippy --locked --offline --features extended_observability --test cpu_pattern_locations -- -D warnings
node "$UPSTREAM/bend2/main.ts" "$FIXTURE"
"$BEND" check "$FIXTURE"
"$BEND" compile "$FIXTURE" --executable --target c --output "$ARTIFACT"
```

Caller-selected roots are placeholders. The committed [census runner](../scripts/checking-census/README.md)
includes exact build/probe commands and preserves per-process phase/stream evidence.
CLI and probe hashes are checked before and after execution and tied to a clean
build receipt. C controls retain the pinned C11/O3 profile; host Rust debug settings
above are separate from Bend optimization profiles. Rust/Cargo 1.96.0, fmt
nightly-2026-05-24, Node 24.19.0, Clang 19.1.7 and PowerShell 7.5.4 stay frozen.
Cargo.lock SHA-256: `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.

The gate starts with 11.36 GiB free and reaches 10.30 GiB, above the
10 GiB stop floor; preflight requires 11 GiB, with a separate 1 GiB temporary floor.
Transient growth comes from rebuilt test caches. After completion, only verified
inactive rebuildable binaries are reclaimed, restoring 11.42 GiB. Source,
unique historical failures and captured binaries remain intact.

The [compact pre-fix receipt](../tests/fixtures/cpu-let-kind-locations/before-receipt.json)
and renewed exploratory failures are preserved in Git. Earlier failed test drafts
(exit-code accessor typo, temporary-directory collision), focused Clippy unused
helper failures and the host-stack failure are retained privately with accurate
status, not counted as passes. Bulk logs, process receipts and binary snapshots
remain **outside Git preservation**. No secrets, private machine paths or large
binaries are included in this feature checkpoint. No merge or release.

CLI SHA-256: `fa2c59592c3aa484fee7c7d84fb9608c16ee4bb4013e4fa03d1780ccbf2f18cd`. Probe SHA-256:
`719e87283a7a6ea8aeb4fb506dd6f9ef3d10ca936162914d7fc4630b566a0780`.

| Private evidence identity | SHA-256 |
| --- | --- |
| `full-gate.log` | `812f63a70dd32a8667948f8f3ee555ebcf6abcd41681c531c41d9ff2b1beaf14` |
| `full-gate.json` | `59dd34c105337c7e875fca4f93b0e41e7b3c73f1649199769e417f8afd49bb2c` |
| `snapshot.json` | `c168948630dcee2b6c1156804a101da86e02424e23eab3d90fbd56fc2a04687c` |
| `census-build.json` | `25261d2aab50a453c7f46ddc117b92c4f9f91ecfb0b45724d30610622095aa31` |
| `census/observations.jsonl` | `9699ab35a48bcd26e1c8a2bbdaa0993308cc311bd19185b7be4069628f4f496a` |
| `census/reconciliation.json` | `ef44c05722b3da3aefa2f6e87cfb8c1fca88b751566fe1fbda1d437cee201641` |
| `census/stream-triage.json` | `dbb9e1aa7645f211faa3be37288eb9fb8f38fb01d7ae98e3a48dd0e87b1d06e7` |
| `matrix/receipt.json` | `1fd4d3143cb825e27b8ba2313241ae373ff10b061a43c7257a255ebb572c4737` |
| `baseline-matrix/receipt.json` | `7fb75772faae4f480e681c3724e62bd784e02afdba05ce242dfe701114720156` |
| `phase-controls.json` | `ac3e74d31cbb3daf7f3aa98ca66814048b3042b224d16f66eb719202765047a4` |
| `controls.json` | `86cc8ff83a2f2161d629a71dbe66afc68b699a3d8153e374d4eba1f9240552e5` |
| `retained-controls.json` | `66a2edefd360063e2d4640ec59725a1f7001d5d0b450ac8d8392e623f07de1ab` |
| `focused-structure.log` | `4d7d3c273966dba7bce0ce86f838a285d7b3488e323e39fefa98fbdd87aa575c` |
| `stack-failed-candidate.patch` | `eada0f63ed2c1ad5186a83b7f3e2761f8b60d3e7279aa8c854d8c00806ebedc6` |
| `independent-review.json` | `7dcaed940c2a24577266af53e36d11001d4f45a6fef215a0886758b78ecdbad8` |
| `independent-census-review.json` | `b5e595a2471dae30b0d891a51458e35a0a9bf8a216034e8af12617b1614e841b` |
| `census/acceptance-triage.json` | `ce748b4fc28ea14f383d2ddc62c82fc8301bf7ccfa4b4efc944a4d8281a968de` |
