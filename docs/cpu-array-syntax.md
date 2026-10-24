# Concrete arrays: printer correction and open normalization gap

Source `9310cf2cd5d4aa3f9748a308ceb51c5c46421a0e` corrects array term printing
and adds two unchanged pinned originals to `compare-cpu.ps1 -Suite parse-arrays`.
Qualification is partial: `array_literal` matches, while the pure native
normalizer still refuses `array_write_stmt` at its unchanged nesting guard.
No evaluator, checker, dependency, toolchain, demand contract or budget changed.
No push occurred.

## Exact results and runtime distinctions

| Original | Required stdout, followed by LF |
| --- | --- |
| `parse/array_literal` | `([3, 3, 3, 3], [True{}, True{}])` |
| `parse/array_write_stmt` | `([7, 1, 0, 0, 0, 42, 0, 0], 42)` |

Actual pinned C/JS generation was checked before implementation: all four
commands succeeded and produced artifacts. Final comparison uses the original
files without wrappers, C at the upstream CLI's O3 profile, and explicit
`--threads 1 --gpu off`. Port C retains strict warning flags; Node runs both
generated JavaScript programs without a host adapter or Bun dependency.

| Lane, two repetitions per original | Literal | Statement writes |
| --- | --- | --- |
| Upstream C, port C, upstream JS, port JS | Exact | Exact |
| Direct pinned interpreter | Exact | Exact |
| Native pure CLI | Exact | Required resource mismatch; exit 1, empty stdout |
| Strict JSON `eval` | Exact entry/value report | Same normalization-depth refusal |

All successful program executions exit 0 with empty stderr and exact stdout.
All repeated streams/statuses are stable; no timeout occurs. This is **22/24
execution observations** and **2/4 strict eval reports** matching. The public
comparison runner exits 1 at the first native write failure; its remaining
observations were completed by a separate bounded capture using the same
immutable binary and generated artifacts. That capture also exits 1. Neither
failure is converted into a passing safety exception.

The separate checked `evaluate_data` API produces both complete expected values,
with explicit ALeaf/ANode core comparisons. It uses the constructor runtime;
the pure CLI and strict eval use the kernel normalizer. This API result does not
qualify or replace the failing CLI lane. No dispatch behavior changed.

## Printer repair and residual diagnosis

Previously the literal's native output exposed `ANode`/`ALeaf` constructors.
The [pinned term printer](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1259)
flattens a complete array tree into ordered bracket notation. The port now walks
that existing tree iteratively, validates it before writing, and displays leaves
without evaluating them. Partial, annotated, malformed and qualified constructor
boundaries retain ordinary spelling. Fifteen direct pinned-printer controls
agree, including an existing deep core tree. The input-depth guard stays intact.
Arrays and lists share a displayed notation, so tests compare actual array core
against explicit constructors, not against a List parsed from the pretty string.

The unchanged write original passes checking but fails during normalization.
A temporary private trace found 124 active weak-head reduction calls plus
normalization/freshening frames at the guard. A candidate avoiding a redundant
selected-arm reduction still failed the original and was reverted. Trace code
was removed; `check.rs` and `reduce.rs` are unchanged from the prior checkpoint.
The public fix is confined to the printer and its regression/comparison coverage.

Six authored reductions show that a write is not needed to reproduce the gap:

```bend
import Base

def main() -> Array<U32> & U32:
  [0 : U32*4n][3]
```

Upstream returns `([0, 0, 0, 0], 0)`; the native pure normalizer reaches its
128-level guard. One- and two-slot reads agree; four- and eight-slot reads,
one eight-slot write, and an eight-slot write/read pair refuse. Four isolated
word-expression controls, including the combined index-mask/comparison, agree.
This narrows the remaining investigation to reduction composition, rather than
establishing a wrong mutation result or a standalone word-operation discrepancy.
The full original remains an executable failing regression in the CPU runner.
No smaller control substitutes for its required success.

## Validation and provenance

| Check | Result |
| --- | --- |
| Original arrays, full capture | 22/24 exact execution observations; 2/4 strict eval reports; native write normalization remains failed; capture exit 1 |
| Five-case baseline | 50/50 exact executions; runner exit 0 |
| Full normalization | 42 fixtures, 330 successes, 108 printer refusals, six retained F32 proof exclusions; exit 0 |
| Focused tests | **67 passed, 0 failed, 0 ignored** |
| Standard repository gate | **828 passed, 0 failed, 49 ignored**; exit 0 |
| Pinned formatting, strict all-target Clippy and build | Passed |
| Direct pinned printer controls | 15/15 exact |
| Depth minimizers | Six upstream successes; two native successes and four retained native refusals |
| Isolated word controls | Four upstream/native pairs agree |

The 49 ignores remain 47 CUDA checks and two optional profilers. The initial
printer mismatch, original depth failures, failed reduction candidate, complete
raw captures and minimizers are retained privately. Existing diagnostic, F32.read,
competing-error-order and resource differences remain unwaived. Full positive,
negative/demand, checking, read and comptime differential matrices were not
renewed wholesale; neither were the broad checker audit, GPU or fresh platform
checks. Their rows keep actual historical source identities.

Commands with frozen tools, `CARGO_NET_OFFLINE=true` and `BEND_GPU=off`:

```sh
node main.ts <original-fixture> -o <fresh-output.c-or-js>
cargo +nightly-2026-05-24 fmt --all -- --check
cargo test --features extended_observability --test cpu_array_syntax --test kernel_normalization --test kernel_arrays --test parser --test parser_numeric --test cli_proofs --quiet
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-features --bin teamy-bend --example compare_normalization
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <fresh-arrays> -Suite parse-arrays -Repetitions 2
python3 <private-evidence>/complete-matrix.py
python3 <private-evidence>/minimize-depth.py
python3 <private-evidence>/minimize-word-depth.py
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <fresh-baseline> -Suite baseline -Repetitions 2
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <clean-pin> -OutputDirectory <fresh-normalization> -Suite all -Repetitions 2 -RequireExactText
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

The first command runs from pinned `bend2`; the array runner and complete capture
return 1 for the retained gap. The others complete successfully. Child comparison
deadlines remain 60 seconds. Sources, immutable binaries, artifacts, commands,
streams, statuses and receipt hashes remain available. Both reference checkouts
are clean at the existing pins; Cargo.lock is unchanged. Original fixture bytes
and [Apache attribution](../tests/fixtures/cpu-array-syntax/README.md) are retained.

## Headroom and remaining acceptance

Initial workspace headroom was **4.33 GiB**. Bounded cleanup of
96 inactive rebuildable root-test products restored **13.46 GiB**.
The gate began with **12.11 GiB**, kept at least **3.48 GiB** workspace
and **4.76 GiB** temporary-space headroom, and finished in 160.74 seconds.
No headroom or timeout stop triggered; final headroom was **3.48 GiB**.

Cache preflight still requires 12 GiB with builds quiescent and only removes
hashed, inactive rebuildable test executables. Source, evidence, libraries and
CLI/probes are protected. The gate monitors 2 GiB workspace / 1 GiB temporary
floors and a 1,200-second deadline for its own process group. Retained binaries
and metadata-triggered relinking remain the storage-growth cause. No historical
evidence was deleted and no Bend resource limit was raised.

| Receipt / input | SHA256 |
| --- | --- |
| arrays | `427425f6a8557b99fdc7530c26979415b8269d6fc3c0b19d85cb595560e701c0` |
| complete-matrix | `0aa5de3f6f50125d63f7e6669394a2f2c83c08f395bf894303d26cf18a698e5a` |
| normalization | `b2a32b3e2fd8ff64910abdd8f629571b25084108e223e31c11af1895c5a373a1` |
| baseline | `4e6d66a71836d838393dc15454f4499f9e8917fa7b4855c1614d70beb42b0716` |
| printer-oracle | `894178ae6c823f0c59806c298b6aaa6d818e58ac35031b42e4715f2255dbe773` |
| depth-controls | `1701cdfc8b30387af7ff0cd983397e4cda77a8b6bbddbbec7394b4ea6ce90060` |
| word-depth-controls | `039ac8af0681bd92e0697130cecb3d769f3e77c8d2217ea6cd6844928af457df` |
| gate-monitor | `1fe4de753cfc15f2a2d580509c59ba19929605ec2ce5d792a469e3b459e13b97` |
| cache-before-build | `fee1d70453d789eb39626233c02d69d595038885f6156ae2b0a7b2a8a4da372d` |
| cache-before-gate | `3b204142e734b94de2629d0a394f0218f95ddf287af40f7759758009fbcfb51a` |
| initial-applicability | `4c948e98dc5b3fd724980a4aaa9029d9c4f300899461c4861722263e1be37794` |
| initial-values | `6b88a965bc3fa63d0a1d507eb488d6e4e9fdcf41c27505ddf98d32061a504ff1` |
| rejected-candidate | `ad7a0de0ebbab63ec5980f89a34cc93344a95090c85b845f206717dfbb0de11b` |
| candidate-patch | `ea9678938038af4c1772d174ea3d47079ec99ff1fb2fbb354aef89dc1c99ff86` |
| Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |
| Port binary | `13835b7ce35a0e2beedb5e0bfe4be37f5c9bfbde58652a14015680569813cee2` |
| Normalization probe | `a45381a113e406669423311e05d60642810c7e133bcd36d438324b71f562e2b2` |

The runtime ledger now has 77 originals; its new write row explicitly fails the
native cell. Thirty-eight interpretation rows and three baseline runtime originals
are renewed at this source. The remaining 72 runtime originals retain historical
receipts. Their union links 115 originals; 1,187 lack those two modes, comprising
439 checker negatives, twelve checking-only modules, 153 interpreter-only cases
and 583 compiled candidates. These are applicability categories, not 1,187
missing runtime tests. Separate no-main checking evidence remains intact.

Next priority is the reproduced four-slot read and original write normalization
gap: reduce avoidable pending reduction work without raising the 128-level guard,
weakening work accounting, changing demand or swapping the CLI's evaluator.
Independent array read-annotation/read-write originals remain unqualified too.
Full CPU parity is incomplete; there is no dependency or executor blocker.
