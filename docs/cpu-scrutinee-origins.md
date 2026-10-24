# Scrutinee occurrence diagnostics

Tested source **`1175aa37f317db553a2e596d16d0089297f1fa40`** closes the substituted, consumed and out-of-order
scrutinee diagnostic family against pinned Bend
**`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`**. Full CPU parity remains incomplete.

## Cause and correction

All eleven [unchanged originals](../tests/fixtures/cpu-scrutinee-origins/README.md)
already rejected during parsing/loading in both implementations. The port reported
a generic flattener error at a later parser position, with host context instead of
the pinned language diagnostic. It also lost the individual use's source when
substitution made several occurrences share one replacement term.

The pinned [body_sub](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2653)
preserves the occurrence span while substituting patterns. Its
[match_flatten](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2680)
selects an invalid variable's use location, an already destructed constructor's
**enclosing match** location, or a computed expression's location. Destructuring
lets create a virtual match at their RHS span in
[body_flatten](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2772).

The private surface Scrutinee wrapper now retains a use anchor independently of
its substituted TermRef. Loader-owned weak anchors record enclosing match locations.
Structured flattening causes select the existing language diagnostic boundary.
The same core substitution, binder IDs, pattern traversal and term allocation are
used. No metadata enters core Term, serialization, evaluation or checked artifacts.
A regression verifies that distinct use anchors still share the exact replacement
allocation without adding strong term edges. The public ParseError message and
position remain unchanged; no public type or method signature changes in this batch.

Twelve authored probes cover literal and bare-global matches, multiline constructor
and variable spans, consumed and substituted uses, order, destructuring, imports,
valid template syntax, CRLF and missing final newline. Earlier computed-scrutinee,
rewrite, local-binder, Nat-constructor and hole-order failures are renewed.
Tests also cover text/JSON/CSV CLI invocation, C/JS/native executable refusal,
existing output preservation, cancellation, writer/flush errors and public-loader
compatibility. Existing guard, term-identity, checking-work and runtime regressions
remain in the full gate. Runtime demand and source-hole refusal are unchanged.

## Results at the tested source

| Evidence | Result |
| --- | --- |
| Focused library/CLI tests before the source commit | 246 passed, 0 failed, 1 ignored |
| Standard gate at the clean source commit | 915 passed, 0 failed, 49 ignored; 98 result-bearing targets |
| Format, strict Clippy, all-feature build, Node provider | Passed |
| Eleven original + twelve authored + eight historical negative cases, twice | 620 observations: 558 exact stream/status and 62 phase probes |
| Phase probes | 52 parse/load and 10 checking observations; no program execution |
| C/JS/native executable generation on these negatives | Exact refusal, no output artifact; no runtime pass inferred |
| Positive worker/no-main controls | 14 observations passed |
| Erased-hole/TODO controls | 18 observations preserve three unresolved disagreements |
| Complete 1,302 checking-status audit | 439 expected negatives rejected; other 366 accepted / 497 rejected; zero transitions from 75085cb |

All repeated matrix and control observations are deterministic. Each comparison
child is bounded at 30 seconds; each audit child at 60 seconds; outer runs at
1,200 seconds. No timeout, cancellation, or disk-floor stop occurred. The 49 ignored
tests remain ignored (47 hardware, two optional profiler controls). No new GPU
hardware or desktop validation is claimed. Existing broad runtime/normalization
receipts retain their historical commits; the full gate renews repository regression
assertions, including the selected 399 applications, not the unqualified 688-vector
exploratory remainder.

The upstream C compiler never reaches generation success for these negatives;
there is no new C-oracle runtime profile or math-library claim. Prior O3 runtime
receipts remain historical. Host Cargo debug overrides below do not alter Bend
execution optimization policies.

## Unresolved and next coverage

The [earlier five open probes](cpu-computed-scrutinee-open-evidence.csv) remain
historical evidence. Constructor and bare-global diagnostics now match exactly.
The [current open ledger](cpu-scrutinee-origins-open-evidence.csv) retains:

- A named hole erased by beta reduction: upstream prints `0` with exit 0; the port
  refuses the source hole with exit 1 before a checked artifact can escape.
- Erased and directly used TODO: upstream permits provisional checking then refuses
  incomplete source at its CLI boundary; the port refuses during source/goal checking.
  Both exit 1, but phase and diagnostics differ.

These are **not approved exceptions**, and matching source-hole refusal was not
weakened to imitate upstream acceptance. Any proposal to accept erased holes or
provisional TODO certificates needs a separate explicit contract review. It does
not block independent CPU work.

Single bounded probes, kept outside detailed qualification, identify the next gaps:
`flatten/error_window_matched_scrutinee.bend` reports expected Type / observed One
at the returned variable upstream; the port reports a generic expected-datatype
error with host context. `flatten/arity_mismatch_000` and `_001` reject at parse/load
in both, but upstream names the required two patterns at the case row while the
port reports its generic arity error at the later parser position. The originals
are unchanged. These three cases are **not counted as qualified**.

Source `1175aa37f317db553a2e596d16d0089297f1fa40` closes substituted, consumed and out-of-order scrutinee diagnostics
for eleven unchanged originals, plus constructor and bare-global matches. Per-use
surface metadata preserves source locations without changing core terms, sharing,
acceptance, evaluation, approved demand semantics or safety limits.
[Report](cpu-scrutinee-origins.md).

Full gate **915 passed, zero failed, 49 ignored**. The 31-case repeated matrix
contains 558 exact stream/status observations and 62 separate phase probes;
14 positive controls pass. All 1,302 checking statuses remain unchanged.
The three erased-hole/TODO controls remain unresolved and unwaived. Single probes
also retain matched-value checking and row-arity diagnostic gaps, not qualifications.

Detailed coverage: **231 originals / 1,071 without detailed receipts**: 411 checker
negatives, one checking-only module, 106 interpreter cases and 553 compiled
candidates. Eighteen checking rows are current; 21 other checking, 99 interpretation
and 93 runtime rows retain historical identities. Linked receipts include unresolved
cells and do not establish universal parity. The incomplete 688-vector exploration,
host/resource/foreign/F32.read, runtime-error-order/API/lane requirements remain open.
External invoke consumers must still inspect exit_code; no external audit is claimed.
Next: reconstructed matched-value and row-arity diagnostics. Frozen inputs/policies;
no installations, pushes, desktop or GPU work.


## Reproduction and storage

Use the pinned environment: Rust 1.96.0, formatter nightly-2026-05-24, Node 24.19.0,
PowerShell 7.5.4 and Clang 19.1.7. Teamy docs remain pinned at
`50e89d6897ce2a65167ef65b17c5fa83fe849158` and upstream is unchanged.
Cargo.lock remains SHA-256
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.

```sh
source <tools>/activate.sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked -p teamy-bend --test cli_checker_diagnostics --test cli_compile_executable --lib
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
python3 <evidence>/matrix.py --name matrix --binary <evidence>/final-snapshot/teamy-bend
python3 <evidence>/audit.py
python3 <evidence>/controls.py
```

These temporary host build settings avoid new debug/incremental cache growth;
repository profiles, dependencies and toolchains are untouched. Existing bounded
runners record actual commands, environments, output bytes, hashes, timing and
disk samples privately. The source snapshot was captured after the standard
all-feature build and provider check, while tests were running. Its SHA-256 is
`0909eaedf796bec672083c34bc4ddd72e17e3089eee32a6d54d1c3e5093a2b57`.

Builds require 10 GiB free before starting, with task stop floors of 2 GiB workspace
and 1 GiB temporary storage. Preflight helpers initially reported insufficient
reserve after dependency rebuilding; **no blocked build was started**. Reclaiming
inactive Cargo test executables, libraries/metadata, build-script and procedural-macro caches restored
the reserve. Source, all unique evidence, old failures and historical snapshots were
preserved. Saved historical binaries dominate storage; rebuilding frozen dependencies
consumes additional cache, so reserve checks remain necessary before future builds.
The gate started with 10.03 GiB free and reached a minimum of
8.78 GiB; temporary storage stayed above 4.38 GiB. Final free space is
10.03 GiB. Exact byte measurements are in the checkpoint receipt.

The first authored template probe used invalid generic syntax and failed earlier
than intended. Its output is retained separately; the final probe uses valid `~`
syntax, reaches the intended scrutinee rejection, and is the one qualified above.
No failed or partial exploration was relabeled passed.

| Private receipt | SHA-256 |
| --- | --- |
| full-gate.log | `d878ebb13bfb4d028277eec549c54a82fe320a4c41d8a0134bb792fa53fed3cf` |
| matrix/receipt.json | `b4adf03e5fdba838dfb85b7d7005b5e004d973746f72f1f35e5d63bda05292cf` |
| controls.json | `f016c6bf4b37bcf8e2e0dca544bf79564eb7d373fe74230202b68889780de604` |
| audit/receipt.json | `92f342d5d5b3a115e6ea056e74e12f33de38266835aa0f73911b4d932b143cd9` |
| original-hashes.json | `1652104de3bb35d848763979c634e2a023178bce02aa74b9fa9e195b221d35fa` |
| next-gap.json (unqualified probes) | `4319c2bbe3782c1a9d8f2c04a891e9063ef31e678c8b8e7d6c763f725aa2562a` |

The [acceptance criterion](cpu-parity-acceptance.md#required-evidence-before-calling-cpu-parity-complete)
is unchanged. The existing external invocation exit-code and executable-emitter
compatibility caveats still apply; no external consumer audit is inferred.
