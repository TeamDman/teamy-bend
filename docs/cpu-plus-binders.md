# Reusable plus binders and checking diagnostics

Tested source **`e9d9b30c5029e18dc5600b407e5e0abab94a8b99`**, following checkpoint
`c89de79149d710e2c00059cffa0726031939cc67`, qualifies the five unchanged
plus_binder originals against **`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`**.
Four IO mains agree across the available CPU runtimes. The negative companion
exposed a diagnostic gap, now corrected through the existing structured failure
boundary. Full CPU parity remains incomplete; no semantic exception was added.

## Original applicability and results

The [five committed files](../tests/fixtures/cpu-plus-binders/README.md) are
byte-for-byte copies of the
[pinned parse fixtures](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse),
with their original comments, expected results and Apache-2.0 attribution.

| Original | Required observation |
| --- | --- |
| plus_binder_case | stdout `12\n25\n8\n` |
| plus_binder_lambda_do | stdout `58\n49\n` |
| plus_binder_nested | stdout `91\n` |
| plus_binder_row | stdout `13\n11\n10\n` |
| plus_binder_term | Checking rejection: unbound marker, exact diagnostic/location |

The four positives have IO mains. Actual C/JS generation succeeds in both
implementations; their applicability is established by generation, not inferred
from comments. Upstream C O3, upstream JS, port C, port JS and native Rust each
run twice: **40 exact executions**, empty stderr and exit 0. Direct upstream IO
fails eight times on the installed Node host with exit 1 and
`TypeError: require is not a function`. These remain blocked cells, not passes.
All four original books separately retain strict-proof rejection of `IO`.

Ten pure helpers within those unchanged books also have exact normal-form
coverage: area/twice, sq, sum2/two/pair/box, and sq/pair/kon. The pinned interpreter
produced each value twice. An internal regression normalizes the actual checked
executable core and compares exact text and parsed structure against the captured
TSV. This is helper normalization evidence; it grants neither strict proof
acceptance to the IO books nor standalone executable printing of function values.
No helper factoring replaces an original runtime fixture.

## Demonstrated diagnostic corrections

Both implementations already rejected plus_binder_term before execution.
Upstream reports an unbound marker as `x^-1`, its U32 context, definition and Bend
source excerpt. The port previously emitted a host-context `unbound variable x`
error. The pinned
[infer-variable rule](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3248)
and [variable printer](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1299)
explain both the rejection and the diagnostic spelling. The `^-1` suffix appears
only when the same name is already in scope; an undeclared `+missing` prints
`missing`. This distinction is covered, along with shadowing and competing markers.

The parser now records display-only marker IDs in the private source-provenance
index and anchors the marker's original span. The checker attaches presentation
metadata only after the same lookup has already failed. Semantic IDs, terms,
sharing, public KernelError text/equality and rejection are unchanged. An
untracked/programmatic variable retains the ordinary error. Context rendering
uses a clone with the remaining allowance; exhausted diagnostic work preserves
the original error. Bound markers still check normally.

Adjacent tests exposed a second exact-output detail: the pinned
[loader](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1023)
blanks consumed import headers before capturing source spans. Diagnostic source
now follows that rule while preserving other original bytes, CRLF, line numbers
and missing-final-newline behavior. The parsed source, input files and loader
acceptance are unchanged. Nearby false-proof and constructor controls prevent
this correction from being specific to a marker fixture.

Executable compilation now retains loader origins/canonical input paths and
uses the same checked diagnostic writer for classified load/check failures.
These rejections happen before output staging or native toolchain execution.
Regression tests cover C, JS and native targets, absent outputs and existing
outputs with `--force`; failed compilation leaves existing contents intact.
Cancellation, write/flush errors and ordinary host-error fallback remain tested.
Pure-data compilation and public loader signatures are unchanged.

The existing public `CompileArgs::invoke` status contract now covers these
classified executable-checker failures: it can return `Ok(CliOutput)` with
exit code 1 instead of `Err`. Repository production dispatch propagates that
output, and `src/lib.rs` reads its exit code before rendering and returning to
main. External consumers remain unaudited and must inspect `exit_code()`;
no signature change or blanket compatibility claim is made.

The final negative matrix contains the original plus eleven authored controls:
**216 observations**, including 96 exact checker/CLI stream-status observations,
24 checking-phase probes and 96 C/JS generation refusals without artifacts.
It covers direct, field, imported, template, shadowed, multiple and undeclared
markers, CRLF/EOF, and two adjacent import-excerpt diagnostic classes.

## Validation at the exact source

| Check | Result |
| --- | --- |
| Standard format, strict Clippy, build, provider and tests | **896 passed, 0 failed, 49 ignored**; 98 result-bearing targets; exit 0 |
| Focused candidate controls | 286 passed, 0 failed, 1 ignored |
| Four original IO mains | 40 exact executions; eight direct-oracle host blocks |
| Original negative and adjacent controls | 216 observations pass |
| Pure helpers from unchanged IO books | Ten exact core/text normal forms; twenty oracle observations |
| Existing exact normalization suite | 106 cases, 1,200 observations: 842 successes, 352 printer/artifact refusals, six strict F32 exclusions |
| Constructor/origin and Nat/List matrix | 42 cases / 588 observations: 31 exact checking negatives, three exact parse/load negatives, eight positive controls |
| Previous local-binder and rewrite matrices | 60 observations each pass |
| Earlier false-list/brace and positive proof controls | 144 observations pass |
| No-main checks, compilation and named normal forms | 306 observations pass |
| Serial C/JS/native baseline | 50 exact executions |
| Corpus checking-status audit | 1,302 rows, zero transitions from de31e54 |

The focused candidate run preceded the source commit and records dirty base
`c89de79149d710e2c00059cffa0726031939cc67`; it is supporting evidence, not a clean
commit receipt. The full gate and final matrices record clean source
`e9d9b30c5029e18dc5600b407e5e0abab94a8b99`.

All final repeated comparisons are deterministic, without timeout or abnormal
exit. All 439 expected checker negatives still reject; the other statuses remain
366 accepted and 497 rejected. Status agreement alone is not semantic/diagnostic
parity. The 49 hardware/profiler ignores remain unrun. The three early focused
candidate failures are retained: a test import compile error, import-header
excerpt mismatch and undeclared-marker spelling mismatch. The final source
corrects each; none is hidden by relaxed assertions.

The previous **688-vector exploratory remainder stays unqualified**. Its
interrupted run, timing failures and full oracle table are unchanged historical
evidence. This batch did not execute or relabel it. Other runtime-negative,
demand, resource and host receipts retain their actual tested commits.

## Coverage and remaining work

| Category | Inventory | Detailed originals | Without detailed receipt |
| --- | ---: | ---: | ---: |
| Checker negative | 439 | 10 | 429 |
| Checking only | 12 | 11 | 1 |
| Interpreter only | 164 | 58 | 106 |
| Compiled candidate | 687 | 134 | 553 |
| Total | 1,302 | 213 | 1,089 |

These links include historical and unresolved cells, not universal passes.
There are 93 runtime, 99 interpretation and 21 checking originals. This source
renews seven runtime originals, all 99 interpretation rows and seventeen checking
rows; 86 runtime and four checking rows remain historical. Helper and authored
control rows do not inflate the upstream inventory.

The [acceptance criterion](cpu-parity-acceptance.md) remains unmet. Complete
lane/API review, original-program resource limits, native foreign/host support,
F32.read and competing-error-order requirements remain open. CPU work can
continue without desktop or GPU. The next high-value independent gap is the
three already-recorded rewrite checking diagnostics: rewrite_non_equation,
rewrite_ctor_brace and dead_motive_syntax. Their phase agreement still has raw
diagnostic differences; those are not waived by this batch.

## Reproduction, storage and frozen inputs

After sourcing the existing activation script:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --lib --test cli_checker_diagnostics --test cli_compile_executable --test cli_effects --test compiler_executable_c --test compiler_executable_javascript --quiet
cargo build --locked --all-features --bin teamy-bend --example compare_normalization
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <clean-pin> -OutputDirectory <fresh-directory> -BinaryDirectory <saved-snapshot> -Suite all -Repetitions 2 -RequireExactText
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <fresh-directory> -BinaryDirectory <saved-snapshot> -Suite plus-binders -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <fresh-directory> -BinaryDirectory <saved-snapshot> -Suite baseline -Repetitions 2
python3 <private-evidence>/marker-matrix.py
python3 <private-evidence>/constructor-compare.py --binary <saved-cli> --output <fresh-name>
python3 <private-evidence>/qualify.py --output <fresh-name> --snapshot final-snapshot
python3 <private-evidence>/local-binding-matrix.py
python3 <private-evidence>/diagnostics.py
python3 <private-evidence>/proof-regression.py
python3 <private-evidence>/audit.py
python3 <private-evidence>/constructor-summarize.py
python3 <private-evidence>/verify.py
```

Private cpu-plus-binders receipts preserve exact argv, hashes, raw streams,
statuses, timings and artifacts. Fixed-output scripts require fresh destinations.
The comparison scripts' optional `-BinaryDirectory` reuses an immutable retained
snapshot; defaults still copy binaries. Receipts continue recording actual binary
hashes. All final matrices here use the one saved source build.

The initial storage inventory found about 20 GiB in tool/evidence files, including
about 18 GiB of ELF binaries. Historical files and every failure case were left
untouched. Only byte-identical new generated artifacts were coalesced, preserving
all paths, content hashes and pre-coalescing metadata in a receipt. New evidence
occupies **65.7 MiB** of unique file data;
reusing snapshots avoided repeated large binary copies. No unique evidence was
deleted. Rebuildable inactive test/library/check-metadata caches alone were
reclaimed for build headroom; frozen offline dependency sources remain available.

Every build started above the unchanged 10-GiB reserve. Full-gate minimum was
**9.07 GiB**; final free space is
**10.03 GiB**. Child bounds are 30/60 seconds,
outer bound twenty minutes, workspace/tmp stop floors 2/1 GiB. Four build jobs,
debug=0 and disabled incremental caching are host storage controls, distinct
from upstream C's O3 and `--threads 1 --gpu off`. Port C retains strict warnings.
Future builds must recheck reserve before regenerating removed caches.

Rust 1.96.0, nightly-2026-05-24 formatting, Node 24.19.0, PowerShell 7.5.4,
Clang 19.1.7, dependencies and all Bend budgets/guards remain frozen.
Cargo.lock SHA256 remains
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
Upstream and teamy-docs `50e89d6897ce2a65167ef65b17c5fa83fe849158` remain clean.
No installation, push, desktop/GPU work or security-policy change occurred.

## Receipt identities

| Evidence | SHA256 |
| --- | --- |
| no-main-final/receipt.json | c9939d7508aeb3489ac677f77dca962dc6fca5ed097d4f5a85ea18a6acec7d57 |
| normalization/receipt.json | f3348efe4bc3d36a84d53b58a1f284b67e1c3136b9e2f12d37bc82ddc9b7a079 |
| helper-oracle/receipt.json | df0276628310e05eed8f055392543990db81be49d94fccfdd6eb00ee2e42f710 |
| audit/receipt.json | 298d6ca1cffd4d9385737e33c142bead3d1c1d6333f6dac1a69d9f3ce59c0813 |
| baseline/receipt.json | 1ec2509b524db1df72b3e16f3ffad09ebb479a0cd949cbe1f0db80dd4e1abd5f |
| runtime/receipt.json | 2a5d6c3a4913c44c2bf9f9cff212de39aa6277f5016111e3038e018d4edb1ebd |
| full-gate.log | bcc81842ec6bdb5fc520831b1ab5ec5b23dd684b7b12ce11a21a0dffda6febc0 |
| full-gate.json | 77fb2e56a37f8a97f3cb429334c82b5a111bf931708bc4cb5e9787b4884d80d4 |
| diagnostics/receipt.json | a3b4ec58b4e3ea15cbfe8750d5c8b8692509f3b0ede4c25069f37f2ff91cfce3 |
| local-binding-final/receipt.json | 3cdc914a84bccbf3555f61a4d7c54af39750459baa504ac74783e25875b69112 |
| marker-final/receipt.json | 1459a3b14cacbad48428bb663deeacac50e514624c0620b80f1b2aec66d1612e |
| constructors/receipt.json | 5b2fec940584a2e6b1adf2f5265fbd54069c0b73690d3c20b42d05605a9444c9 |
| constructor-summary.json | 76a59780add7a2f34b112747caf8dec9f4c782af53b0dab90aae08b2d788a6d2 |
| proof-regression/receipt.json | 6c6441798978c54a36972f57eed47e91078d0e55f2a273c38d3dea67589e1e38 |
| oracle-provenance.json | 767721822835cbd2835e567aca5285a5a20053c1b6a069755e8e682dc563efdd |
| storage-preflight.json | ea5a5c01de0d8c45a37d45a3b252a1a4d4f3dc498b6951c86a89fd712699958a |
| pre-gate-cache.json | a09318ca74f70cc0194393a9f5432f3cdc3e3cb92227045a3e2f7a91f26ad263 |
| focused.json | 8779ba817f905a818fb7222af311f318fd24fa3f6ab5285f624d333a618c45a4 |
| focused-corrected.json | 0c03e887e7eb48866f10a530eb19d5d63115678ed65b3f04006f93e15e865dcc |
| focused-final.json | d3e2eb10835ebbba6b4f2b2292698e7aa8104cfd84c0f14ba781018d7929d3ee |
| focused-complete.json | c317e2b731f4408d0a93e3cacbbc7cf262714bb803a61d3905bc16cba57e76a6 |
| compact-artifacts.json | 34c32bc2aa6d2159989f35977085ccb7e6f72b36cd9e7222cae971b706e7a377 |
| storage-final.json | d0a0a4316a84d6e6a64f6da2230359876cb11b5652add428748314520624898b |
| final-cache-preflight.json | 43b038ea6ae2707d7d16cd1e251ee27b4342496e40cd10125361a8b0e70b78e0 |
| final-library-cache.json | 7ace3722135690e580e793bc403b2eebac8022f9653ba4ecf2fb8dbc4b5e931f |
| final-check-metadata.json | ba1a81d875e8304b42f5367a240d964924a06b2053bf722bd180e9f199416cf6 |
| final-cache-complete.json | afc4c46703f5f820bf89948810b8879f258dbfde43bc84fa63f80c0c960fda6c |
| invoke-callers.json | e1c85b3f0b75d551842210b5600db3ac1a2fc6189874231650bcfd6cb11ec102 |

CLI SHA256: `5f8eaa3a3988a3df4976bbc714f274780c3ae6821a979299de539929ff3e8917`.
Normalization probe SHA256: `f81091899bebfd920cd8ef11a511f1b2fb43bfc8cf4f892649d80d9233347eb8`.
The later documentation checkpoint reports this tested source; it does not claim
a second full-gate execution. No task-owned test remains running.
