# Approved native executable demand checkpoint

Historical implementation and receipts follow. The subsequent
[recursive-companion checkpoint](cpu-recursive-companions.md) renews the affected
matrices. Its [ordering research](cpu-failure-order.md) identifies C lowering as
the competing-error cause and clarifies that the earlier unoptimized Clang
failure used a comparison-harness profile; pinned cli_build uses O3, which passes
the Word control. The older O2 receipt below remains tied to `208b343`.

Implementation: `208b343a286a8501fc3a479a4eb7b40bb01832d1`, on
`fix/cpu-approved-demand-20261023`, local and unpushed. The presentation-only
checkpoint `571d1ce` / `20ffa42` was completed first and remains separate.
Target and reference are unchanged:
[Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
Teamy documentation remains at `50e89d6897ce2a65167ef65b17c5fa83fe849158`.
No dependency, lockfile, toolchain, upstream source or GPU requirement changed.

## Implemented contract

The user approved the three choices in the [decision brief](cpu-demand-choice-brief.md):
strict live executable arguments, runtime Nat validation before shallow observation,
and continued refusal to inspect private requests, including wildcard matches.
The pinned [runtime usage account](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/docs/BendTT/main.typ#L481)
describes weak by-value executable reduction; the
[Nat guide](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/guide/GUIDE.md#L111)
sets the runtime maximum at `2^48-1`. The accepted exception concerns the
[C wildcard guard inconsistency](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L2657),
not arbitrary disagreement with upstream C.

Native IO execution now reuses the existing checked executable lowering to
remove erased computations. Native adapters retain erased argument slots, so
source arities and IO continuations remain intact. Ordinary live applications
demand their arguments; live constructor fields complete in source order.
Closures remain values: constructing an IO action does not run its effect, and
unselected branches, erased arguments/fields and closure bodies remain unevaluated.
The shared lowerer's existing compile-time reductions are unchanged. Pure-main
normalization and proof-data evaluation keep their separate interpreter semantics.

Compact Nat add/double results and ordinary successors validate the 48-bit bound
before becoming observable values. Existing sealed-body checks and ordinary
fallbacks remain. Private requests still fail through the structured language
failure boundary; no fixture-specific diagnostic matching or host-warning
suppression was added. Earlier `KernelError` private-field and `run_main` language
failure `Ok(1)` API caveats still require downstream-consumer review; this change
adds no public API signature change.

Strict construction initially exposed depth and then work exhaustion in the
original 10,000-element `hand` / `tmpl` workloads. The final evaluator compresses
only constructor-tail continuations whose earlier fields have completed. Their
unfinished results occupy existing bounded arena slots and remain inaccessible
until completion. Collection traces their fields and parent links; every
completion charges work and checks cancellation. Already-ready arguments avoid
redundant forcing. Both original workloads now succeed unchanged. The limits
remain 2,000,000 steps, 131,072 arena entries and 4,096 non-tail continuations;
output limits also remain unchanged. No source workload was shortened.

## Demand observations and narrow exception

All ten authored controls ran twice in each available lane. They do not add
upstream inventory coverage. The [demand ledger](cpu-demand-evidence.csv) records
exact classifications; the original six controls account for 60 executions.

| Control | Native, port C/JS, upstream JS | Upstream C |
| --- | --- | --- |
| Unused overflowing argument; shallow add/double overflow | Exit 1, empty stdout, exact Nat diagnostic | Exact agreement |
| Unused RHS request; ordinary constructor RHS request | Exit 1, empty stdout, exact request diagnostic | `positive\n`, exit 0: accepted wildcard exception |
| Forced private-request wildcard | Exit 1, empty stdout, exact request diagnostic | `0\n`, exit 0: accepted wildcard exception |
| Erasure, unselected branch, unused closures/IO action | Exact five-line success output, exit 0 | Exact agreement |
| Request error before Nat overflow | `before`, request diagnostic, exit 1 | Exact agreement |
| Nat overflow before request error | `before`, Nat diagnostic, exit 1 | Request diagnostic instead: **open ordering difference** |
| Live Word tail containing a private request | Empty stdout, request diagnostic, exit 1 | Default Clang compile fails; supplemental `-O2` agrees twice |

The exact diagnostics are `bend: a Nat past the largest immediate 2^48-1\n`
and `bend: runtime fail-stop\n`. No control prints `SECRET` or `after`.
The success control prints `erased\nerased\n7\nselected\nunused closures\n`.

The accepted exception is explicitly labelled
`accepted_upstream_c_wildcard_exception` for the three named controls. Those six
upstream C observations are not exact passes. The matrix has 90 exact executions,
six accepted-exception executions and two ordering differences; two Word-C
executions are unavailable because default compilation failed. Its runner exits
1, correctly retaining those open cells. All completed repetitions are deterministic.

The new competing-error example is
[nat_first.bend](../tests/fixtures/cpu-demand-review/nat_first.bend):
`pair(overflow(), invalid())`, where `pair` ignores both live arguments.
Upstream generated C jumps to `FID_INVALID` before evaluating the inline
`nat_chk(nat_mul(...)+1)` expression in a later continuation. This is visible in
the saved generated source at lines 1456–1495. CPU `err_post` immediately exits;
this is not delayed GPU error reporting. Both default and `-O2` C produce the
request diagnostic twice, while native, port C/JS and upstream JS produce the
Nat diagnostic twice. Pinned [C emission](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L2324)
and [JS emission](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L2948)
take different paths for calls and intrinsics. **Recommendation:** retain the
port's existing first-error order and review this as a separate oracle-contract
question. The approved wildcard exception does not waive it, and this checkpoint
does not change another runtime to imitate it.

For `word_live_tail`, the installed Clang reports register-allocation exhaustion
on unmodified upstream C at the default flags. The same source compiles with
`-O2` and matches the request refusal twice. Both profiles are retained; the
supplement does not relabel the failed default compilation as passed.

## Validation and exact commands

| Check | Result |
| --- | --- |
| Standard gate, Node TCP/UDP provider included | **799 passed, 0 failed, 49 ignored**, exit 0 |
| Strict all-target/all-feature Clippy | Exit 0 |
| Positive runtime matrix | 308/320 exact; twelve known Bun-host failures; runner exit 0 with explicit blocked cells |
| Six original runtime negatives | 44/56 exact; eight resource-policy, two native foreign, two Bun-host gaps; required-parity exit 1 |
| All sixteen positive comptime originals | Native, port C/JS, upstream C: 32/32 each. Default upstream JS: 28/32; four stack refusals match with `--stack-size=4096` |
| Normalization, thirteen cases including nine originals | Exact text and structural agreement twice; 104 successful commands plus four expected upstream printer refusals |
| Demand matrix | 98 executions as classified above, one failed default C build; supplemental C `-O2` controls executed four times |
| Deadlines and cleanup | Zero timeouts; demand/negative working directories empty after child exit |

The gate covers the final implementation and tests; a module-comment edit
followed its build. Strict Clippy was renewed after that edit. Differential
receipts identify the clean implementation commit and the rebuilt CLI snapshot,
SHA-256 `8227343a29521bac299e07cb063f84818859ed422e2dd6c4a7679b41e9905596`.
Focused regressions cover ordinary/large/boundary values, live and erased Word
fields, first failures, output prefix, no partial constructor escape, collection
roots, cyclic pending values, non-tail depth, work budgets and cancellation.
Historical low-level lazy evaluator controls remain separate from IO execution.

With the already activated tools and fresh `<out>` directories:

```text
export CARGO_NET_OFFLINE=true BEND_GPU=off
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo build --locked --all-features
cargo build --locked --all-features --example compare_normalization
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Suite demand -Reference <reference> -OutputDirectory <out>/demand
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <reference> -OutputDirectory <out>/negative
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Suite all -Repetitions 2 -Reference <reference> -OutputDirectory <out>/positive
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Suite all -RequireExactText -Reference <reference> -OutputDirectory <out>/normalization
```

Private `cpu-approved-demand` evidence retains the bounded comptime and `-O2`
probe scripts, exact commands, raw streams/status, binaries, hashes, source patch
and earlier failed experiments. An initial redundant test build exhausted disk;
only rebuildable root-package test executables were removed. Source, dependency
libraries, previous receipts and snapshots were preserved. The final checks ran
with recovered headroom and the existing incremental-cache placement.

Receipt SHA-256 values:

| Receipt | SHA-256 |
| --- | --- |
| demand | `c9c0f3a320de20466a804dfb7d59d3736b9fdb996f6a01887cb6b5d4cb9d06ee` |
| positive | `08ec869f88c809cfcb5b628a197e485c6a13b5a07054838b25c4f4c815f52573` |
| negative | `e36bb3487b01e05d46079713c3601086f2351b2842a180f394b07fe061d948a8` |
| comptime | `3f60e466b9ec1a9671b8b324b404c264a14fa9d9c90f9f09b4fcf221a5f23fda` |
| normalization | `bb395a6d534e14aefa8c626b17c83e5c13647f9085f6cc551024266ecb7875e8` |
| supplemental upstream C `-O2` | `c8437f4e5b7f12fba7e5ffdf51b8623fefb4ca9441bd2e67be5bae4b563824d6` |

## Remaining requirements

CPU parity is incomplete under the [acceptance criterion](cpu-parity-acceptance.md).
The three approved choices are implemented; the separate competing-error ordering
question remains open. Existing resource-policy, native foreign, Bun-host,
checking-diagnostic, wider corpus/API and platform qualification gaps remain.
All 51 existing runtime rows and nine interpretation rows are renewed where
their profiles were run; historical multiworker and direct-CLI observations
remain tied to earlier receipts. Windows/macOS, GPU and new multiworker profiles
were not rerun here. Authored demand controls do not reduce the 1,242 inventory
rows lacking execution receipts of these kinds.

The next independent small CPU family is normalized results for
`flatten/computed_recursion_002_sel`, `_003_flip` and `_004_sel`: pinned sources
exist, have function-valued mains and no Base import, and their interpretation
cells are not yet qualified. That can proceed without a dependency, GPU or
newer upstream. Review the competing-error contract separately before changing
first-error behavior or adding another oracle exception.
