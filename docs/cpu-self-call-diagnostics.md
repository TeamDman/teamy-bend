<!-- SPDX-License-Identifier: MPL-2.0 -->
# Self-call refusal diagnostics

Tested implementation `b8a07c233ab3726d9152f13d465130ef1672bb20` starts from
published `73148a54e9e3dcad66dd1a1f46f20cf3f9d7818d`. Upstream remains
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`; docs remain
`50e89d6897ce2a65167ef65b17c5fa83fe849158`. Dependencies, lockfile, toolchains,
termination checks, execution budgets, structural guards and cancellation
boundaries are unchanged. Full CPU parity remains incomplete.

## Contract and correction

The three unchanged originals `check/cop_circular_witness_000.bend`,
`check/descent_field_order.bend` and `check/partial_self_call_stuck.bend` already
failed in both checkers. The port previously exposed its internal message through
the host formatter. Pinned [infer-ref](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3261)
rejects a live safe self-reference unless its supplied columns decrease; its
error observes the bare reference and retains that reference's source span:

```text
- expected : a decreasing self-call (arguments are read left to right: each passed unchanged until one shrinks)
- observed : part
```

The fix attaches existing structured diagnostic metadata **after the unchanged
non-Less verdict**. Neither the descent loop, error propagation, argument order,
erased demand nor unsafe-context rules change. Constructor-field ordering is
still tested by the original `descent_field_order` refusal; it is not replaced by
a first-differing-field rule. A partial self-call whose supplied columns shrink
remains accepted; one that stays equal remains rejected.

Checking installs the current definition with no body before checking its body.
Formatting the rejected reference cannot unfold an unchecked recursive body.
Existing source occurrence tracking yields the original call location, including
reconstructed and imported references. There is no new source metadata or term
copy, and no change to pointer identity, sharing, evaluation or runtime code.

The existing diagnostic helper clones the current remaining fuel/depth allowance.
If formatting cannot fit, the original refusal remains available through the
existing host fallback. Unit regressions verify the unchanged one-step rule cost,
Rc ownership/fresh IDs, error identity, exhausted-display fallback and actual
fuel/depth guard priority. The unsafe-context control is an internal inference
rule test, not strict whole-book acceptance. No new mid-normalization cancellation
facility is claimed; existing cancellation and writer controls remain in the gate.

Public signatures and `KernelError` message/Display/equality/kind are unchanged.
As with earlier sourced failures, CLI `invoke` may return status-bearing `Ok`
output with exit code 1. Repository dispatch reads that status; external invoke
caller auditing remains outstanding. No error-string parsing grants a language
exit contract.

## Validation

[Compact before/after failures](../tests/fixtures/cpu-self-call/self-call-before.json)
retain the old actual error text and new exact diagnostic. The baseline binary
is clean `6507da388f35a88767d309b14878835bda0cdcec`, production-identical to the
starting documentation checkpoint. All seven baseline cases reject; the 28
commands include upstream checking and both generation refusals, plus port check.

Seven negatives (three originals plus bare, unchanged, reconstructed and imported
controls) pass **126 exact stream commands and 42 separate checking-phase probes**.
Each lane repeats twice with identical output/status. Port check/eval/run and
C/JS/native compilation reject with stdout empty, status 1 and no artifact;
upstream checking/C/JS generation match exactly. No negative runtime execution
is claimed. Full stderr includes context order, names, source window and location.

The unchanged positive `check/partial_self_call.bend` plus alias and later-column
controls yield `1n`, `7n` and `2n`. Their **78 commands** comprise 12 pure
normalizations, 24 checking/phase observations, 18 generations/builds and
**24 actual C/JS executions** (port/upstream, twice each). Pure native API
`run_main` checks are normalization; these fixtures do not exercise native IO.
Upstream C uses its actual O3 profile; port C also retains strict warnings.
The repeated 30-second bounded matrices complete without timeout or mismatch.
See [commands/results](cpu-self-call-results.json) and
[validation/build identities](cpu-self-call-validation.json).

The standard gate at clean `b8a07c233ab3726d9152f13d465130ef1672bb20` passes **1,201 tests / 0 failed / 49 ignored** in 841.62 seconds. Formatting, strict production Clippy, the all-feature build and real Node socket-provider checks pass. The ignores remain 47 CUDA hardware tests and two optional profilers.

The focused new unit test passes; renewed integration and descent-alias controls pass **5 / 0 / 0**. Targeted strict Clippy passes. These initial focused receipts were captured on the dirty implementation candidate; the clean full gate rechecks the final committed tests, including earlier stack, shared-reference, aggregate work, demand, cancellation and diagnostic controls. No assertion or ignore was weakened. Production and evidence received independent review.

Two exploratory failures remain recorded. An authored import initially used
unsupported `from ... import ...` syntax; it was corrected to existing
`import ./bad.bend as B` before qualification. The first new integration run
passed its negative test but failed its positive test because the authored
expectation omitted Nat's `n` suffix (1 passed / 1 failed; later target unrun).
The pinned interpreter confirmed `1n`, `7n`, `2n`; only expectations were corrected.
Neither failure represents a newly fixed production parser or printer defect.

## Census and remaining scope

The renewed full census measures **366 both accepted, 354 exact rejections, 494 acceptance disagreements, 82 different rejections and 6 upstream host/oracle failures**, across all 1,302 pinned fixtures.

All port statuses and phases are unchanged. Exactly 28 port stderr streams change through this self-call diagnostic boundary; 27 are now exact upstream rejections. Every other port stream is unchanged. One of the 28 changed diagnostics remains non-exact: `halt/non_structural_descent` prints context type `Nat - [Zero]`, while upstream prints `Nat<> - Zero{}`. Its expected/observed terms, phase, status and source location agree; this is still a context-printer mismatch, not a waived or completed comparison. All 385 historical IO members renew byte-for-byte. **191 measured differences remain outside that exact cohort: 109 acceptance and 82 rejection differences**. IO-symptom candidates are not new exclusions. The residual decrease 217→191 consists of 27 exact diagnostic fixes plus one upstream stack-failure-to-acceptance transition, which adds a remaining acceptance disagreement.

Upstream status/stream transitions versus the preceding ca333c4 census: `flatten/literal_rows_cubic.bend`. These are independent oracle observations, not port improvements. Current host/oracle failures: `check/bystander_hypothesis_cost.bend`, `check/deep_hyp_rewrite.bend`, `page/deep_list_walk.bend`, `reg/borrow_fork_hold.bend`, `reg/dead_param_fork.bend`, `rfc/share_loop.bend`. Historical unstable outcomes remain unresolved, even if this run accepts a previously failing input. No followup replaces a census observation.

[Reconciliation](cpu-self-call-census.json) preserves exact changes and raw identities. [Changed status rows](cpu-self-call-census-delta.csv), together with the immutable preceding [full CSV](cpu-final-todo-census.csv), reconstruct current classifications without duplicating unchanged rows (set engine_commit on all rows to the tested source above before applying the delta). Both-accept is checking-status evidence, not normalization or runtime parity. Prior counts remain tied to their actual commits.

Applicable original linkage rises **453→457 of 1,302**. The **845 unlinked** entries comprise 521 interpreter/compiled candidates, 310 checker negatives, 13 interpreter-only cases and 1 checking-only module. Linkage and repository test totals do not establish universal CPU parity.

Nat-first C ordering remains pending/unwaived; the six previously approved
wildcard observations remain a separate historical scope. The earlier demand
matrix is not relabelled as a current run. Nat/string compact lowering remains
unadopted. Existing hole/TODO, F32.read, resource, foreign/host, oracle, API and
supplementary all-target lint gaps are unchanged by this diagnostic correction.
No new semantic exception was introduced.

The newly exposed residual context-printer mismatch in `halt/non_structural_descent` is the next concrete diagnostic gap. An independent bare parameterized datatype-head gap is `check/family_head_bare`. Both sides reject, but the port retains its internal “parameterized datatype needs <...> arguments” message instead of the pinned family-instance expectation and source context. [Compact current failures](cpu-self-call-next.json) retain the evidence; no type-acceptance change is proposed. The nearby `data_tuple_family` and `data_family_alias` fixtures actually accept upstream and meet the port strict-IO exclusion; they are not diagnostic mismatches in this family.

## Reproduction and preservation

Activate the installed tools and use the existing frozen host build profile:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --lib \
  self_call_diagnostics_preserve_guards_sharing_and_remaining_budget
cargo test --locked --features extended_observability \
  --test cpu_self_calls --test kernel_descent_alias
cargo clippy --locked --features extended_observability --test cpu_self_calls -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
node "$UPSTREAM/bend2/main.ts" tests/fixtures/cpu-self-call/check/partial_self_call_stuck.bend
"$BEND" check tests/fixtures/cpu-self-call/check/partial_self_call_stuck.bend
python3 scripts/checking-census/run.py --reference "$UPSTREAM" \
  --binary "$BEND" --phase-probe "$PROBE" --output "$FRESH_CENSUS" \
  --build-receipt "$BUILD_RECEIPT"
```

The [census instructions](../scripts/checking-census/README.md) explain build
provenance; use the exact recorded binary and phase-probe identities. Receipts
record all matrix commands, fixture/artifact hashes and lane outcomes with portable
placeholders. Repetitions are compacted only after exact equality checks. Census
limits remain four workers, 30-second upstream and 60-second port/probe bounds.
Host Rust debug/incremental settings are separate from Bend optimization profiles.

Minimum recorded free space was **11.203 GiB workspace / 4.581 GiB temporary**. The 11 GiB workspace build preflight and 10 GiB workspace / 1 GiB temporary stop floors remain enforced. Existing cache placement and disabled host debug/incremental data remain in use. No cache or unique historical evidence was deleted.

Compact unique failures, attributed fixtures, exact goldens and reproducible
receipts are in Git. Raw logs, full census streams, generated artifacts, failed
draft and lossless binary snapshots remain local and are not preserved by Git
pushing. No install, desktop/GPU work, main merge or release occurred.
