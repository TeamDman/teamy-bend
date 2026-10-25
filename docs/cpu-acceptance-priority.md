<!-- SPDX-License-Identifier: MPL-2.0 -->
# Acceptance disagreements: mode and resource classification

Clean tested checkpoint: `67b2035d525ea54b34c8d1ab4e1cf8a8f21e7b2f`.
**No new production acceptance defect was demonstrated in these 109 cases.**
All 109 strict-check disagreements remain recorded. This checkpoint adds
classification, reproducible checking probes and executable regression coverage;
it changes no production source, semantics, dependencies, toolchains or budgets.
Two independent reviews agreed that weakening strict checking or raising limits
would not be a justified correction. CPU parity remains incomplete.

## Measured scope

The input is the reconciled 1,302-row census at `59ac3fdb…`, selecting its
**109 acceptance disagreements outside the 385 historically renewed IO rows**.
Every selected source hash, upstream checking result and port strict-check
status/stdout/stderr renewed exactly. Historical labels remain in the new
[109-row classification](cpu-acceptance-priority.csv); the original census and
triage are untouched. No new exclusion or policy exception is added.

| Current cause | Count | Fresh executable loading/checking |
| --- | ---: | --- |
| Execution-only numeric API | 38 | Accepted |
| Execution-only File API | 8 | Accepted |
| Strict foreign refusal | 22 | Accepted |
| Unsafe definitions/signature formation | 3 | Accepted |
| Execution-only templates | 3 | Accepted |
| Chan / network API | 5 / 11 | Accepted |
| Private Emit constructor / newly exposed IO | 1 / 1 | Accepted |
| Nat literal guard, 128 | 10 | Load refusal |
| Pattern-compilation guard, 64 | 4 | Load refusal |
| Binder telescope guard, 128 | 1 | Load refusal |
| Missing Window / Audio implementation | 2 | Check refusal |
| **Total** | **109** | **92 accepted; 15 load and 2 check refusals** |

The 92 acceptances are **checking observations, not 92 runtime passes**. Existing
runtime/interpretation/checking receipt links are retained per row for follow-up;
links are not universal qualification. Foreign native execution, effects,
resource behavior and oracle availability need their own evidence.

All 17 executable refusals were additionally reproduced through executable
JavaScript generation, capturing exact status/stderr and confirming no output
artifact. These are failed generations, not unrun executable successes.
[Compact commands, diagnostic receipts and hashes](cpu-acceptance-priority-results.json).

Window/Audio are absent implementation surfaces: checking stops before devices
or host operations. Pinned [opaque types](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L100)
and [seven operations](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L324)
exist upstream. Direct upstream CLI attempts separately fail with
`TypeError: require is not a function`; their checking-only probes pass. This
host execution failure neither explains nor excuses the port's missing API.
The initial private selection receipt mislabeled direct CLI as checking; the
published [baseline receipt](../tests/fixtures/cpu-acceptance-priority/baseline-selection.json)
corrects the label and preserves the observed streams. The original private
receipt remains intact.

The 15 resource refusals remain unresolved compatibility limits. Their renewed
errors do not demonstrate inflated accounting or justify bypassing guards.
[Compact-pattern research](cpu-compact-patterns.md) remains unadopted, including
the original string-pattern failure. Other pattern cases need separate analysis.

## Selected candidate: unsafe_many

`check/unsafe_many.bend` initially appeared actionable because strict checking
reports Data versus Type before its later unsafe definition. Investigation
showed existing executable support already matches pinned
[`lhs_kind`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L653):
unsafe reusable domains can form at Type. The port's `Lhs::domain_kind` applies
that exception only in executable checking, and its unsafe-name pre-scan supplies
the later completion's context to the earlier law. Binder use rules remain intact.
Strict checking intentionally does not mint a proof token for this contract.

The unchanged original's main returns **7 without invoking twice**. The authored
live companion actually invokes the nested reusable function twice and returns
**5n**. Both agree across native, port C/JS and pinned upstream C/JS, twice each:
**20 executable observations**, plus **4 upstream direct CLI observations**.
Generation succeeds eight times; four C builds use C11/O3 and pthread/math links,
with unchanged strict warning flags for port C. No tolerance or output
normalization is used. There were no timeouts or repeat-result changes.

Four negative controls cover a safe reusable signature, a later ordinary
signature, false reflexivity and a named unfinished hole. All executable-path
refusals match pinned diagnostics, source locations, status 1 and empty stdout;
rejected generation emits no artifact. Strict proof mode remains separately
recorded: it rejects the unsafe declarations earlier in three controls, so those
strict diagnostics are not counted as exact upstream diagnostic matches.
Across negative diagnostics (including checking-only observations), 60 match
exactly and 12 strict-mode observations retain those earlier refusals.
The later ordinary signature must fail in `wrong`, preventing an earlier refusal
at `twice` from falsely satisfying the regression. See
[fixtures and attribution](../tests/fixtures/cpu-acceptance-priority/README.md).

## Validation and reproducibility

- Standard gate: **1,183 passed / 0 failed / 49 ignored** at the clean tested checkpoint.
- Focused executable-unsafe target: **9 passed, 0 failed**; strict target Clippy passed.
  These preliminary receipts identify dirty `b7649af…`, before committing the tests;
  they are not represented as clean-commit runs. The full gate renews the tests
  at the clean checkpoint above.
- 109-case renewal: **344 commands** (109 upstream checks, 109 strict checks,
  109 executable phase probes, 17 refusal generations), plus an earlier independent
  109 executable phase probes at unchanged production source.
- Six-case comparison matrix: **144 command observations**; runtime, phase,
  strict refusal and generation observations remain separate in the receipt.
- Full 1,302-row census was not rerun: only tests/harness/docs changed. Its last
  measurement at `59ac3fdb…` remains **227 non-IO differences: 109 acceptance +
  118 differing rejections**, with **six separate upstream oracle failures**.
  None is silently subtracted by the new causal classification.

```sh
source "$ACTIVATE"
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test executable_unsafe -- --nocapture
cargo clippy --locked --features extended_observability --test executable_unsafe -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
python3 scripts/checking-census/acceptance.py --reference "$UPSTREAM" \
  --binary "$BEND" --phase-probe "$PROBE" --census docs/cpu-lambda-arm-census.csv \
  --build-receipt "$BUILD_RECEIPT" --output "$FRESH_OUTPUT"
```

The [runner instructions](../scripts/checking-census/README.md) retain exact probe
build requirements. Rust debug overrides reduce host build-cache size and do not
change Bend C/O3 runtime profiles. The build receipt binds binary/probe hashes to
the clean commit and Cargo feature fingerprint. Upstream pin remains
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`; Cargo.lock remains
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.

Three initial focused attempts failed because drafted assertions used the wrong
separator and a control spelled reflexivity as an undefined name. Those attempts
are retained privately, corrected controls passed, and no production defect is
claimed from them. A publication-preparation path check also caught an old binary
path before anything was copied into tracked evidence. Raw logs, generated
programs, historical failures and the lossless binary snapshot remain outside Git;
the small reviewed receipts and source are remotely preservable.

Disk preflight/stop floors remain 11/10 GiB workspace and 1 GiB temporary space.
Measured minimums were **11.481 GiB workspace / 4.850 GiB temporary**; see [validation receipt](cpu-acceptance-priority-validation.json). No source/evidence was deleted and no cache reclamation was needed.

## Remaining work and decisions

The next bounded acceptance investigation should isolate `reg/nat_chain` and
`reg/nat_table` to determine whether their generated pattern depth contains
avoidable work while retaining both the 64-level pattern and 128-level input
guards. No compact representation or policy change is adopted here. Window/Audio
require an explicit implementation workstream, not a diagnostic substitution.

`nat_first` remains pending and unwaived; the six narrow approved wildcard
observations do not cover it. Full parity still includes the 118 recorded
rejection differences, existing host/foreign/F32.read/hole/resource gaps and
unavailable oracle lanes. The six unstable upstream cases were not rerun, and the
accepted sidecar lacking complete streams still receives no exact-pass credit.
Coverage links now total **444 linked / 858 unlinked**: 522 compiled candidates,
322 checker negatives, 13 interpreter profiles and one checking-only module remain
unlinked. These are applicability categories, not counts of missing runtime tests.
No new mid-normalization cancellation is claimed. Prior downstream API caveats
remain, but this checkpoint changes no production API. Supplementary all-target
Clippy failures were not rerun or cleared; no desktop/GPU work, installations,
merges or releases occurred.
