# Annotated and chained array reads

Source `0f83aeaaec7ecbf51aa5e7ee1750fff36441e7ff` qualifies the unchanged pinned
parse/array_read_annotation and parse/array_read_write originals across the
available compiled/native execution lanes. It also repairs a composed-word
normalization defect exposed by a separate pure-entry derivative. CPU parity
remains incomplete; the upstream direct-IO host and strict-proof boundaries
are recorded separately.

## Observations and applicability

| Source | Exact stdout including LF | Available execution result | Strict eval |
| --- | --- | --- | --- |
| Original array_read_annotation | `4\n` | Five lanes agree twice | IO book excluded |
| Original array_read_write | `97\n` | Five lanes agree twice | IO book excluded |
| Annotation pure-entry derivative | `4\n` | Six lanes agree twice | main and main.out agree twice |
| Read/write pure-entry derivative | `97\n` | Six lanes agree twice | main and main.out agree twice |

Actual upstream C/JS printers accept all four programs. C uses the pinned CLI
O3 profile with `-lpthread -lm` and `--threads 1 --gpu off`; port C retains its
strict warning flags. The five lanes are native, port/upstream C and port/upstream
JavaScript. Pure programs additionally run through the pinned direct interpreter.
All completed successful executions have empty stderr and exit 0; repetitions
are deterministic and no timeout occurs.

The originals' direct upstream IO entry fails under the installed Node with
exit 1, empty stdout and exactly `TypeError: require is not a function\n`.
These four repeated observations are `blocked_import_meta_require`, not a pass
or the separate Bun-FFI blocker. Strict eval of either original, selecting main
or main.out, refuses the whole IO book with `main: undefined name IO`, empty
stdout and exit 1. Those eight observations are deliberate proof-mode exclusions,
not diagnostic parity or runtime defects. Selecting a helper does not bypass
whole-book checking.

The two Apache-attributed derivatives replace only the IO main with a pure U32
main calling the original helper. Every original helper and the original files
are retained. They add no upstream inventory rows. The complete capture has
**44 exact executions, eight exact strict reports, four host-blocked executions
and eight proof exclusions**. The public positive runner exits 0; the supplemental
complete capture deliberately exits 1 because its required upstream direct-IO
lane is blocked. The public receipts keep matched=false for both originals
and completed_runtimes_match=true for their available execution lanes. No host
adapter was introduced.

## Demonstrated defect and bounded correction

At `eefac845ba8d3fba91206540589b9424dc24bcf2`, the original IO programs already
matched across their available lanes, but the read/write pure derivative exhausted the unchanged 2,000,000-step
kernel budget. The array-free expression `(9 * 10 + 7 : U32)` reproduced it.
Each read, their sum, multiplication alone, addition alone and the updated array
pair passed. The error was excessive work in composed word arithmetic, not an
incorrect array write or index.

The port's Word.adc expanded sum/carry Boolean formulas, repeating reductions
when arithmetic was composed. The
[pinned Word.adc and pair-consuming helper](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L1143)
consume Bool.full_add's pair before constructing WCon. The correction restores
that shape and its a, b, f match order. An explicitly typed core match inlines
the helper so recursive calls remain directly visible to the port's strict
checker; the existing named helper remains available. This is a checked Base
library change, with no Rust reducer/runtime changes or new cache.

The exact failing pure derivative and array-free expression now return 97.
Additional checks cover composed subtraction (83), a carry chain (65536), U32
wrapping (1), zero-width Word.adc and an unobserved symbolic word tail. The last
two normalize to `f => c => WNil{}` and `t => False{}`, respectively, matching
the pinned interpreter. Thirteen bounded controls produce **52/52 exact native/
interpreter observations**, twice per lane. Function controls establish
normalization, not compiled-function printer applicability. The existing
exhaustive Boolean full-adder equality proof remains checked.

No 128-level guard, 2,000,000-step budget, input bound, evaluator selection,
approved executable demand policy, cancellation or stream contract changed.
The renewed guard and executable tests retain bounded refusal, recovery, weak proof demand,
first-error and cancellation coverage. Existing executable demand matrices
retain their previous classifications; they do not establish universal
first-error equivalence.

## Validation and provenance

| Check at 0f83aea | Result |
| --- | --- |
| Standard gate | **834 passed, 0 failed, 49 ignored**; exit 0 |
| Focused integration tests | **31 passed**; exit 0 |
| Internal guard tests | **5 passed**; exit 0 |
| Pinned formatting, strict all-target Clippy, all-feature build | Passed |
| Array-read runner | Four programs, repeated exact available lanes; exit 0 |
| Complete array-read capture | 52 exact, four host-blocked, eight proof exclusions; exit 1 |
| Bounded arithmetic/read controls | 52/52 exact; exit 0 |
| Baseline | 50/50 exact execution observations |
| Normalization | 42 fixtures; 330 successful commands, 108 printer refusals, six retained F32 proof exclusions |
| Strict checking audit | 1,302 processed; no acceptance-status changes; all 439 checker negatives refused |
| Six runtime-negative fixtures | Existing classifications unchanged; exit 1 |
| Ten executable demand controls | Existing classifications unchanged; exit 1 |

The audit still accepts 366 expected positives and refuses 487, plus all 449
nonzero-expectation fixtures. A matching status does not qualify raw diagnostics.
Negative cells retain four resource refusals, one native foreign gap and one
unavailable Bun lane; two C cells remain inapplicable. The approved C wildcard
exceptions and unresolved competing-error order remain distinct. F32.read,
checker diagnostics, host/API/platform and broader coverage requirements remain
open. The 49 ignored checks are 47 CUDA checks and two optional profilers.

All final matrices and the full gate identify 0f83aea. The initial 832-pass gate,
six pure-write resource mismatches and failed intermediate parser/type-checking
experiments remain historical evidence at their actual inputs. The candidate
focused, formatting and strict all-target Clippy checks preceded the commit
but used the identical committed source.
No full positive-runtime, read/comptime or raw checker-diagnostic differential
renewal, GPU, or fresh Windows/macOS validation is claimed here.

Eleven runtime rows are current (two added originals, three baseline and six
negative originals); 68 other runtime rows keep historical source identities.
All 38 interpretation and ten demand rows are renewed, with historical O2
columns retained. Runtime/interpretation union is **117 originals**; the remaining
1,185 comprise 439 checker negatives, twelve check-only modules, 153 interpreter
cases and 581 compiled candidates. These are applicability categories, not a
count of runtime failures. Complete lane/API review remains required.

## Commands and disk safeguards

Use the activated frozen environment with CARGO_NET_OFFLINE=true and BEND_GPU=off.
Choose fresh evidence directories for every comparison.

```sh
cargo +nightly-2026-05-24 fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --features extended_observability --test cpu_array_syntax --test parser_numeric --test executable_check --test kernel_arrays --test cli_proofs --quiet
cargo test --features extended_observability --lib depth_tests --quiet
cargo build --all-features --bin teamy-bend --example compare_normalization
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <arrays> -Suite parse-array-reads -Repetitions 2
python3 <private-evidence>/complete-matrix.py
python3 <private-evidence>/controls-final.py
python3 <private-evidence>/audit.py
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <baseline> -Suite baseline -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <clean-pin> -OutputDirectory <negatives> -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <clean-pin> -OutputDirectory <demand> -Suite demand -Repetitions 2
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <clean-pin> -OutputDirectory <normalization> -Suite all -Repetitions 2 -RequireExactText
```

Individual comparisons use 60-second bounds; checking-audit concurrency is four.
Exact commands, source/binary hashes, raw streams/statuses, timing, repetitions
and cleanup observations are preserved privately. Public documents omit raw
machine-specific diagnostics.

Preflight removes only inactive rebuildable test products. A later preflight was
127 MiB short of the 12-GiB reserve; it refused to start a build. Five verified
byte-identical immutable snapshots were then hardlinked together, preserving all
paths, bytes, modes, hashes and receipts and recovering 561,459,200 bytes. Deduplication
did not link a mutable target binary or remove source/evidence. The full gate ran
before new comparison snapshots, with 2-GiB workspace and 1-GiB temporary-space
stop floors. It took 172.10 seconds; minimum headroom was
3.64 GiB workspace and 4.78 GiB temporary space. No exhaustion occurred.

| Receipt/input | SHA256 |
| --- | --- |
| arrays | `de81928a9ea7b5000794f616c5e59a257b5bcd27287221cf173cedd1a1076fce` |
| complete-matrix | `d61468ddeadcdc95e8b10fbf97837f3920794eb521950ed1c9f4cbddf8241aff` |
| controls | `44659994ae4beee6f85866bc8cbec8ed2a1c44921abfa808010fea21484c43c0` |
| audit | `d2e7253aa9ad175e907a5fec4794b9aa1f9cc49e5aa4f29023a90435711ac13a` |
| negatives | `fba32313ff0ff1afe4ce2bbcfdb170f5c56b0c35a3a93006c7f3a1d8110cfa04` |
| demand | `b8742dc2d1aca997389b5002c73428f2795f5b39a2a38974ba92f88e23817d6a` |
| baseline | `48c7ba7d218b668e42dd9eea0da10a230997f67d094334306cc08cc65dc6efab` |
| normalization | `85030701952989fc119c0e4e206aa2d1c520ef248da7630e3fb9ece76fa9db41` |
| gate | `ad9a5b83d11dccd2f22b237c40437faeaea49b488b642744f67dd2c636e5a1e1` |
| cache | `47f62c6156d09a6d7a025b8d6b7efa51cb916c16f55f7aea35dd5ad5c2874d9a` |
| deduplication | `ab095b23053c65f64ef9e0a243528f3be7571aacabcca8351992d6eb85a52cee` |
| pre-fix-complete-matrix | `e283940830f491e011812fe9c6467ec507b05da56ec1b9167ce4adb499ab9b0a` |
| Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |
| Port binary | `6d4151c547e2867546ee31fa6307d4beab1ab10a1144e76827f74b954a285a1b` |

Next bounded family: parse/bitwise_dot_operators and parse/brace_term_forms,
selected from the applicability ledger. Their source was inspected only here;
actual printer qualification and corresponding execution/normalization remain
unrun. Remaining array parser negatives need checker-diagnostic qualification,
not invented runtime claims. Dependencies, toolchains, upstream pin and docs
reference remain frozen and clean. No installation or push occurred.
