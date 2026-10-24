# Default Nat function qualification and false-proof diagnostics

Source `0093d01bf3e1f18337cf94f2ba7ee2882828b0bf` adds the unchanged pinned
[`parse/chain_default_nat_f`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/chain_default_nat_f.bend)
and [`_g`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/chain_default_nat_g.bend)
to the existing normalization runner and actual-core regression loop. No engine,
dependency, toolchain, budget or demand-policy change was needed. No push occurred.
The originals remain byte-for-byte copies with [Apache attribution](../tests/fixtures/cpu-normalization/README.md).

## Function results and actual applicability

| Original | Exact normal form |
| --- | --- |
| `_f` | `a => b => c => Nat.add(a, Nat.mul(b, c))` |
| `_g` | `a => b => Cmp.is_lt(Nat.cmp(a, b))` |

Pinned interpretation, port raw `run`, strict JSON `eval` and the actual normalized
core agree, twice. The arithmetic expression preserves multiplication inside
addition; comparison retains the pinned Bool-producing helper. These are returned
functions, not invocations with concrete arguments. Their normal forms do not
independently qualify every Nat operation or runtime overflow policy.

Both actual upstream C and JS generators refuse each function type, twice:
eight exits of 1, empty stdout, diagnostic identifying the unprintable function
type, and no output artifact. These refusals occur before C compilation or JS
execution, so no O3 runtime result is invented. The ordinary strict proof/core
mode applies to both fixtures; executable-only F32 observations remain distinct.

## Wrong-reverse and lost-duplicate diagnostic cause

The preceding no-main [list fixture](cpu-list-reverse-sort.md) and its two
authored false-proof controls were renewed at this source. The original still
has exact checking-output agreement and five matching named normal forms.
The controls remain invalid proofs, with exit 1, empty stdout and stable stderr
in upstream CLI, port strict `check` and compatibility `run`, twice.

| Control | Rejection phase / proof | Upstream displayed endpoints | Port diagnostic retains |
| --- | --- | --- | --- |
| Wrong reverse | Reflexivity checking, `rev`, Bend line 63 | `[3, 2, 1]` versus `[3, 1, 2]` | `List.reverse(&2, U32, l123)` versus `l321` |
| Lost duplicate | Reflexivity checking, `srt`, Bend line 75 | `[1, 2, 3, 3]` versus `[1, 2, 3]` | `sort([3, 1, 2, 3])` versus `[1, 2, 3]` |

A separate four-observation oracle probe confirms parsing/loading completes,
then `book_valid` throws a structured `Err` at the stated proof and source line.
Its JSON adapter exits 0 after observing an expected rejection; that status does
not mean the Bend program checks. No program execution begins. The Rust regression
parses each mutated source before calling strict `check_book`, and now requires
the exact offending definition plus the reflexivity failure prefix. Rejection
checks were strengthened, not relaxed.

The cause is visible in both implementations:

- Pinned [`check-rfl`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3571)
  compares the equality endpoints and retains them, the definition and source span
  in `Err`. Its [`expr_show` and `err_show`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1437)
  strongly normalize endpoint terms for display and render the Bend source excerpt.
- The port's [reflexivity check](../src/kernel/check.rs) also compares the endpoints
  (line 1062), then formats the original term references into `KernelError` (1063).
  [Conversion](../src/kernel/reduce.rs) reduces terms to decide equality; displaying
  the subsequent error does not reuse a normalized-endpoint result. `KernelError`
  retains a message and private classification, without this structured span and
  endpoint payload. Strict and executable checking add the failing definition name.
- The [check](../src/cli/check/check_cli.rs) and [run](../src/cli/run/run_cli.rs)
  commands wrap that error through `eyre`. The resulting terminal report includes
  host call-site/context presentation rather than the pinned Bend-source report.

Thus these controls agree on rejected proof, phase and offending definition;
they do not agree on diagnostic payload or rendering. This is not an arithmetic,
evaluation-order, resource-exhaustion or missing-host failure. Removing terminal
decoration alone would not close the difference. Exact text remains an open
checker-diagnostic gap in the [control ledger](cpu-checking-control-evidence.csv),
without a new exception. No broad diagnostic/API migration was attempted. A later
repair would need to address retained checker context and bounded error rendering,
while preserving Teamy host diagnostics and the [existing API caveat](cpu-nat-failure-contract.md#review-of-rust-api-consumers).

## Validation

| Check | Final result |
| --- | --- |
| Two default-Nat originals | Eight exact interpreter/native outputs, four strict eval reports, four actual-core comparisons, eight verified printer refusals |
| Full normalization | 34 fixtures: 266 successes, 76 printer refusals, six F32 proof exclusions; exit 0 |
| Renewed no-main list checking | 48 commands: 36 successful checking/definition observations, twelve expected false-proof rejections; raw negative diagnostics still differ |
| Structured oracle phase probe | Four observed checking rejections; four successful adapter exits, zero accepted proofs |
| Focused checks | **58 passed, 0 failed, 0 ignored** |
| Standard repository gate | **825 passed, 0 failed, 49 ignored**; exit 0 |
| Strict all-target Clippy / build | Both exit 0 |

All repeated applicable stream/status observations are stable; no comparison
timeout occurred. No earlier runtime/demand/read/comptime/audit or platform receipt
is relabelled as a new run. The 49 ignored tests remain 47 CUDA checks and two
optional profilers. GPU and fresh Windows/macOS checks were not run.

After activating the frozen environment, with `CARGO_NET_OFFLINE=true` and
`BEND_GPU=off`, relevant exact commands are:

```sh
cargo test --features extended_observability --test kernel_normalization --test kernel_core --test kernel_collections --test cli_proofs --quiet
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-features --bin teamy-bend --example compare_normalization
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <clean-pin> -OutputDirectory <fresh-evidence> -Suite all -Repetitions 2 -RequireExactText
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

The runner also offers `-Suite default-nat` for just these originals; the full
`all` suite is the executed receipt here. The list-checking adapter/CLI commands
are documented in the prior checkpoint. The additional phase probe loads each
authored mutation with pinned `book_load`, calls `book_valid` in a separate
`checking` phase, then records the thrown `Err` definition/span and `expr_show`
endpoints. All exact commands and raw bytes are preserved privately, with hashes
below. Its four children and the checking comparison have 30-second bounds;
the normalization runner retains its 60-second bound.

## Headroom, receipts and remaining coverage

Initial workspace headroom was 5.32 GiB; bounded cleanup
restored 14.45 GiB. The gate began with 13.34 GiB and kept
**4.82 GiB** workspace / **4.84 GiB** temporary-space headroom
over 156.03 seconds. No headroom/timeout stop triggered. Verified
deduplication of two immutable CLI/probe copies saved 0.24 GiB, retaining
all paths and hashes. Final workspace headroom was **5.07 GiB**; source and
evidence are preserved, with no mutable build output hard-linked.

The same bounded cache helper requires 12 GiB before broad builds and removes
only inactive rebuildable root-test binaries, preserving protected CLI/probes,
source, libraries and evidence. The gate monitors 2 GiB workspace / 1 GiB temporary
space floors and a 1,200-second deadline. Retained binary variants and Git metadata
relinking remain the [documented growth cause](cpu-nat-literal-roundtrip.md#disk-cause-and-prevention).
These are build-process safeguards, not changes to Bend resource policy.

| Receipt / input | SHA256 |
| --- | --- |
| normalization | `46eecfde243c4d778437ca9cd6d4e16a892cc0dcd6fa93df7dfc7de101c598f5` |
| checking | `35b6660e23729847c2e63020ed99e19a585c1dd8fbaf3e33c2d72b2cc8b92362` |
| diagnostic-phase | `57b8f4f15e020af21fd6b7d91663dfec909e36ee2917d656dbb92ebe8f412a3b` |
| phase-adapter | `a1df21c8e44bd38e25ed3e83fe3111306564f3107b8e4013625bc16f257b0455` |
| gate-monitor | `7a852ef4e38726c0ae8bb4ed36a9c2df92ace069bf14fec90bb88dfe56690f86` |
| cache-before-tests | `311bcc7cce81bffbfd51a03aa6ed0418bbc1de2c791146f6f532e82785d43eff` |
| cache-before-build | `116aa274b5a64d93ad7f1882f5813805c7abbd60d2d25c5149b6783b5551a11e` |
| cache-before-gate | `3c61f96cdd32c7ab5873b715210887c674abdb5cad49f123d7ff23e34ed99ddf` |
| snapshot-dedup | `8580e61ba7e867fddf5166315889224450329c39c2cd15c6bcf3e88fddfa0cff` |
| Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |
| Port binary | `9ca3cb247acde0f69cc4df4f35866dbd9207717a4c57d1e77d8b31bcecda1e5b` |
| Structural probe | `9bfabbafa86c2f4c85424f5d2dd69b1cc7aa4e9210ca9a2adbe7e2698b6ec22e` |

The interpretation ledger now links thirty originals at this source. Runtime
plus interpretation link 105 originals; 1,197 lack those modes: 439 checker
negatives, twelve checking-only modules, 153 interpreter-only cases and 593
compiled candidates. This is not a count of missing runtime tests. List checking
is separately renewed; the other four checking rows and runtime/demand ledgers
retain their earlier source identities. The two authored diagnostic controls add
no inventory rows and retain `diagnostic_mismatch`.

The acceptance criterion remains unsatisfied. Existing F32.read, failure-order,
resource-policy, foreign, unavailable host, API and platform cells remain open.
Next independent family: `parse/chain_spacing_rules_f`, `_g` and `_h`, covering
spaced modulo/greater-than, nested generic closers and a Nat literal's open tail.
They are unlinked candidates whose actual printer applicability must be checked;
their sources were inspected here, not executed or qualified.
