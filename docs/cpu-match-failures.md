# Empty-match, missing-case and unsupported-pattern diagnostics

Tested source **`d82417425b9c46f6ac7a63a39858987ca965b5bf`** closes this bounded diagnostic family against
Bend **`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`**. Full CPU parity remains incomplete.

## Causes and correction

All [29 unchanged originals](../tests/fixtures/cpu-match-failures/README.md) already
rejected in the port. Twenty-seven fail during checking; `unsupported_pattern`
and `invalid_assign_target` fail during parse/load. The port lacked structured
cannot-infer/missing-case reports and precise nested pattern locations, so it
emitted generic Rust host-context diagnostics instead of the pinned Bend report.

The pinned [inference rule](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3395)
requires an annotated term for these goal-dependent forms. The
[match rule](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3517)
reports the remaining constructors only after its ordinary domain, quantity and
live-empty-context checks. The port now attaches the corresponding structured
facts to the same errors. Rendering uses a clone with the remaining budget;
exhaustion retains the original refusal. Acceptance, traversal and safety budgets
are unchanged, including the distinction between live and erased empty binders.

[Flattening](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2710)
retains the enclosing match span on generated empty branches. A private MatchSite
now carries that origin through surface flattening and substitution. Statement
matches use the keyword location, destructuring uses its original expression,
and explicit lambda-match fallbacks retain the backslash location. Source anchors
for explicit lambdas also cover nested constructor patterns. The
[pattern parser](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1690)
reports the unsupported term and grammar without normalizing away the invalid form.
The public ParseError message and position remain unchanged.

One original exposed an additional context-order defect: parser IDs put local `c`
before freshly admitted pattern fields `a,b`. The pinned
[error renderer](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1450)
orders bindings by their context indices. A private context admission list now
produces `a,b,c`; semantic lookup and context-dead traversal still use the original
ID-ordered map. No core Term field, binder ID, serialization, sharing or evaluation
behavior changes. Regressions compare tracked/plain books, work/fuel/depth/fresh
counters, per-edge origins and pointer sharing, and prove exhausted presentation
cannot turn a refusal into acceptance.

Fifteen authored negative controls cover nested matches, multiline headers,
destructuring, literal matches, imports, templates, local-after-field context,
nested/annotated patterns, CRLF and EOF. Positive controls preserve immediate
lambda application, empty datatype elimination and a live empty context. The
corresponding erased-context original still rejects. Original fixture bytes and
Apache attribution are preserved, including one original trailing space flagged
by diff whitespace checks.

## Validation at the tested source

| Evidence | Result |
| --- | --- |
| Standard gate | 926 passed, 0 failed, 49 ignored; 98 result-bearing targets |
| Format, strict Clippy, all-feature build, real Node socket provider | Passed |
| 29 original + 15 authored + 8 historical negative cases, twice | 1,040 observations: 936 exact streams/status and 104 phase probes |
| Phase probes | 82 checking / 22 parse-load; no program execution |
| Executable C/JS/native generation | Exact refusal, no output artifact |
| Positive worker/no-main and new applicability controls | 32 observations passed |
| Erased-hole/TODO controls | 18 observations preserve three unresolved differences |
| 1,302 checking-status audit | 439 expected negatives rejected; other 366 accepted / 497 rejected; zero transitions from 77116c8 |

Every repeated matrix/control signature is deterministic. No timeout, crash or
resource-floor stop occurred. Matrix children use 30-second bounds, audit children
60 seconds and outer commands 1,200 seconds. The full gate took 313.24
seconds. The 49 ignored tests remain unrun: 47 hardware and two optional profiler
checks. CLI tests cover text/JSON/CSV, status-bearing failures, retained output
files, cancellation, write/flush failures and unchanged public load errors.

The final narrow precommit CLI rerun passed two test functions (22 filtered);
its working-tree receipt is not relabeled as a committed full gate. Earlier logs
retain two initial test-construction mistakes, the nested-lambda source gap,
and a clean-commit Clippy failure. The follow-up lint-only refactor passed the
full gate without warning suppression. The full gate renews all relevant regression
controls, including the existing selected 399 applications, and does not complete
the interrupted 688-vector exploration.

No C binary is generated for these rejection cases; no C optimization profile or
successful runtime claim is inferred. Existing O3 runtime receipts remain tied
to their historical source commits. Checking all originals for status does not
qualify their missing runtime, normal-form or exact diagnostic lanes.

## Remaining gaps and compatibility

[Three erased-hole/TODO differences](cpu-match-failures-open-evidence.csv) remain
unwaived. Upstream accepts a beta-erased named hole; the port retains source-hole
refusal. Erased/direct TODO reach upstream's post-check incompleteness refusal,
while the port refuses during source/goal checking. No new exception or change
to approved runtime demand semantics was introduced.

The next independent family is match-domain, mismatched/repeated-constructor and erased
scrutinee diagnostics. Single phase/stream probes are retained for
`page/match_not_datatype`, `page/empty_match_datatype`, `check/float_match_scrutinee`,
`check/error_window_match_repeated`, `check/erased_match_empty` and
`check/erased_match_ctor`. The two domain cases need the normalized function
domain in the report; the two constructor cases need the remaining datatype family and case-head source (the
pinned rule uses its constructor span); the two erased-scrutinee cases need the
message-only language report. Each still rejects with status 1 in both implementations,
but the port emits host context. They remain unqualified and their exact differences
are not waived. Correcting these boundaries need not alter erased-match safety or
numeric proof exclusions.

Detailed evidence now links **274 / 1,302 originals**. The 1,028 without detailed
receipts comprise **368 checker negatives, 1 checking-only module, 106 interpreter
cases and 553 compiled candidates**. Source hints are not established applicability;
not every unlinked fixture is a missing runtime test. The checking ledger has 82
rows, with 37 renewed here and 45 historical; the 99 interpretation and 93 runtime
rows retain historical identities. The [acceptance criterion](cpu-parity-acceptance.md)
still requires explicit lane decisions and qualified results, including unresolved
host/resource/foreign/F32.read/error-order/API cells. Linked rows may contain open
cells and are not a parity percentage.

The existing API caveat applies to these additional classified diagnostics:
invocation can return `Ok(CliOutput)` with exit_code 1. Repository dispatch forwards
that result and `src/lib.rs` inspects exit_code. External consumers have not been
audited and must handle the status; no new public signature is introduced here.
Kernel error message/equality and public parse errors retain their prior contracts.

## Reproduction and evidence

Host build settings, distinct from Bend runtime optimization profiles:

```sh
source "$TOOLS_ROOT/activate.sh"
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

The bounded private command receipts record exact argv, source hashes, raw bytes,
status, elapsed time and resource samples. Differential command shapes are:

```sh
node "$PIN_ROOT/bend2/main.ts" "$FIXTURE"
"$BINARY" --output-format json check "$FIXTURE"
"$BINARY" --output-format json eval "$FIXTURE"
"$BINARY" --output-format json run "$FIXTURE"
node "$PIN_ROOT/bend2/main.ts" "$FIXTURE" -o "$ARTIFACT.c"
node "$PIN_ROOT/bend2/main.ts" "$FIXTURE" -o "$ARTIFACT.js"
"$BINARY" compile "$FIXTURE" --executable --target c --output "$ARTIFACT.c.port"
"$BINARY" compile "$FIXTURE" --executable --target javascript --output "$ARTIFACT.js.port"
"$BINARY" compile "$FIXTURE" --executable --target native --output "$ARTIFACT.native.port"
```

Here the variables denote the immutable pin, exact saved binary and case-specific
paths in the private receipt. The separate phase adapter invokes pinned load/check
functions; it is not a stdout/stderr oracle. Public CSVs contain only compact hashes
and classification; raw logs and machine paths remain private.

| Evidence | SHA-256 |
| --- | --- |
| Exact tested binary | `5d9877b04d47efbc858f7d4e3a9a82a7aaaab3ac4d6ff0878c6105890121113a` |
| Lossless binary archive | `8603485430f2649c276b96366ff719159682d537dbdcca168179429bb3684efa` |
| Matrix receipt | `f50267560fb7d695c49305a6a8a8e11714fc27fb101e9c6ebecaadfca6f8e379` |
| Checking-status audit | `53eea2b42ddab4cd1ed2d6e4c7886ffbe23828af091a640eaa3cf19b22822092` |
| Historical positive/open controls | `ade1602a097eb90422707cd201da56870010c8dc0c0db736654e8c6b81f52a54` |
| New positive controls | `9985bfbf36b1cc5b982979ed9af695539ab9705402467facfa0e556471f894a5` |
| Full gate receipt | `2fc6fb1e675cdb30e3064b16d5b16edc83fa66e2b45ac044e44d7448b25b984e` |
| Full gate log | `a60d7b85998aee26405d1de311cd7d56d1e0775468e64469c6d1543a1b465a79` |
| Next-family characterization only | `de90f228dced5ee5fa0e1716c36f4c90e892cfe67701f70b2b5035d5f6a0904d` |

Disk preflight enforced a 10 GiB build-start reserve and preserved all unique
source/evidence. Only inactive, rebuildable Cargo caches were reclaimed. Full-gate
minimum headroom was 8.757 GiB; final headroom is
10.033 GiB. Historical executable snapshots dominate
retained evidence, so this task saves a verified lossless gzip of its new binary
(7,022,680 bytes versus 26,952,560 raw bytes),
with the exact working copy in temporary storage. Decompression was byte-checked
against the built binary; no historical executable or unique failure log was
removed. After those checks, only the two inactive Cargo build-binary hardlinks
were retired; both the mandatory 10 GiB reserve and the optional 32 MiB cushion
are now met. The temporary working binary and lossless archive remain available.
Historical named executables occupy 11.926 GiB after accounting for hardlinks;
future batches must keep the preflight and compact-snapshot safeguards.

Dependencies, Cargo.lock, upstream and docs pins, toolchains, parser/kernel/runtime
budgets, cancellation, network policy and approved demand semantics are frozen.
No installation, push, desktop or GPU work occurred.
