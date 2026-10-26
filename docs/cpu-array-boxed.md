# Boxed-array cloning and owner disposal

Tested source: `0c401575cb1c485c4dd8beabfa5b2efd4d9bcb26`. This checkpoint adds tests, fixtures and an array
comparison manifest. Production code, dependencies, toolchains, upstream
references and resource limits are unchanged. The publication checkpoint adds
reviewed documentation after the clean-source validation.

The unchanged pinned originals execute successfully across native Rust, upstream
C/JS and port C/JS, twice per lane with status 0 and empty stderr:

| Original | Exact stdout |
| --- | --- |
| `reg/array_clone_boxed` | `ababababababababhelloworldabcdabzzdcleaf\n` |
| `reg/array_heap_element` | `716\n` |

These are new executions of the originals. No production defect was demonstrated.
Strict proof evaluation of IO remains excluded separately.

## Ownership and layout coverage

The first original exercises intrinsic clone with shared boxed String leaves,
read copies, an unread copy and a swap. The second uses its own recursive
`Arr.clone`; it does not alone qualify intrinsic cloning of the constructor type.
The authored controls add that coverage:

- **Boxed drop:** depth-zero and depth-two arrays of computed Strings, one unread
  clone, independent mutations of both surviving branches, retained read and
  displaced values, and eight ordered lines showing full array contents.
- **Tag transitions:** intrinsic cloning of `OwnedText{String}` / `Vacant{}`
  elements, a swap that returns the previous owner, a set that discards the new
  owning payload, a read and clone after clearing to the nullary arm, then an
  independent refill. Twelve lines show full tags, strings, original arrays,
  cleared snapshots and changed arrays. Both owning and nullary initial arms
  are exercised at leaf/last depth-two positions.

Pinned [Array.clone](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L2181)
requires `Data`; swap accepts `Type`. Pinned
[Array.set](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L2195)
drops the displaced result, while swap returns it.
[blk_copy](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L4056)
retains boxed cells. The port's existing ownership-mask machinery and
`tb_c_blk_write` preserve discard-versus-transfer behavior. Existing stable-array
regressions already cover raw/list mask transitions in port C; this batch adds
owning String/nullary observations across all five executable lanes.

All four new positive programs run twice through the same native checked book.
Separate generated-C variants check zero live words, blocks, tasks,
continuations, frames and depth after successful normal completion. These are
selected accounting checks, not identical physical layouts, exact destruction
timing, general memory-safety proofs, native/JS heap inspection or new
cancellation coverage. Existing layout guards remain unchanged, including the
port C check on depth plus log2 cell stride, capped at 17. A wide cell cannot be
assumed to have the same logical-depth boundary as a narrow one.

## Results and preserved refusals

Final post-gate matrix: **132 observations / 32 repeat pairs / zero timeouts**.
Five positives give **50 exact successful executions** across five lanes. Two
checker negatives give 18 exact rejection observations with no requested output
artifacts. The depth witness gives ten runtime refusals, including two unwaived
port-C diagnostic differences. All original streams and exit statuses are retained.
The baseline has 120 identical stream observations and 12 identical port
generated artifacts; its binary belongs to historical source `a347a2d8f679d10ee21fdca3539deb398ec5e633`,
not the newly tested source. Compile-result paths necessarily name different
evidence folders.

Both new negatives load successfully then reject during checking: using one
clone result twice without a reusable binder, and cloning a non-Data element.
Their exact stderr/status and requested-artifact absence match upstream.
The existing affine closure-swap control remains a positive Type-element case.

The unchanged depth-32 fixture still refuses. Native, upstream C/JS and port JS
print `bend: an array past the deepest block class 31`; port C prints
`teamy-bend executable C: array budget exhausted`. Two port-C repeat observations
represent one unwaived case/backend discrepancy. No resource exception, relaxed
limit or equivalent-diagnostic waiver was introduced.

Standard full gate: **1363 passed / 0 failed / 49 ignored**,
exit 0. Formatting, standard strict Clippy, build and real TCP/UDP provider checks
passed. Focused checks passed 12 tests; targeted strict Clippy passed.
The full gate renews all three new Rust tests at clean source. Hardware/profiler
ignores remain unrun; old supplementary all-target lint failures were not rerun
or cleared. Gate minimum headroom: 12.090 GiB workspace /
1.364 GiB temporary. Final matrix uses the gate binary
identified in the validation receipt.

The first authored sketches destructured computed tuples directly. Pinned
upstream rejected all three during loading because the scrutinee must be a
parameter/field. Their sources and exact failures are preserved. Corrected
helpers destructure parameters and preserve the intended ownership sequence;
the corrected reuse-clone negative now reaches the intended checking phase.
The non-Data negative already reached checking. Those initial invalid
controls are not production acceptance defects or qualified affine negatives.

## Current and historical scope

The two originals and new controls are newly qualified here. The affine swap
and depth refusal are renewals. No whole checking census or rejection subset
was rerun. Historical census source remains
`c5edea8a75022c1654fbf568629422cf73bee1d0`: 366 both accepted, 405 exact rejections,
494 acceptance differences, 31 different rejections and six upstream stack
failures. Outside its fixed 385-row IO cohort, 109 acceptance and 31 rejection
differences remained. The 31-row subset was last measured at `1a135e4`, still
different. Historical executable-check acceptances do not establish execution.

Receipt linkage is now 524/1,302 (previously 522); 778 entries remain
unlinked: 256 checker_rejection, 509 interpretation_and_compiled_candidates, 1 checking_without_main, 12 interpretation_only.
Links include failures and do not constitute universal parity. No checking-census
disagreement was closed by this coverage-only checkpoint.

The argument-recovery barrier remains blocked by nontrivial declaration lowering.
No safe declaration-reuse design was adopted. nat_first stays pending/unwaived;
compact Nat/string lowering is unadopted. Base coordinates, F32.read, source
holes, external invoke consumers, foreign/host/oracle limitations and old
supplementary all-target Clippy failures remain open. Full CPU parity is incomplete.

Next substantive originals: `reg/array_nested` and `run/array_split_join`, covering
nested affine array transfer and half-array reconstruction/indexing. Only their
source was inspected in this batch; no new execution is claimed.

## Reproduction, storage and preservation

The frozen environment remains Rust/Cargo 1.96.0, formatter nightly-2026-05-24,
Node 24.19.0, Clang 19.1.7 and PowerShell 7.5.4. Upstream is
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, mounted docs is
`50e89d6897ce2a65167ef65b17c5fa83fe849158`, and Cargo.lock SHA-256 is
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
Host overrides: CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0,
CARGO_INCREMENTAL=0, CARGO_BUILD_JOBS=4, CARGO_NET_OFFLINE=true, BEND_GPU=off.

```sh
cargo test --locked --offline --features extended_observability \
  --test cpu_array_boxed --test cpu_array_borrow --test compiler_executable_c_stable_arrays
cargo clippy --locked --offline --all-features --test cpu_array_boxed -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
python3 scripts/array-runtime/compare.py --repo "$REPO" --reference "$REFERENCE" \
  --binary "$PORT_BINARY" --manifest "$REPO/scripts/array-runtime/boxed-cases.json" \
  --output "$NEW_EVIDENCE"
```

Final comparison uses the gate's non-Tracy test-profile binary. Upstream C uses
its pinned CPU O3 profile; port C keeps strict warnings. Differential children
are bounded at 60 seconds; integration programs at 20 seconds and their C
compiler at 45 seconds. Outer commands retain the 1200-second bound and unchanged
11 GiB prebuild / 10 GiB active workspace / 1 GiB temporary floors.

Before running, temporary free space was 1.29 GiB. Under the approved cleanup
policy, 156 inactive generated ELF copies from two completed array batches were
verified byte-for-byte, with permissions, against their existing archives before
removal. About 132.5 MiB was reclaimed, bringing temporary free space to 1.42 GiB;
workspace capacity was not reclaimed by this operation. Sources, raw streams,
receipts, archives and unpushed progress were retained. Removed binary paths
require restoration from the recorded archive/member with the recorded mode;
no mutable hardlink aliases were introduced. The cleanup manifest is backed up.

Current source bundles, raw evidence, failed sketches and final binary have
verified local preservation. Compact commands/streams/hashes are committed;
public copies replace only explicit machine roots and retain raw stream hashes.
No desktop, GPU, installation, dependency change, release or blocked Library
retry occurred. Publication uses the standing reviewed, non-force feature/main
workflow.

[Independent review](cpu-array-boxed-review.json), [exact validation](cpu-array-boxed-validation.json), [compact streams](cpu-array-boxed-results.json.gz), [coverage](cpu-array-boxed-coverage.json), and [evidence rows](cpu-array-boxed-evidence.csv).
