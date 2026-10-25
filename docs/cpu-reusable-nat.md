# Reusable Nat successor-tail parity

Tested source: `b9fc3a7febb2a1a0a3ddef53395ee3ee69847196`. The subsequent checkpoint changes reports and compact
receipts only. Upstream stays `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.
**The `1n++k` acceptance/runtime gap is fixed; full CPU parity remains incomplete.**

## Contract and correction

Pinned [numeric parsing](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2205)
consumes exactly one immediately adjacent `+` before ordinary term parsing handles
its tail. [Unary `+`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1788)
marks a reusable field. Thus `1n++k` means a successor with reusable predecessor;
it is not string append. Whitespace before the first plus retains infix parsing.

The token lexer uses maximal munch. The numeric parser now temporarily splits an
adjacent `++` token, leaving the second plus to existing binder machinery. It
restores the complete original token before propagating either success or failure.
Independent review caught why restoration matters: retained template bodies are
snapshotted after their initial parse and must replay the original token stream.
Two distinct specializations and a cache hit now match explicit spaced-tail syntax.
No checker rule, evaluator, term representation, sharing policy, runtime demand,
dependency, budget, guard or cancellation boundary changed.

## Results and limits

The unchanged `reg/cop_let_descent_escape.bend` now prints **`22\n`** in native Rust,
port C/JS and pinned upstream C/JS, twice each. Its direct upstream Node command
still fails with `TypeError: require is not a function`. Strict proof checking now
gets past parsing and refuses `IO`; this is separate from executable success.

Nine project-authored positives cover tight/zero/nested tails, whitespace after
plus, triple markers, recursive reuse, template replay/cache and string append.
Full normalized constructor graphs match the pinned interpreter in the Rust
integration test. The matrix separately captures the upstream oracle graphs and
checks those captures against the stored JSON; it is not a second port graph run.
Six negatives
retain affine duplication and infix rejection at all public check/eval/run/compiler
boundaries. Two negative diagnostics remain different: spaced addition's namespace
and F32's printed representation. Matching refusal is not exact diagnostic parity.

The pure String control has identical complete data and compiled C/JS output
`"abcd"`; native display still exposes `SCon` constructors. The generic repeated
unary-prefix case `1n++++k` remains rejected by the port and accepted upstream.
Both residuals reproduce byte-for-byte with the captured pre-fix binary as well.
They are recorded residuals, not new exceptions or part of this parser correction.
Literal expansion stays capped at 128 and expression nesting at 64. Tests preserve
129 rejection, malformed-tail refusal, ordinary affine and non-Data restrictions.

| Validation | Result |
| --- | --- |
| Standard gate | **1,150 passed / 0 failed / 49 ignored** |
| Format, strict production Clippy, all-feature build, Node provider | Pass |
| Focused parser/templates/value/runtime/normalization/aggregate checks | 87 passed |
| Checker work accounting / existing sharing and work-guard units | 1 passed / 7 passed |
| Focused integration strict Clippy | Pass |
| Runtime/checking matrix | 346 observations; deterministic repeats; no timeouts |
| Dedicated phase probes | 64 deterministic observations; phase-only evidence |
| Full checking census | 1,302 measured; all port statuses unchanged |

Matrix observation categories: `{"build_only": 20, "exact_execution": 98, "exact_interpreter": 18, "exact_rejection": 72, "full_graph_oracle_capture_port_graph_asserted_in_tests": 18, "generation_only": 40, "strict_IO_proof_refusal": 4, "strict_check_or_value": 36, "unresolved_Node_host_failure": 2, "unresolved_diagnostic_difference": 36, "unresolved_native_String_presentation": 2}`.
Generation and build observations are distinct from execution. Rejected negative
programs never generate runnable artifacts; downstream execution is unrun.
Supplementary all-target Clippy was not renewed; its older failures remain open.
No GPU or desktop checks ran. Existing public API and cancellation caveats remain.

Earlier validation attempts are retained: one test incorrectly equated allocation
IDs from two versus three unary markers, one assumed reference interning that the
parser does not provide, and Facet decoding of a deep oracle graph overflowed the
test thread before Bend evaluation. The final test compares full canonical graph
serialization byte-for-byte, following existing repository tests; no nodes are
ignored and no stack limit was raised. Strict Clippy's lazy `bool::then` correction
was applied without suppression. Those failed attempts are not counted as passes.

## Census and acceptance scope

Whole-inventory counts remain `{"acceptance_disagreement": 494, "both_accept": 366, "different_rejection": 191, "exact_rejection": 245, "oracle_or_infrastructure_failure": 6}`.
Exactly one port phase/stream changes: the original reusable-Nat fixture moves
from a load-time `String.append` pattern rejection to a check-time `IO` refusal.
All 385 historical IO exclusions retain exact observations. **300 differences remain
outside those 385**, including this newly exposed IO symptom; it is not automatically
subtracted or waived. Six upstream stack failures remain separate.

The baseline 302 classification remains 151 diagnostic candidates, 109 acceptance
mismatches and 42 other/preemption differences. Current residual categories remain
149 / 109 / 42. These are measured checking categories, not counts of executable
semantic defects. [Current per-case phases](cpu-checking-triage.csv) and
[compact census delta](cpu-reusable-nat-census.json) preserve source identities.

The original gains one explicit runtime link: **411 linked / 891 unlinked**.
Remaining unlinked applicability: 341 checker negatives, one checking-only module,
13 interpreter-only cases and 536 compiled candidates. Links include blocked,
partial and historical receipts; they do not establish universal parity.
The acceptance criterion remains unchanged. F32.read, error-order, erased-hole/TODO,
host/foreign/resource disagreements, unavailable oracle lanes and the incomplete
688-vector exploration remain unwaived. Next bounded work can address the generic
repeated unary-prefix parser gap; native String presentation is an independent gap.

## Reproduction and preservation

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo test --locked --offline --features extended_observability --test parser --test parser_templates --test cpu_reusable_nat --test kernel_open_closures --test kernel_normalization
cargo test --locked --offline --features extended_observability --lib reusable_nat
cargo test --locked --offline --features extended_observability --lib composition_work_tests
cargo clippy --locked --offline --features extended_observability --test cpu_reusable_nat -- -D warnings
node "$UPSTREAM/bend2/main.ts" "$FIXTURE" -o "$OUTPUT.c"
"$TEAMY_BEND_CC" -std=c11 -O3 "$OUTPUT.c" -lpthread -lm -o "$OUTPUT.bin"
"$OUTPUT.bin" --threads 1 --gpu off
"$BEND" run "$FIXTURE"
```

Port matrix C adds `-Wall -Wextra -Werror -pedantic` and uses `-pthread -lm -ldl`.
The standard integration helper's strict unoptimized C controls are separate from
this primary upstream O3 comparison. Node runs emitted JS directly; no Bun install.
The [committed census runner](../scripts/checking-census/README.md) specifies full
checking reproduction. Process bounds remain 30 seconds for matrix/oracle checks,
60 seconds for port census and 20 seconds for integration execution; the gate has
a 1,200-second deadline. Rust 1.96.0, fmt nightly-2026-05-24, Node 24.19, Clang 19.1.7,
PS 7.5.4 and lockfile remain frozen. Host debug settings do not change Bend budgets.

Gate preflight 11.32 GiB; minimum 10.26 GiB.
After reclaiming only completed inactive build caches, 11.37 GiB remains.
Source, unique failures and historical snapshots are preserved. Compact pre-fix
streams, the unchanged original and current transition are in Git. Bulk raw logs,
process receipts and binaries remain private local evidence outside Git preservation.
Tested CLI SHA-256: `39291b59e2e7d3ff6f373ed4cf1ca612877adbb58f03359d76203a0de18bfb8e`.

| Private evidence | SHA-256 |
| --- | --- |
| `full-gate.log` | `3e2991e3012cf51456f2e510943b7b2d7a41c5272c86edd94fd4267cb56a719c` |
| `full-gate.json` | `1e7804037ea81ee547af11d6d5124e2ac2a29b0eb23ea25e3d77088add09c765` |
| `focused-corrected.json` | `a79f33b2debe7d96e9d981bd6a079659d97b28f99de0c4aed71d3d84639bfa2d` |
| `budget-corrected.json` | `c0e7e4aaeab7251edd499cca743d6a79c08013ab41f5e325ea34232ba74cadb1` |
| `sharing-controls.json` | `7235a7a3af1265b50678bf8b091c2ddbca344b4f896495da5f3c2c8873843794` |
| `clippy-corrected.json` | `4c861fcf4aeb0d0fa6d046037841bb9374d906acca652d9777fdc909342d85ef` |
| `source-review.json` | `8e9df96f60f97601df1d7e6a7cfec34252a337cbb9013ca5d4d3f7d5b70e0f5f` |
| `census-review.json` | `d383aa8331cb0d1a40cd37699fe4adb040413613bf46f4ae12a98ce192181efe` |
| `residual-baseline.json` | `dd226a00e744c1b3c01f2075c5765f45b18517fbd75d30c1f6d2a80b317d25ec` |
| `snapshot.json` | `1c37637631ee60854807564bcd4a59c24d52e7020b34d9fb9eaf81ea6bfc2cde` |
| `matrix/receipt.json` | `7bb8a01104722cbcaca5c68d33f8d0499fd8497ec4d7d6410ee9932057d90781` |
| `matrix-label-clarification.json` | `51e18f12a7328361b68b1e90b3e9a7d9018de502f59be045fb25791ec0aa95f3` |
| `phases.json` | `0627374e53baed48f6b69b2c536bd9fb86e893aa85fc8ccd0b6bbd0cb5704e1f` |
| `census/observations.jsonl` | `7c076edff1e760edc1797e3f64e0a07a3216f70fe5ec3e3c95e5a0cd03d3c5f5` |
| `census/reconciliation.json` | `45e330496f8054b3d80fab55d7cee13ee1fe14828762225a6d3707ce0f1e9174` |
| `parser-focused.log` | `a2f037934f9abbc7cc4eaa90c855176cc2e50e5374a306fc1ba34b226010a091` |
| `focused-final.log` | `2d611f208b6c23223543c4a853b2e5dcced3e0b670a34293b46a07dfce4b0076` |
| `graph-stack-diagnosis.log` | `0a382b786dd0aa35e72326fd4175256c8f3422fa69c0af0f71b4f574a0d86813` |
| `budget-units.log` | `67ee23b2826a5d4e25f3633feadfc4f604c25e198cd41475da7c483baf1e982b` |
| `focused-clippy.log` | `4e95c15419f4c957f2942820fe371c24e8812a304deb8b7f04ca0f5455db7346` |
