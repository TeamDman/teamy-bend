# Generic borrowed-array ownership qualification

Tested source: `a347a2d8f679d10ee21fdca3539deb398ec5e633`. This checkpoint adds tests, fixtures and a comparison
manifest. Production code, dependencies, toolchains and safety limits are
unchanged. Independently reviewed source is validated by the full gate and a
post-gate differential run; documentation-only checkpoint identity is separate.

The unchanged pinned `reg/array_borrow_generic` and
`reg/array_borrow_generic_sum` originals produce **120** and **2260** across native
Rust, upstream C/JS and port C/JS, each twice with status 0 and empty stderr.
These are newly established executions of the originals, not checking-only or
normalization evidence. No production defect was demonstrated in this family.

## Ownership and layout observations

Pinned [val_to/val_arms](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1765)
retains a destination borrowed root only when all contributing arm roots agree;
otherwise the source roots are marked owned (lines 1784–1804). The original sum
enters through One and rereads Two. An explicitly attributed Apache-2.0
derivative changes only its initial index to 1 and expected result to **1440**,
exercising the other entry arm without changing the original.

An authored generic control covers U32/String arguments and both initial arms.
It performs get, clone, opposite-arm swap, then gets the changed and kept cells.
The first read remains live across cloning. All four retained values—read,
displaced, current and untouched clone—are printed with tags, payloads and full
selected lists, for sixteen ordered lines. String/list formatting allocates while
other retained cells remain live. Both raw U32 and computed String payloads are
observed; results are not reduced to a single checksum.

Native checked books run twice to check invocation isolation. Separate generated-C
cleanup variants check zero live words, blocks, tasks, continuations, frames and
depth after normal completion of all four new positive fixtures. These variants
retain default limits and are distinct from untouched O3 oracle programs.
They establish selected runtime-accounting observations, not general memory-safety
proofs, cancellation coverage or native/JS heap inspection.

The port can specialize erased type arguments. Matching generic-source behavior
therefore does not establish identical upstream borrow ABIs or physical layout
transitions. Existing stable-layout and affine guards remain intact.

## Exact results and boundaries

Final post-gate matrix: **132 observations / 32 repeat pairs / zero timeouts**.
Five positives give **50 exact successful executions** across five lanes. Two
checker negatives give 18 exact rejection observations with no requested output
artifacts. The depth witness gives ten runtime refusals, including two unwaived
port-C diagnostic differences. All original streams and exit statuses are retained.
The baseline has 120 identical stream observations and 12 identical port
generated artifacts; its binary belongs to historical source `d1ba0e59c60d6f569df3e6b26329077123c08a47`,
not the newly tested source. Compile-result paths necessarily name different
evidence folders.

A new reused-read fixture and the prior non-Data-get control reject during
checking with exact diagnostics and no requested output artifacts. The prior
affine closure-swap control still executes successfully. Strict port check/eval
refuses IO separately from executable checking/runtime success. Pure proof
normalization and direct upstream Node IO execution are not qualified here.

The unchanged depth-32 original continues to refuse safely. Native, upstream
C/JS and port JS print `bend: an array past the deepest block class 31`;
port C prints `teamy-bend executable C: array budget exhausted`. Its two repeated
observations represent one case/backend discrepancy. This is an
unwaived resource/diagnostic difference, not an exact match. No deep allocation
policy, budget, guard or exception was changed.

Standard full gate: **1360 passed / 0 failed / 49 ignored**,
exit 0. Formatting, standard strict Clippy, build and real TCP/UDP provider checks
passed. Focused checks passed 12 tests; renewed targeted strict Clippy passed.
The full gate renews all three new Rust tests at clean source. Hardware/profiler
ignores remain unrun; old supplementary all-target lint failures were not rerun
or cleared. Gate minimum headroom: 12.113 GiB workspace /
1.296 GiB temporary. Final matrix uses the gate binary
identified in the validation receipt.

The first authored control cloned before reading; that successful preliminary
source and receipt remain preserved. Review strengthened it to retain the read
before cloning. The initial targeted lint run rejected a context-free test
assertion; its receipt remains. Adding an explanatory assertion message corrected
the lint without changing the condition or suppressing warnings.

## Current versus historical parity

The two original runtime qualifications and the bounded controls above are new.
Prior affine/Data and depth-32 observations are renewals. No full checking census
or rejection subset was rerun. The historical full census remains bound to
`c5edea8a75022c1654fbf568629422cf73bee1d0`: 366 both accepted, 405 exact rejections,
494 acceptance differences, 31 different rejections and six upstream stack
failures. Outside its fixed 385-row IO cohort, 109 acceptance and 31 rejection
differences remained. The 31-row subset was last measured at `1a135e4`, still
different. The separate historical 92 executable-check acceptances are not 92
runtime passes.

Receipt linkage is now 522/1,302 (previously 520); 780 entries remain
unlinked: 256 checker_rejection, 511 interpretation_and_compiled_candidates, 1 checking_without_main, 12 interpretation_only.
Links include failures and do not constitute universal parity. No checking-census
disagreement was closed by this coverage-only checkpoint.

The argument-recovery barrier remains blocked by nontrivial declaration lowering;
no safe reuse design was adopted here. nat_first remains pending/unwaived; compact
Nat/string lowering is unadopted. Base coordinates, F32.read, source holes,
external invoke consumers, foreign/host/oracle limitations and supplementary
all-target Clippy failures remain open. No mid-normalization cancellation claim
is added. Full CPU parity is incomplete.

Next substantive originals: `reg/array_clone_boxed` and `reg/array_heap_element`:
shared boxed elements, dropped/read clones and constructor tag preservation.
Their source was inspected, but no new execution of them is claimed here.

## Reproduction and preservation

Use the existing activation environment. Rust/Cargo remains 1.96.0, formatter
nightly-2026-05-24, Node 24.19.0, Clang 19.1.7 and PowerShell 7.5.4. Upstream remains
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, mounted docs remains
`50e89d6897ce2a65167ef65b17c5fa83fe849158`, and Cargo.lock SHA-256 remains
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
Host overrides: CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0,
CARGO_INCREMENTAL=0, CARGO_BUILD_JOBS=4, CARGO_NET_OFFLINE=true, BEND_GPU=off.

```sh
cargo test --locked --offline --features extended_observability \
  --test cpu_array_borrow --test cpu_array_runtime --test compiler_executable_c_stable_arrays
cargo clippy --locked --offline --all-features --test cpu_array_borrow -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
python3 scripts/array-runtime/compare.py --repo "$REPO" --reference "$REFERENCE" \
  --binary "$PORT_BINARY" --manifest "$REPO/scripts/array-runtime/borrow-cases.json" \
  --output "$NEW_EVIDENCE"
```

The final comparison uses the gate's non-Tracy test-profile binary. C oracles use
the pinned CPU O3 profile, port C retains strict warnings, and each executable
runs with GPU disabled where applicable. Differential children are bounded at
60 seconds; integration programs at 20 seconds and their C compiler at 45 seconds.
The outer runner monitors unchanged 11 GiB prebuild / 10 GiB active workspace
and 1 GiB temporary floors. Source bundles and raw evidence have verified local
backups; compact streams/commands/hashes are in Git. Original machine-root
strings are replaced only in public copies; raw stream hashes remain.
No desktop, GPU, installation, dependency change, cleanup or release was needed.
Blocked Library delivery was not retried. Reviewed publication uses the standing
non-force feature/main workflow.

[Independent review](cpu-array-borrow-review.json), [exact validation](cpu-array-borrow-validation.json), [compact streams](cpu-array-borrow-results.json.gz), [coverage](cpu-array-borrow-coverage.json), and [evidence rows](cpu-array-borrow-evidence.csv).
