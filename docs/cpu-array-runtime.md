# Typed array execution and ownership qualification

Tested source: `d1ba0e59c60d6f569df3e6b26329077123c08a47`. This batch changes tests and comparison tooling only.
Production code, dependencies, toolchains, upstream pin and safety limits remain
unchanged. Independent review and the standard gate bind to the final source;
documentation-only checkpoint identity is recorded separately.

The unchanged `run/array_fill_fold` and `run/array_poly_elem` originals now have
actual native Rust, upstream C/JS and port C/JS execution evidence: `140375` and
`102122`, respectively, with exit 0 and empty stderr. Unchanged `reg/array_keep_cow`
and `reg/array_copy_pair` likewise produce `70` and `36`. All five lanes ran twice.
This extends the previous native-only selection probes; it is not a new production
repair, whole-census renewal or proof-normalization claim.

## Contract and bounded controls

Pinned [Array operations](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L2123)
mask indices by size minus one. Depth zero is a single leaf. `Array.clone` creates
two independently usable owners; `get` and `clone` require Data elements, while
`swap` can move an affine Type element. Existing pure array tests already covered
related cases; this batch adds cross-runtime observations and the original fixtures.

Four authored executable controls observe:

- Reads at 0, size, U32 maximum, a wrapping swap and depth-zero storage:
  `10, 10, 40, 139, 7, 1`, as six ordered lines.
- Retained read/displaced strings and independently changed/original arrays:
  `root, root, new, root`, as four ordered lines.
- F32 negative-zero and finite 1.5 storage bits through clone/get/swap:
  `2147483648, 1069547520, 2147483648`. These are numeric bit observations;
  no NaN payload, floating math tolerance or general host-library guarantee follows.
- Affine closure cells moved through swap: `7, 9`. Copying a non-Data cell through
  `get`, duplicating an affine array owner, and supplying a Bool to a U32 array
  are separate exact checker rejections with no emitted artifacts.

Generated-C cleanup variants assert zero live words/blocks/tasks/continuations/
frames/depth after normal completion of the string, closure and F32 controls.
They preserve ordinary runtime limits and are separate from untouched O3 oracle
executions. This is accounting evidence for selected C programs, not general
native/JS heap inspection, a physical COW-layout guarantee or cancellation proof.
Native checked books are reused for a second identical invocation.

## Current results and retained failures

Final post-gate matrix: **198 observations / 48 repeat pairs / zero timeouts**.
Eight positives give **80 exact successful executions** across five lanes. Three
checker negatives give 27 exact rejection observations with no output artifacts.
The depth witness gives ten runtime refusals, including two unwaived port-C
diagnostic differences. The earlier all-features matrix independently agrees:
180 exact cross-profile stream observations and 18 identical generated port
artifacts; compile-result path fields necessarily name different evidence folders.

The unchanged depth-32 original still refuses safely in every executable lane.
Native, upstream C/JS and port JS report `bend: an array past the deepest block
class 31`; port C reports `teamy-bend executable C: array budget exhausted`.
The latter remains an unwaived resource/diagnostic difference. No huge depth-31
allocation experiment, safety-limit change or exception was introduced.

Strict port check/eval refuses F32 axioms or IO in the executable positives and
depth witness. Those exclusions are recorded apart from upstream checking and
actual executable success. Direct Node execution of upstream IO programs was not
retried; generated C/JS is the measured oracle route. No Bun, desktop or GPU lane
was added or claimed.

Preliminary authored float/string controls incorrectly destructured computed
values; both implementations rejected them. Their exact sources and receipts
remain preserved. Corrected controls use ordinary parameter scrutinees and do
not modify the originals. The first final matrix ran all observations but failed
its post-run harness assertion because successful port compilation emits a JSON
artifact receipt. The corrected harness validates that receipt's exact path,
target, byte count and provider artifact instead of assuming empty stdout.
The failed matrix remains historical evidence, not a completed qualification.

Standard full gate: **1357 passed / 0 failed / 49 ignored**,
exit 0. Formatting, standard strict Clippy, build and real TCP/UDP provider
checks passed. Focused dirty-source checks passed 14 tests and targeted strict
Clippy; the full gate renews the Rust tests at clean source. Hardware/profiler
ignores remain unrun, and old supplementary all-target lint failures were not
rerun or cleared. Minimum headroom was 12.143 GiB workspace /
1.349 GiB temporary. The gate tests use a non-Tracy
profile, distinct from the preceding all-features matrix. Both binary hashes
and complete corresponding observations are retained in the validation receipt.

## What remains historical or unqualified

No full checking census or 31-row rejection subset was rerun here. The last full
1,302-original census remains tied to `c5edea8a75022c1654fbf568629422cf73bee1d0`:
366 both accepted, 405 exact rejections, 494 acceptance differences, 31 differing
rejections and six upstream stack failures. Outside its fixed 385-row IO cohort,
109 acceptance and 31 rejection differences remained. The 31-row subset was last
renewed at `1a135e437d91c99072bbd880b9a6d43ceb984d2a`, all still different.
The separate historical 109-row acceptance classification's 92 executable-check
acceptances are not 92 runtime passes.

Receipt linkage is now 520/1,302 (previously 516); 782 entries remain
unlinked: 256 checker_rejection, 513 interpretation_and_compiled_candidates, 1 checking_without_main, 12 interpretation_only.
Links include failures and do not constitute universal parity. The four original
positive array runtime comparisons are newly established here; depth32 merely
renews an existing residual. No checking-census disagreement was closed.

The argument-recovery barrier and depth-refusal neighbor from the previous batch
remain unqualified. nat_first remains pending/unwaived, compact Nat/string
lowering unadopted, external invoke consumers unaudited, and Base coordinates,
F32.read, source holes, foreign/host/oracle limitations and supplementary
all-target Clippy failures remain open. No new mid-normalization cancellation
claim is made.

Next substantive original family: `reg/array_borrow_generic` and
`reg/array_borrow_generic_sum`, which exercise retained borrowed fields across
generic/product/sum layout conversions. Their source was inspected, but no new
execution receipt is claimed in this checkpoint. Full CPU parity is incomplete.

## Reproduction and preservation

Use the existing activation environment with Rust/Cargo 1.96.0, formatter
nightly-2026-05-24, Node 24.19.0, Clang 19.1.7 and PowerShell 7.5.4. Upstream remains
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`; docs remains
`50e89d6897ce2a65167ef65b17c5fa83fe849158`. Cargo.lock SHA-256 remains
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
Host build overrides: CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0,
CARGO_INCREMENTAL=0, CARGO_BUILD_JOBS=4, CARGO_NET_OFFLINE=true, BEND_GPU=off.
These are not changes to the generated Bend C optimization profile.

```sh
cargo test --locked --offline --features extended_observability \
  --test cpu_array_runtime --test compiler_executable_c_stable_arrays --test kernel_arrays
cargo clippy --locked --offline --all-features --test cpu_array_runtime -- -D warnings
cargo build --locked --offline --all-features
python3 scripts/array-runtime/compare.py --repo "$REPO" --reference "$REFERENCE" \
  --binary "$PORT_BINARY" --manifest "$REPO/scripts/array-runtime/cases.json" \
  --output "$NEW_EVIDENCE"
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Every child has a 60-second differential bound; integration program execution
uses 20-second bounds and its C compiler helper uses 45 seconds. The outer recorded runner monitors workspace/tmp floors
throughout execution. Source bundles, raw logs, compiled artifacts and binary
snapshots have verified local backups. Git holds the original fixtures, runner,
compact exact streams/hashes and review/validation receipts, not large binaries.
Public receipts replace explicit machine roots only; raw stream hashes remain.
No unique evidence was deleted or cleanup needed. The 11 GiB prebuild/10 GiB
active workspace and 1 GiB temporary floors remain unchanged. Blocked Library
delivery was not retried. Publication uses reviewed feature and non-force main
updates under the standing authorization; no release is involved.

[Exact validation](cpu-array-runtime-validation.json), [compact streams](cpu-array-runtime-results.json.gz), [coverage](cpu-array-runtime-coverage.json), and [evidence rows](cpu-array-runtime-evidence.csv).
