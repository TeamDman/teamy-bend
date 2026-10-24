# Rewrite proof diagnostics and occurrence provenance

Tested source **`abd6d22b1a0d53420a6f60423b381b37137cccc3`** closes three demonstrated diagnostic gaps and
qualifies five unchanged negatives against Bend
**`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`**. Full CPU parity remains incomplete.
No acceptance, evaluation, demand or resource-policy change is included.

## Findings and correction

| Original | Actual phase | Before | Validated result |
| --- | --- | --- | --- |
| page/rewrite_non_equation | Checking | Correct rejection, host-style non-equation error | Exact expected/observed type, context and evidence-use source |
| check/error_window_rewrite_proof | Checking | Correct type rejection, missing substituted proof-use source | Exact diagnostic and source |
| check/missing_ih_rewrite | Checking | Already exact | Exact renewed qualification |
| grade/reject_rewrite | Checking | Correct rejection, host-style function-type error | Exact normalized stuck-rewrite type, context and application source |
| state/rewrite_goal | Checking | Already exact | Exact renewed qualification |

The [original copies](../tests/fixtures/cpu-rewrite-proof/README.md) retain every
upstream byte, comment and Apache attribution. Independent load/check probes
establish the phases. Check, strict eval, run and executable C/JS/native compilation
all refuse before program execution; compilation creates no artifact. A C runtime
optimization profile is inapplicable here: no C compiler is invoked. This batch
makes no runtime-execution or normalized-main claim for the rejected programs.

The pinned [rewrite rule](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3597)
reports the inferred evidence type and its use site. The
[application rule](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3350)
rejects a non-function before checking its argument, reporting the original
inferred type at the application source. The port now attaches this structured
information to its existing errors. Presentation normalizes types/context using
only a clone's remaining work allowance; exhaustion retains the underlying error.

The private provenance index already retained equality-operand occurrences.
It now also retains anchored rewrite-evidence and motive-lambda body occurrences.
Substitution may share one replacement between different uses; each use still
has its own source. Independently sourced replacements retain their source.
The [pinned substitution](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L710)
and [rewrite parser](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1833)
support this behavior. Multiline probes confirm that a proof-binder error points
to its **use**, even though the original fixture comment suggests its binding.
Application origins follow the function's starting source, including a
parenthesized function on another line, as in
[parse_grow](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1989).

No Term/Body field, semantic ID, strong term edge, serialization, evaluator,
sharing rule or work counter changes. Tests compare tracked/untracked terms,
errors, fuel, depth and fresh IDs; replacement sharing and metadata cleanup are
explicitly checked. New first-error and exhausted-presentation controls retain
existing safety budgets. Imported, template, nested, annotated, parenthesized,
CRLF and multiline cases preserve actual source ownership.

## Boundary and public API implications

The established language-diagnostic boundary still returns status-1 CliOutput
for classified failures. Repository dispatch propagates it, and `run_main` reads
`exit_code()` before emitting the result. No public signature or new boundary is
introduced. **External consumers remain unaudited:** additional classified errors
now return `Ok(CliOutput)` carrying status 1 instead of a host `Err`; callers must
inspect the status. Underlying KernelError messages/equality are preserved.

Focused tests cover text/JSON/CSV failure output, write/flush errors, cancellation,
untracked fallback and C/JS/native compilation with absent and existing output.
Existing files remain byte-identical under `--force`; no compiler staging occurs.

## Validation at the tested source

| Check | Result |
| --- | --- |
| Standard format, strict Clippy, all-feature build, Node provider and tests | **905 passed, 0 failed, 49 ignored**, 98 result-bearing targets; exit 0 |
| Five original negatives and fifteen complete adjacent cases | 400 observations, included below |
| Five preceding rewrite diagnostic controls | 100 observations, included below |
| Combined negative matrix | **500 observations**: 450 exact stdout/stderr/status; 50 phase probes |
| Actual phase probes | 48 checking, two parse/load; no execution started |
| Positive proof/rewrite_type_family | Four checking observations; accepted, no main, no runtime claim |
| Checking-status audit | All 1,302 statuses unchanged from 4c90907; zero timeouts/transitions |
| Existing expected checker negatives | 439 rejected |
| Other corpus statuses | 366 accepted, 497 rejected |

All final comparison-lane repetitions are deterministic, exit 1 with empty stdout,
and finish within 30 seconds per child; the phase probes exit 0. Every attempted executable emission refuses
without an artifact. The audit uses 60-second child bounds. Hardware/profiler
ignores remain unrun. The focused dirty candidate passed 257 tests with one
existing ignore; it supports the clean gate but is not relabeled as a clean-source
receipt. Its first run failed only a new test expectation (`Usage` versus the
existing `Quant` spelling); the failed receipt survives separately.

Broad historical normalization/runtime matrices are deliberately **not** relabeled
as current. Relevant tests ran in the full gate; old detailed lane receipts retain
their original commits. The previous 688-vector exploration remains incomplete
and unqualified. No new dependency, toolchain, pin, guard, budget, cancellation,
security-policy or approved demand-contract change occurred. No install or push.

## Remaining gaps and applicability

Three exploratory controls contain an unfinished `?unused` argument. The pinned
checker reports the non-function error first; the port's existing whole-book
`inspect` rejects the hole before rule checking. Both reject, but stderr and
first-error ordering differ. These are retained **unresolved** observations, not
an exact match or a new compatibility exception. Complete ill-typed argument
controls do match the required function-before-argument rejection. This batch
leaves whole-book incomplete-proof preflight unchanged.

The untouched `check/motive_suffix_join.bend` also rejects in parse/load in both
implementations. Upstream identifies a computed match scrutinee at its source;
the port reports the flattener failure with host context and a later parser
position. This separate provenance path remains unqualified. It is the next
concrete independent gap: propagate computed-scrutinee source/structured failure
through flattening, preserving rejection, then cover adjacent computed matches.
No user decision is needed for the completed rewrite batch.

Detailed inventory coverage is **218 / 1,302 originals**, including historical and
unresolved cells. The five newly qualified originals increase this count from
213; authored probes do not inflate it. There are **1,084 without detailed receipts**:

| Applicability category | Inventory | Detailed originals | Without detailed receipt |
| --- | ---: | ---: | ---: |
| checker_negative | 439 | 15 | 424 |
| checking_only | 12 | 11 | 1 |
| interpreter_only | 164 | 58 | 106 |
| compiled_candidate | 687 | 134 | 553 |

The primary checking ledger has 26 originals: nine carry this source (five new,
three preceding rewrite negatives and the positive no-main proof). Seventeen
checking rows, all 99 interpretation rows and all 93 runtime rows keep historical
identities. These overlapping categories are not a parity percentage. The
[acceptance criterion](cpu-parity-acceptance.md) remains unchanged and unmet:
complete lane/API review, original resource failures, native foreign support,
Bun/direct-IO host limits, F32.read and competing-error requirements remain open.
CPU work does not require desktop or GPU.

## Reproduction, evidence and storage

Use the existing activation script, then:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --lib --test cli_checker_diagnostics --test cli_compile_executable --test kernel_core
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
python3 <private-evidence>/matrix.py --name <fresh-directory> --binary <saved-cli>
python3 <private-evidence>/controls.py
python3 <private-evidence>/audit.py
python3 <private-evidence>/verify.py
```

The private cpu-rewrite-proof directory preserves exact command arrays, reference,
source/binary hashes, byte streams, exits, phases, timing, artifact absence,
initial failures and every bounded-run receipt. Fixed-output scripts require a
fresh destination. One saved CLI snapshot is shared by all final comparisons;
historical binaries and unique evidence were not copied or deleted. No generated
C/JS executable duplication is needed for checking refusals.

Repeated cache pressure comes from the preserved historical ELF evidence (about
18 GiB) plus rebuildable Cargo profiles. Preflight initially found less than the
unchanged 10-GiB reserve; test-executable/check-metadata cleanup alone could not
restore it, so inactive library caches were reclaimed. A final rebuildable
top-level library archive restored the additional 32-MiB cushion after hardlinks
made earlier reclaim estimates insufficient. No build bypassed preflight.
The full gate began with 10.03 GiB, reached a minimum of
8.87 GiB, and final headroom is 10.04 GiB.
New retained evidence is about 27.8 MiB. Source and all
historical/unique failure evidence survive. Outer bounds remain twenty minutes,
with workspace/tmp stop floors of 2/1 GiB. Debug/incremental overrides are host
storage settings, distinct from Bend/C runtime optimization and work budgets.

Rust 1.96.0, nightly-2026-05-24 formatting, Node 24.19.0, PowerShell 7.5.4 and
Clang 19.1.7 remain unchanged. Upstream/docs trees are clean at their frozen pins.
Cargo.lock SHA256 remains
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
Saved CLI SHA256: `058c1493b5a173958e334f40802506a6da25512997f74bcfb122462c6385d96a`.

## Receipt hashes

| Receipt | SHA256 |
| --- | --- |
| matrix/receipt.json | `ad1830fb9f544473294532fa494c4b86713b0c65c5e792c41dcc33ef82ab2519` |
| controls.json | `9519b7a2375d8191874114fd9a62d2e1c12828b56b3dc7c3f81751afa955a3e1` |
| audit/receipt.json | `77195bc4be5ee401e9f9b17824c3f32f6ecf494e05274d51008b7a76b81ece6e` |
| full-gate.json | `06f448745fa52be3ddf75008e651d937c680a160fc0d72f278ec767de5cdf57b` |
| full-gate.log | `12fd846d9c44060f02c31507e8e698214ccf6e597cdb68790e9332a7ea26430f` |
| focused2.json | `bf62776a435c170073dd3ab294290c3ada726b2d8206f6de1fdf1718d97ac746` |
| raw-oracle.json | `595e51e9d83db592be807dc7bb0bc15c764c4e1c05bc2115cd935dee76b7c9ea` |
| initial.json | `95962f23c60473b66c2853cb9eccb26a9a0e469f7c9bffd5e8aaccc09e670022` |
| original-hashes.json | `c606a10655d128d8a73af3c1b06ad998d2cd0318ea59bffe4017d92be34890e7` |
