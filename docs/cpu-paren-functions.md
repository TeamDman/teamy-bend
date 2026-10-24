# Parenthesized-function qualification

Source `5037d9ae6592211471e0361c81841481e3f3afc7` qualifies the unchanged pinned
`parse/chain_paren_group_f`, `_g` and `_h` originals. The change adds fixtures
to the existing strict normalization runner and actual-core regression loop.
No engine code, comparison vocabulary, dependency, toolchain, demand policy or
safety budget changed. No push occurred.

## Applicable behavior

| Original | Exact normal form |
| --- | --- |
| `_f` | `a => b => c => (U32.add(a, b), Nat.add(c, 1n))` |
| `_g` | `a => b => n => Nat.add(U32.to_nat(U32.mul(a, b)), n)` |
| `_h` | `a => b => c => [U32.add(a, b), U32.shl(c)]` |

Pinned interpretation, port raw `run`, strict JSON `eval` and actual normalized
core agree twice for each original, including exact whitespace and final LF in
raw output. These are function normal forms; no concrete arguments are supplied.
The tuple keeps separate U32 and Nat operations, nested multiplication remains
inside `U32.to_nat`, and the list retains its two elements and unary shift call.

The [pinned syntax rules](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L103)
state that a parenthesized `: T` selects the operator namespace and nested
parentheses inherit it. The unannotated arithmetic chains here default to Nat.
The unchanged
[`_f`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/chain_paren_group_f.bend),
[`_g`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/chain_paren_group_g.bend)
and [`_h`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/chain_paren_group_h.bend)
fixtures exercise tuple-component annotations, nested typed arithmetic and
list-element/shift syntax. All definitions are strictly checked, but checking
their helper bodies does not qualify separate selected-main observations for
the `_k` and `_m` originals.

Actual upstream C and JS generation refuses each returned function type twice:
twelve exits of 1, empty stdout, stable diagnostics and no output artifacts.
This establishes printer applicability from execution, rather than comments.
No C compilation, O3 run or generated JS execution is applicable to these
function results. Strict actual-core evidence stays separate from the existing
F32 executable-output/core-regression mode and its proof exclusions.

Original bytes and `#|` goldens are preserved with their
[Apache-2.0 attribution](../tests/fixtures/cpu-normalization/README.md).
This result qualifies normalization and presentation of these originals, not
all concrete numeric inputs, array behavior or whole-parser parity.

## Validation and provenance

| Check | Final result |
| --- | --- |
| Three parenthesized originals | Twelve exact interpreter/native outputs, six strict eval reports, six actual-core comparisons and twelve verified printer refusals |
| Full normalization | 40 fixtures: 314 successes, 100 printer refusals, six retained F32 proof exclusions; runner exit 0 |
| Focused normalization/parser/CLI tests | **59 passed, 0 failed, 0 ignored** |
| Standard repository gate | **825 passed, 0 failed, 49 ignored**; exit 0 |
| Pinned formatting, strict all-target Clippy, build | All exit 0 |

Every repeated status and raw stream is stable, and no comparison times out.
The 49 ignored checks remain 47 CUDA tests and two optional profilers. The
fixture loop gained three inputs, rather than three new test functions; test
totals therefore need not increase. No engine repair or comparison relaxation
was needed, and no candidate validation failed.

The existing wrong-reverse/lost-duplicate strict rejection assertions pass
unchanged. Their cross-implementation raw
[diagnostic mismatches](cpu-checking-control-evidence.csv) remain open at the
historical `0093d01` receipt; those CLI/phase captures were not renewed here.
Only the interpretation ledger is renewed. Runtime, checking, diagnostic-control,
demand, read, comptime and broad-audit receipts retain their actual source
identities. GPU and fresh Windows/macOS checks were not run.

After activating the frozen tools, set `CARGO_NET_OFFLINE=true` and `BEND_GPU=off`:

```sh
cargo +nightly-2026-05-24 fmt --all -- --check
cargo test --features extended_observability --test kernel_normalization --test parser --test parser_numeric --test cli_proofs --quiet
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-features --bin teamy-bend --example compare_normalization
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <clean-pin> -OutputDirectory <fresh-evidence> -Suite all -Repetitions 2 -RequireExactText
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

`-Suite paren-functions` selects the new family; the recorded run uses `all`.
Each comparison child retains its 60-second deadline. The exact commands,
statuses, streams, source and binary/probe hashes are retained privately.
Upstream remains clean at `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`,
docs at `50e89d6`, and Cargo.lock is unchanged.

## Disk safeguards and remaining work

Initial workspace headroom was **4.82 GiB**. Bounded cleanup
of 96 inactive rebuildable root-test products restored **13.95 GiB**.
The gate began with **13.09 GiB**, kept at least **4.57 GiB** workspace
and **4.84 GiB** temporary-space headroom, and finished in 157.89 seconds.
No headroom or timeout stop triggered. Final workspace headroom was
**4.57 GiB**. One immutable CLI/probe snapshot is retained for this
checkpoint; no extra snapshot deduplication was needed.

The existing cache preflight requires 12 GiB before broad builds. With builds
quiescent, it hashes and removes only inactive rebuildable root-test products;
CLI/probes, dependency libraries, source and evidence are protected. It checks
running target executables but is not a Cargo lock. The standard gate requires
the reserve and monitors 2 GiB workspace / 1 GiB temporary-space floors and a
1,200-second deadline, stopping only its own process group if necessary.
Retained binary variants and metadata-triggered relinking remain the
[documented growth cause](cpu-nat-literal-roundtrip.md#disk-cause-and-prevention).
No Bend resource limit changed and no historical evidence was removed.

| Receipt / input | SHA256 |
| --- | --- |
| normalization | `dcec411f5d78068b89988124195dfd624eff91406fdd472b6db04f397407ebb3` |
| gate-monitor | `6e2fbf974a703ced9c1113b3f3c2175da2e694a2d96e0db5dc2fe3d79a0ca4fe` |
| cache-before-build | `720f7c2816917513a5f5d6e4a7e15c6d6347811c31441d18060a9f868da6d406` |
| cache-before-gate | `ee55ecf47e71a60c4d9b4c8f12c1005f68a71b18c219af200615e0fd70f272e4` |
| Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |
| Port binary | `98243348b979be6f205d323d03dddc0567df189fe2e16743d57d53e4f9bfc027` |
| Structural probe | `eb5aad47035f69971d854c9aac3c835bf066abdd263f9beadcd8e3c577009ee7` |

The runtime/interpretation ledgers now link 111 originals. The 1,191 entries
without those modes comprise 439 checker negatives, twelve checking-only modules,
153 interpreter-only cases and 587 compiled candidates; these are distinct
requirements, not 1,191 missing runtime tests. Explicit no-main checking
receipts remain separate. CPU acceptance is still incomplete, including
diagnostic text, F32.read, competing-error order, resource policy, foreign/host,
API and platform gaps. No mismatch is waived by this result.

Next independent family: `parse/chain_paren_group_k` and `_m`, covering typed
comparison chains and array-index expressions. Both are unlinked compiled
candidates; actual printer applicability and selected-main actual-core/text
comparisons remain required. Their sources were inspected, not executed here.
