# Spacing-rule function qualification

Source `f485b653d1ce11c4d1ce4726172547503eaf60d9` qualifies the unchanged pinned
`parse/chain_spacing_rules_f`, `_g` and `_h` originals. The change adds those
fixtures to the existing normalization runner and actual-core regression loop.
No engine code, comparison vocabulary, dependency, toolchain, demand policy or
safety budget changed. No push occurred.

## Applicable behavior

| Original | Exact normal form |
| --- | --- |
| `_f` | `a => b => U32.mod(a, b)` |
| `_g` | `xs => a => b => Cmp.is_gt(U32.cmp(a, b))` |
| `_h` | `n => p => 1n+Nat.add(p, n)` |

Pinned interpretation, port raw `run`, strict JSON `eval` and actual normalized
core match all three twice. The `_g` result retains the unused `xs` binder;
the `_h` result retains an open successor tail containing the whole Nat addition.
No binder erasure, arithmetic invocation or demand-policy inference is used to
make the functions agree.

The [pinned syntax rules](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L103)
describe operator namespaces, spacing before greater-than-headed operators,
nested type-argument closers and reusable binders. The three
[original sources](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse)
exercise spaced `%`, spaced `>`, `List<List<U32>>`, `+n` and `1n+p + n`.
Their comments also mention glued rewrite syntax, but these particular bodies
do not contain a rewrite; no new rewrite coverage is claimed here.

Both actual upstream C and JS generators refuse each returned function type,
twice: twelve exits of 1 with empty stdout and no output artifact. Applicability
is verified before comparison, not inferred solely from source comments. These
refusals happen before C compilation or JS execution; no compiled run or O3
result is claimed. Each original supports strict proof checking and direct
actual-core comparison. The F32 executable-output/core-regression mode and its
strict proof exclusions remain separate and unchanged.

These results qualify function normalization and presentation, not all concrete
inputs to modulo, comparison or Nat addition. Original bytes, goldens and
[Apache-2.0 attribution](../tests/fixtures/cpu-normalization/README.md) are retained.

## Validation and provenance

| Check | Final result |
| --- | --- |
| Three spacing originals | Twelve exact interpreter/native outputs, six strict eval reports, six actual-core comparisons and twelve verified printer refusals |
| Full normalization | 37 fixtures: 290 successes, 88 printer refusals, six retained F32 proof exclusions; runner exit 0 |
| Focused normalization/parser/CLI tests | **59 passed, 0 failed, 0 ignored** |
| Standard repository gate | **825 passed, 0 failed, 49 ignored**; exit 0 |
| Pinned formatting, strict all-target Clippy, build | All exit 0 |

All repetitions are stable and no comparison times out. The 49 ignored tests
remain 47 CUDA checks and two optional profilers. The focused suite includes
the existing wrong-reverse/lost-duplicate strict rejection checks, with their
specific proof-name assertions unchanged. Their cross-implementation raw
[diagnostic mismatches](cpu-checking-control-evidence.csv) remain open at the
previous `0093d01` receipt. We did not rerun or relabel those CLI/phase captures,
or substitute a shared rejection for exact diagnostic agreement.

No engine repair was needed and no candidate validation failed. The fixture
loop gained three inputs, rather than three new test functions; repository test
counts therefore need not increase. Thirty-three interpretation originals are
renewed at this source. Runtime, checking, diagnostic-control, demand, read,
comptime, broad-audit and platform receipts retain their earlier identities.
GPU and fresh Windows/macOS checks were not run.

After activating the existing frozen tools, set `CARGO_NET_OFFLINE=true` and
`BEND_GPU=off`. The executed commands are:

```sh
cargo +nightly-2026-05-24 fmt --all -- --check
cargo test --features extended_observability --test kernel_normalization --test parser --test parser_numeric --test cli_proofs --quiet
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-features --bin teamy-bend --example compare_normalization
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <clean-pin> -OutputDirectory <fresh-evidence> -Suite all -Repetitions 2 -RequireExactText
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

`-Suite spacing-functions` selects just the new family; the executed receipt is
the full `all` suite. Each child retains a 60-second deadline. Receipts identify
the clean source commit, binary/probe hashes, unchanged fixture hashes, exact
commands, streams and statuses. Upstream remains clean at
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, docs at `50e89d6`, and Cargo.lock
remains unchanged. Raw captures stay private; the hashes below identify them.

## Disk safeguards and remaining work

Initial workspace headroom was **5.07 GiB**. Bounded cleanup
of 96 inactive rebuildable root-test products restored **14.20 GiB**.
The gate began with **13.33 GiB**, kept at least **4.82 GiB** workspace
and **4.84 GiB** temporary-space headroom, and finished in 158.33 seconds.
No headroom or timeout stop triggered. Final workspace headroom was
**4.82 GiB**. This checkpoint retains one immutable CLI/probe snapshot;
no source or historical evidence was removed and no extra snapshot deduplication
was needed.

The bounded cache helper requires 12 GiB before broad builds. With builds/tests
quiescent, it hashes and removes only inactive rebuildable root-test executables,
protecting CLI/probes, dependency libraries, source and evidence. The standard
gate checks that reserve and monitors 2 GiB workspace / 1 GiB temporary-space
floors and a 1,200-second deadline. It stops only its own process group if needed.
Retained binary variants and Git-metadata-triggered relinking remain the
[documented growth cause](cpu-nat-literal-roundtrip.md#disk-cause-and-prevention).
No Bend limit was raised and no evidence was deleted.

| Receipt / input | SHA256 |
| --- | --- |
| normalization | `24c2de58437c5e01f4fd4ab686bd9ac6f853445bda9d59818189a9310475c9cc` |
| gate-monitor | `eb389a689d6a3431b89c05b45e0264ac1e8b9dc7a6bce9779699b3088081fec5` |
| cache-before-build | `b78869d4b002dde928dea125134bfdbf81ec7fb40e3a853234a7df2ebb71500a` |
| cache-before-gate | `f7e0a38154a4f9ae9c2f6dc9df782b6ceb1bef24e71f3c67daf9bebc7b5b335b` |
| Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |
| Port binary | `b766db0407e654107f4f37bc08fad3ba3d0be89050c5c821f87c3f38eee85781` |
| Structural probe | `e8766fdf9a268090786cce5da2b7cf90e614a51f1901e8ec7f84b75c99930f96` |

The runtime/interpretation ledgers now link 108 originals. The 1,194 entries
without those modes comprise 439 checker negatives, twelve checking-only modules,
153 interpreter-only cases and 590 compiled candidates; they are not 1,194
missing runtime tests. Explicit no-main checking receipts remain separate.
The acceptance criterion is still unmet, including checker diagnostic text,
F32.read, competing-error order, resource policy, foreign/host, API and platform
gaps. No existing discrepancy is waived by this normalization result.

Next independent family: `parse/chain_paren_group_f`, `_g` and `_h`, covering
tuple-component annotations, nested typed arithmetic and list-element/shift
normal forms. They are unlinked compiled candidates; actual printer applicability
and strict actual-core/text comparison are still required. Their sources were
inspected, not executed here. The `_k` and `_m` originals remain unqualified too.
