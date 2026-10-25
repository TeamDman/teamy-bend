<!-- SPDX-License-Identifier: MPL-2.0 -->
# Compile-time application type diagnostics

Tested source `54bc100bafda627fadd0443592be0e6200235838`; upstream remains
[`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
The original `comptime/wrong_type` now reports the pinned Bool/Nat mismatch,
binder context, specialized definition and Bend source window exactly. The type
rejection was already correct; this fixes its diagnostic provenance. CPU parity
remains incomplete under the [acceptance criterion](cpu-parity-acceptance.md).

## Cause and correction

Freshening a lambda argument discarded the source needed by the existing
structured checker diagnostic. The first candidate retained application-child
origins, fixing `wrong_type` but exposing an adjacent macro-substitution location
error: `err_op_ns` pointed to its caller instead of the template body. The final
change also propagates actual recorded argument sources through compile-time
substitution, while preserving explicit occurrence sources first. Quantity
literals and the quantity type now record their real parser locations.

Pinned [`term_higher`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L710)
preserves an available replacement span. Pinned
[`infer-app`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3351)
checks the argument; the final
[`check-any` refusal](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3620)
uses that term's span. These rules establish the expected source, not a new
acceptance or evaluation policy.

Provenance remains external weak metadata. No shared reference is copied, and no
term identity, binder ID, evaluator, semantic charge or guard is changed. Unknown
identities are not promoted to explicit sources. Public parser/KernelError
message fields, Display, equality, signatures and error types remain unchanged;
optional structured diagnostic content and CLI formatting intentionally improve.
Derived Debug output can also reflect the private diagnostic metadata.
Newly formatted `invoke` failures use the existing `Ok(CliOutput)` boundary with
`exit_code = 1`, so external consumers must inspect that field. External callers
remain unaudited. Existing repository boundary controls retain nonzero exit,
writer failures, cancellation checks and no emitted artifact on rejection.
Cancellation coverage is at existing entry/diagnostic boundaries; it does not
establish mid-normalization cancellation.

## Comparisons and retained failures

The [matrix](cpu-comptime-type-results.json) records **442 commands: 434 matching and eight strict-IO exclusions**. There are zero unexpected outcomes or timeouts; repeated status/stdout/stderr are deterministic. This includes **216 exact negative diagnostic comparisons plus 48 separate port phase probes**, **60 actual executions** (56 generated C/JS and four native IO) and **20 pure normalizations**. All twelve negative roots reject during checking. Nine diagnostic differences close in this bounded fixture set; three adjacent controls were already exact. These counts describe different kinds of observations, not unique semantic features.

Six upstream originals are copied byte-for-byte with Apache attribution.
`wrong_type` and `err_op_ns` now match exactly; `dup_lone` and `err_capture` retain
their already-correct checking diagnostics. Authored controls cover ordinary
applications, multiline/imported/nested templates, repeated uses and quantity
atoms/types. [Compact pre-fix evidence](../tests/fixtures/cpu-comptime-type/before.json)
preserves the old streams and baseline binary/source identity. The failed first
candidate remains in the receipt; it was not relabeled as a pass.

Five pure controls normalize to `4n`, `8n`, `4n`, `6n` and `9`. Two unchanged IO
originals execute to `6\n` (`dup_many`) and `12\n` (`shadow_arg`). Native IO, pure
normalization and generated execution are distinct lanes. Direct upstream IO
interpretation is unrun; generated upstream C/JS provide the executable oracles.
Strict proof checking continues to reject IO. C uses the pinned O3 profile, one
worker and GPU off; port C also retains strict Clang warnings. All fourteen emitted
port C/JS artifacts are byte-identical to the pre-fix baseline. Emission alone is
not counted as execution.

The nearby list callback negative now reports matching types, binder context,
definition and source text, but **bundled Base line 390 differs from upstream
line 1017**. It remains an exact diagnostic mismatch, not an exception. Only its
current-port expected-output file was renewed; historical port/upstream files
and unchanged rejection assertions remain. Seven bounded refusal commands were
recorded on an intermediate candidate and repeated separately at the final source;
both identities are retained. No execution or strict-proof qualification follows
from those refusals.

## Validation and reproduction

The standard gate passes **1,240 / 0 / 49** (passed / failed / ignored), 183 result groups, exit 0, in 972.61 seconds. Formatting, strict production Clippy, all-feature build and real Node TCP/UDP validation pass. Pre-commit focused executable/template tests on the final candidate pass **24 / 0 / 0**; application-focused unit controls **4 / 0 / 0** and the final known-source control **1 / 0 / 0** pass in their separately recorded candidate runs. The full gate renews all of them at the clean tested commit. Targeted strict Clippy passes. These are separate runs, not an additive unique-test count. The earlier partial focused runs remain **1 / 1 / 0** and **5 / 1 / 0** failures at their recorded intermediate sources. The supplementary all-target lint failures remain historical and unrenewed.

Frozen host build overrides, distinct from Bend's C O3 runtime profile:

```bash
source "$TOOLS/activate.sh"
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --offline --features extended_observability --test cpu_comptime_type --test cpu_template_occurrences --test parser_templates --test cpu_list_for_each
cargo test --locked --offline --features extended_observability --lib application_
cargo test --locked --offline --features extended_observability --lib known_sources_require
pwsh -NoLogo -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo clippy --locked --offline --features extended_observability --test cpu_comptime_type -- -D warnings
```

The [validation receipt](cpu-comptime-type-validation.json) gives exact commands,
source/binary hashes, profiles, deadlines, failures and totals. New controls compare
plain/tracked errors, semantic work/depth/fresh counts, zero/exhausted budgets,
source priority and strong ownership counts. Existing template/shared-reference,
host-stack, packed-word, aggregate-work and cancellation regressions remain in the
full gate. No ignore, assertion, work/depth guard or execution limit was relaxed.
Independent source and evidence reviews are recorded. Git whitespace checking
finds only three inherited trailing spaces in original `#|` blank-source-line
expectations; preserving the original fixtures retains these bytes.

## Measured census and remaining work

The complete [checking census](cpu-comptime-type-census.json) renews all **1,302** originals at this tested source: **366 both accept, 377 exact rejections, 494 acceptance disagreements, 59 different rejections and six oracle failures**. Both-accept means checking status only. All **385 historical IO members are exactly unchanged**, permitting the same subset accounting; outside that subset the measured residual is **168 = 109 acceptance disagreements + 59 rejection differences**, down from 172. The remaining historical triage labels are 109 acceptance/rejection mismatches, 29 other rejection differences and 30 diagnostic-only candidates; these labels do not waive observations or establish semantic parity.

Only four port stderr streams change: `check/error_window_filled_param_use`, `check/error_window_param_use`, `comptime/err_op_ns` and `comptime/wrong_type` become exact rejections. Port status/phase and upstream status/phase/streams are unchanged throughout; the six oracle failures are unchanged. There is no oracle-variation adjustment in this census. The two error-window gains are checking-only evidence, not additional executable qualifications. The fresh missing-IO symptom count is 386; it does not expand the historical 385-member subset. Earlier census files keep their original identities.

The [family ledger](cpu-comptime-type-evidence.csv) qualifies six originals. Overall coverage is **481 linked / 821 unlinked**: 286 checker negatives, 521 interpreter/compiled candidates, 1 checking-only module, 13 interpreter-only cases. Linked evidence is not universal parity; unlinked entries are not all missing runtime tests. Historical receipts keep their original tested commits.

The next bounded diagnostic gap is `comptime/err_rebind`: both implementations refuse rebinding a compile-time argument during loading, but upstream reports the expanded reference `inc` as an invalid pattern at the original binder line. The port still emits its generic template-rebinding error with a later source position. The fresh census preserves this reproduction; no fix or executable qualification is claimed here. The F32 builtin-argument/arity rows also remain unresolved, with strict-proof missing-definition behavior distinct from the upstream operand/arity rejection.

`nat_first` remains pending and unwaived; compact Nat/string lowering remains
unadopted. External invoke auditing and earlier parenthesis/error ordering,
resource, source-hole/TODO, F32.read, foreign, host/printer and supplementary
all-target lint gaps are not closed by this checkpoint. No new exception or
claim of overall CPU parity is made.

Minimum recorded free space is **11.048 GiB workspace / 3.589 GiB temporary**, with the 11 GiB preflight and 10/1 GiB stop floors enforced. Existing verified debug-cache/registry relocations from the preceding checkpoint remain in use. No cache was moved or removed in this task, and no deadline or disk-floor stop occurred.

Small source fixtures, expected diagnostics, sanitized before/after commands,
streams and hashes are committed. Bulk logs, binary/gzip snapshots, generated
artifacts and full census JSONL remain local under `$EVIDENCE`; Git publication
does not preserve them. Unique historical evidence is intact. No dependency or
toolchain change, installation, desktop/GPU work, main merge or release occurred.
