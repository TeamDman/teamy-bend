# Nested-kind provenance and checking-difference triage

Tested source: `244b659c78caa56c4cc2d1df25daa9e693e591b8`. Subsequent checkpoint changes only reports and compact
receipts. Pinned upstream remains `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.
**The nested-kind provenance gap is fixed; full CPU parity remains incomplete.**

## Correction and validation

Quantified type atoms lacked a source anchor, and their domain/codomain child
origins were lost when a local variable became a shared replacement. An adjacent
annotation case had the same child-origin omission. Three production metadata
additions register `All` atoms and record the two child edges of `All` and `Ann`.
The existing source-priority and substitution machinery then report the original
Bend location. Pinned [infer-all](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3331)
and check-any retain the rejected child span.

No core term, reference allocation, strong ownership, serialization, evaluation,
checking rule, demand order, budget or cancellation boundary changed. Independent
review confirmed child indices, weak-source ownership, source precedence and all
13 fixture hashes/expected diagnostics. The reviewer ran no builds or tests.

| Validation | Result |
| --- | --- |
| Standard gate | **1,143 passed / 0 failed / 49 ignored** |
| Format, strict production Clippy, all-feature build, Node provider | Pass |
| Focused core/parser/normalization/template/integration checks | 112 passed |
| Origin units / occurrence units / focused integration Clippy | 16 passed / 8 passed / pass |
| Target matrix | 594 observations: 540 exact negative boundaries plus 54 positive/applicability observations |
| Prior runtime/diagnostic renewal | 246 observations, retaining two failed oracle/host lanes repeated twice (four failed observations) |
| Combined matrices | **840 observations**, deterministic repeats, no timeouts |
| Upstream negative-phase probes | 30 load successfully, then reject while checking |
| Positive/hole controls | 14 positive observations; 18 retain three unwaived cases |
| Prior ten retained cells | Nine exact; `unsafe_many` unresolved |
| Full checking census | All 1,302 measured; 1,302 port statuses unchanged |

The original deeper-kind fixture remains unchanged. Controls cover domains,
codomains, annotation values/types, multiline/imported sources, nested arrows,
reusable grades and first-domain-error ordering. Units verify distinct edge origins
when both children share one replacement, unchanged strong ownership and separately
sourced replacement precedence. Existing tracked/plain checker-work, source-hole,
structural/depth, default-host-stack, shared-reference and aggregate-work regressions
pass. No stack, work, depth, parser or host limit was increased.

Rejected programs do not execute: generation/build/runtime phases after checking
refusal remain unrun. Native graph/value and pure C/JS observations are distinct
from executable-Base applicability. Old C-generation/Node-host failures, imported-JS
printer timeouts and other prior discrepancies remain explicit. Supplementary
all-target Clippy was not rerun; its historical failures remain. No GPU or desktop
checks ran. Existing external API caveats remain; no new public API change and no
claim of mid-normalization cancellation.

## Exhaustive baseline 302 classification

The [per-fixture CSV](cpu-checking-triage.csv) classifies the **302 non-IO checking
differences measured at `ddd4f0a9bcc6d8b224b1c17d45fe63b91f6712ac`**. Each row retains
the pinned fixture hash, phases/statuses, evidence basis and current disposition.
The [summary](cpu-checking-triage.json) distinguishes baseline from renewed evidence.

| Primary category | Baseline fixtures | Interpretation |
| --- | ---: | --- |
| Diagnostic-only candidates | 151 | Same apparent rejection cause/rule; not exact qualification |
| Acceptance/rejection mismatch | 109 | Upstream accepts checking; strict port rejects |
| Other rejection differences | 42 | Parser boundary, missing-name/foreign/resource preemption or completion policy |
| Host/oracle limitation inside 302 | 0 | The six oracle stack failures are separate |
| Total | 302 | No case waived by triage |

The 151 candidates comprise 104 same-load-cause and 47 same-check-rule/display cases.
The 42 others comprise 24 parser-boundary, eight missing proof-name, three resource,
three foreign-proof, three TODO/completion and one duplicate-IO/parser precedence
case. Common rejection phase alone does not prove diagnostic-only behavior.

Here the host/oracle category means an unavailable measurement, not a measured
proof-mode refusal. Foreign/host API refusals retain secondary tags under acceptance
or preemption; they are not silently dropped from the 302.

The 109 acceptance disagreements comprise 38 numeric proof-name, eight File-name,
22 foreign-refusal, two unsafe-refusal, ten Nat-literal-limit, four pattern-depth-limit
and 25 other symptoms. These classifications are not new exclusions or evidence
that 109 executable programs fail. Some already have separate executable matches.
`List.for_each` and `App.play` exist in the executable prelude, so their strict
checking failures must not be called generic template-parser defects.

The six upstream Node stack failures remain outside 302: `check/bystander_hypothesis_cost`,
`check/deep_hyp_rewrite`, `page/deep_list_walk`, `reg/borrow_fork_hold`,
`reg/dead_param_fork`, `rfc/share_loop`. Their oracle language outcomes remain unknown.
Five missing-file import negatives are intentional fixtures, not missing environment
setup. `state/operator_goal` prints different representations of the same words
0x3fc00000/0x40200000 (1.5/2.5), not different float values.

Current full-census counts: `{"acceptance_disagreement": 494, "both_accept": 366, "different_rejection": 191, "exact_rejection": 245, "oracle_or_infrastructure_failure": 6}`.
The 385 historical IO exclusions retain exact source/status/streams. Current
non-IO differences: **300**, separate from six oracle failures.
The two improved originals were diagnostic candidates, leaving 149 current
presentation candidates, 109 acceptance mismatches and 42 other differences.
Changed port streams versus the previous source: `check/cop_dead_hypothesis_000.bend`, `flatten/destructure_annotation_error.bend`. Full raw observations
are retained; [compact renewal](cpu-nested-kind-census.json) avoids duplicating the
prior 1,302-row CSV. The old 489-case subset is historical and is never substituted
for these whole-inventory measurements.

## Next semantic priority

After classification, a bounded pure probe isolated **`reg/cop_let_descent_escape`'s
reusable Nat-pattern parsing**. Pinned upstream accepts `case 1n++k` and evaluates
the small probe to `4n`; this port load-rejects it as `String.append(1n, k)`. Ordinary
`1n+k` still rejects duplicated affine use in both. A spaced `1n + +k` form also
rejects upstream and must not silently become accepted during a fix.

The [three probes and receipt](../tests/fixtures/cpu-nested-kind/next-gap/README.md)
retain 12 deterministic process observations plus six independent phase probes.
They are unqualified failing evidence, not an implementation or passing regression.
Pinned numeric parsing consumes one `+` before reusable `+k`; the port tokenizes
`++` together. A contextual numeric-pattern correction, with ordinary-affine,
whitespace and string-append controls, is the next substantive CPU task. It needs
no desktop or new dependency. The 24 parser-boundary candidates deserve positive
companion checks afterward; diagnostic candidates should not displace this
semantic investigation indefinitely.

Family qualification coverage remains 410 linked / 892 unlinked: 341 checker negatives, one
checking-only, 13 interpreter-profile, 537 compiled candidates. The census and triage
are separate checking evidence, not new blanket runtime credit. F32.read,
competing-error ordering, erased-hole/TODO behavior, host/foreign/resource/API gaps,
failed oracle lanes and the incomplete 688-vector exploration remain unwaived.

## Reproduction and preservation

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo test --locked --offline --features extended_observability --test kernel_core --test kernel_normalization --test parser --test parser_numeric --test parser_templates --test cpu_template_occurrences --test cpu_pattern_locations
cargo test --locked --offline --features extended_observability --lib origins
cargo test --locked --offline --features extended_observability --lib occurrence
cargo clippy --locked --offline --features extended_observability --test cpu_pattern_locations -- -D warnings
node "$UPSTREAM/bend2/main.ts" "$FIXTURE"
"$BEND" check "$FIXTURE"
```

The committed [census runner](../scripts/checking-census/README.md) specifies phase
probe/build commands. Child bounds remain 20 seconds in integration tests, 30 seconds
in matrices/upstream census and 60 seconds for the port census. The gate deadline is
1,200 seconds. Host Rust debug settings above are separate from Bend C11/O3 profiles.
Rust/Cargo 1.96.0, fmt nightly-2026-05-24, Node 24.19.0, Clang 19.1.7 and PS 7.5.4 remain
frozen. Cargo.lock SHA-256 is `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.

Gate preflight: 11.33 GiB free; minimum: 10.30 GiB.
Only completed, inactive rebuildable binaries were reclaimed afterward; free space
is 11.40 GiB. The 11 GiB preflight, 10 GiB stop and 1 GiB temporary floors
remain. Source and unique historical failures are preserved.

The compact original failure and next semantic probe are preserved in Git. Bulk
logs, raw process receipts and binary snapshots remain outside Git preservation.
Reviewed feature publication is not a main merge or release. Tested CLI SHA-256:
`46957d205cb4ceb3c5cb55e060bf6cda987dc347f12e61cbd4fccf77ee1ad130`.

| Private evidence | SHA-256 |
| --- | --- |
| `full-gate.log` | `0740546225a0029a017a029ee66e4b3f1169e0f017f10d5df39b4163f0c168b1` |
| `full-gate.json` | `cae592a97310776bfdba52969a5a10f6a35ec72738edfefa7ce2f84433c4268c` |
| `snapshot.json` | `2ed421c3235ac1e68450ad64b357f77eacd32d3aecbe34f37825de778b1e1640` |
| `source-review.json` | `b0502be4b8e5a7e9a6d85ec5808af17ff2930bbea24f7770849b6b73fb4a8bd9` |
| `census-review.json` | `3af69d8785d4bc9476ab398682a92234edd8c26ea752d170e1aa9e9f9e87f7a1` |
| `review-classification.json` | `de79c9e348409b9e878223f806e3e596ca65b8b88d4c7f520077da7bd5a30573` |
| `census/observations.jsonl` | `42520b5b945dfb208f275d16545d8b2f50a0931fe5f9da8927d418edf2b3e401` |
| `census/reconciliation.json` | `9cf12b076362257246c340e2ef5ee7f23c5cdbca5c6993d0e4175dd93257e2bd` |
| `matrix/receipt.json` | `6b4afd95d36d704cc03afb86a69b222300227970459ef988372fb0e651f7c955` |
| `baseline-matrix/receipt.json` | `6f116af6f4621638b98e64f03fcd4eee7af8a47b58e844f49e505c4705ec6ddf` |
| `phase-controls.json` | `9458eef046456ffbe11109dbfba53b53bfb651732fbe2f244f006c7abaa4d7b6` |
| `next-gap/receipt.json` | `e16eaaf6762010e6e0af3d23c23527dc349cb0dfca819f6e6fbacbb8ac243c7e` |
| `next-gap/phases.json` | `1421a222f98d8424db20f9fa1409f9ea8d81daf8b8b6d7f58505da151f165db5` |
