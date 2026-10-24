# Typed comparison and array-index function qualification

Source `694a2108462140b4901ab948690fc7da625a3b63` adds the unchanged pinned
`parse/chain_paren_group_k` and `_m` originals to the existing strict comparison
suite. No engine, dependency, toolchain, comparison vocabulary, demand policy
or safety budget changed. No push occurred.

## Observations and applicability

| Original | Exact normal form |
| --- | --- |
| `_k` | `a => b => c => Cmp.is_lt(U32.cmp(U32.add(a, b), c))` |
| `_m` | `xs => i => j => Array.get.at(U32, U32.add(i, j), Array.size(U32, xs))` |

Actual upstream C/JS generation was checked first on both original files.
All four commands exited 1 with empty stdout and no output artifacts because
the selected result has a function type. The final matrix independently repeats
those checks twice: eight further verified printer refusals, with stable raw
diagnostics. No C compilation/O3 execution or generated JS run is claimed.

Pinned interpretation, port raw `run`, strict JSON `eval` and actual normalized
core match both originals twice. Raw values include identical whitespace and
final LF. The comparison preserves separate addition, comparison and Bool
classification in `_k`; `_m` retains both the U32 index expression and the array
size/result threading. No concrete arguments are supplied, so this qualifies
the returned functions rather than arbitrary numeric inputs or array reads.

The [pinned syntax rules](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L103)
make the parenthesized annotation choose U32 operators, and separately make
array-index arithmetic U32 without an annotation. The
[`Array.get` definition](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L2226)
expands into `Array.get.at(T, i, Array.size(T, a))`, explaining the surviving
structure in `_m`. The unchanged
[`_k`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/chain_paren_group_k.bend)
and [`_m`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/chain_paren_group_m.bend)
sources retain their goldens and [Apache attribution](../tests/fixtures/cpu-normalization/README.md).

All five parenthesized companions now have selected-main normalization evidence.
Their strict actual-core comparisons remain distinct from the existing F32
executable-output/core-regression mode and its strict-proof exclusions.

## Validation and receipts

| Check | Final result |
| --- | --- |
| Two comparison/index originals | Eight exact interpreter/native outputs, four strict eval reports, four actual-core comparisons and eight verified printer refusals |
| Full normalization | 42 fixtures: 330 successes, 108 printer refusals, six retained F32 proof exclusions; runner exit 0 |
| Focused normalization/parser/CLI tests | **59 passed, 0 failed, 0 ignored** |
| Standard repository gate | **825 passed, 0 failed, 49 ignored**; exit 0 |
| Pinned formatting, strict all-target Clippy, build | All exit 0 |

All repeated streams and statuses are stable; no timeout occurred. The 49 ignores
remain 47 CUDA checks and two optional profilers. The two cases extend an existing
test loop, so no additional named-test count is expected. No engine defect or
candidate validation failure was found. Existing wrong-reverse/lost-duplicate
rejection assertions pass, while their historical raw diagnostic differences
remain unwaived at `0093d01`. Those CLI/phase comparisons were not renewed.

Only interpretation receipts are renewed. Runtime, demand, checking, read,
comptime and broad-audit matrices retain their actual earlier source identities;
fresh Windows/macOS and GPU checks remain unrun. Upstream is still clean at
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, docs at `50e89d6`, and Cargo.lock is
unchanged. Raw captures, complete commands and immutable CLI/probe snapshots
are retained privately.

With the frozen tools activated, `CARGO_NET_OFFLINE=true` and `BEND_GPU=off`:

```sh
node main.ts <original-fixture> -o <fresh-output.c-or-js>
cargo +nightly-2026-05-24 fmt --all -- --check
cargo test --features extended_observability --test kernel_normalization --test parser --test parser_numeric --test cli_proofs --quiet
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-features --bin teamy-bend --example compare_normalization
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <clean-pin> -OutputDirectory <fresh-evidence> -Suite all -Repetitions 2 -RequireExactText
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

The first command runs in the pinned `bend2` directory for each original/target.
`-Suite paren-functions` now selects all five companions; the final receipt uses
`all`. Each comparison child has a 60-second deadline.

## Headroom and remaining gaps

Initial workspace headroom was **4.57 GiB**. Bounded cleanup
of 96 inactive rebuildable root-test products restored **13.71 GiB**.
The gate began with **12.84 GiB**, kept at least **4.33 GiB** workspace
and **4.84 GiB** temporary-space headroom, and finished in 157.40 seconds.
No headroom or timeout stop triggered. Final workspace headroom was
**4.33 GiB**. One immutable CLI/probe snapshot is retained for this
checkpoint; no extra snapshot deduplication was needed.

The existing preflight requires 12 GiB before broad builds and removes only
hashed, inactive rebuildable root-test products while builds are quiescent.
Source, evidence, libraries and CLI/probes are protected. Its process scan is
not a Cargo lock. The gate monitors 2 GiB workspace / 1 GiB temporary-space
floors and a 1,200-second deadline for its own process group. Retained binary
variants and metadata-triggered relinking remain the documented growth cause;
these build safeguards do not change Bend resource policies.

| Receipt / input | SHA256 |
| --- | --- |
| normalization | `3a9b175403df0bd71b9b86b655bbeb5767e415b61036c11080fc7caafacb4212` |
| gate-monitor | `b58349718daddb40a35b0d2621b5aa7827680c9f3889657215aa34e25cdd50b4` |
| cache-before-build | `18bc8c50523e88c3637d9c33acb1412a6357a37cd71d6d67899996b9fba580d0` |
| cache-before-gate | `aeb547e7c88f3aa58ee318795e1dc531979cdb2ae8dc1260394ad81781b32a87` |
| Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |
| initial-applicability | `f10d588a01e4ab168c182d5fa8d07cbafa95d5d1101c1b630f0a97e503c3d637` |
| Port binary | `acc9c09385433c6acae2dc95de818a2693d041660ba2236bb1f7725dfc655a06` |
| Structural probe | `d80c2246dd3e711cd94bae2dea9b57bcd1787123c7cbf720d8d7cff034b0b649` |

Runtime/interpretation ledgers link 113 originals. The 1,189 without those modes
comprise 439 checker negatives, twelve checking-only modules, 153 interpreter-only
cases and 585 compiled candidates. They are distinct acceptance requirements,
not 1,189 missing runtime tests. Explicit no-main checking evidence stays separate.
Full CPU parity is still incomplete; diagnostic text, F32.read, competing-error
order, resource, foreign/host, API and platform gaps remain open.

Next independent family: `parse/array_literal` and `parse/array_write_stmt`.
They cover concrete array construction and statement/semicolon writes returning
arrays and an element. Both are unlinked compiled candidates; actual printer
applicability and corresponding runtime observations must be established first.
Their sources were inspected only, not executed during this checkpoint.
