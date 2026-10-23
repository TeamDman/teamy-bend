# Rewrite-term presentation checkpoint

Candidate `571d1ce4fe648a8a0536d7ba80123105a3bc4217` closes the demonstrated
rewrite-term printer gap against
[Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
The change is confined to presentation: no checking, reduction, motive binding,
runtime demand, budget or effect behavior changes. The newly approved demand
contract is separate subsequent work; it is not implemented by this commit.

## Source rule and regression coverage

The parser stores the rewrite motive under two lambdas: its implicit endpoint
`_` and optional equation binder. The old printer exposed both internal lambdas
and lost the `%name@e` spelling. The pinned
[rewrite printer](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1411)
opens those binders, strips only annotations around their telescope and prints
the motive body in its existing scope. The port now follows that rule, including
branch precedence and the fallback for a motive without two lambda binders.
It does not substitute, normalize or mutate the motive while printing.

Regressions cover unnamed/named binders, an equation binder shadowing the outer
evidence name, nested rewrites reusing names, ordinary inner lambdas, match-branch
grouping, annotated telescopes and the non-telescope fallback. Complete applications
still compute reflexivity. New structural cases preserve evidence, the entire
two-lambda motive, body, equality carrier/endpoints, datatype arguments/exclusions
and binder distances. Counterexamples change each of these independently.
Only source-declared parameterless datatype names are resolved when comparing
parsed oracle text with the normalized core; arbitrary references are not unfolded.
Unsupported forms still fail closed. The comparator does not accept a rewrite
by ignoring its motive or by comparing opaque strings.

## Exact results

The normalization runner now covers **nine unchanged upstream originals and four
authored controls**, twice. All thirteen agree in core structure and raw term
text, with status 0 and empty stderr. The prior six operator/recursive fixtures
are renewed. Three new originals are:

| Original | Exact normalized stdout term |
| --- | --- |
| `eval/stuck_rewrite_print.bend` | `a => b => e => %e : {_ == a : U32}; {==}` |
| `proof/context_rewrite_hypothesis.bend` | `Z{}` |
| `proof/context_rewrite_value.bend` | `U{}` |

Every term has one final LF in raw `run`/upstream output. Facet `eval` retains
its `{entry,value}` envelope with the same decoded term. Authored controls print:

```text
e => %e : N; Z{}
a => b => e => %h@e : {_ == a : N}; {==}
a => b => e => %e@e : {e == e : {a == _ : N}}; {==}
e => f => %h@e : N; %h@f : N; Z{}
```

Before correction, the first printed `e => %e: _ =>  => N; Z{}`. Named cases
similarly exposed `_ => h => ...` instead of `%h@e`. The preflight preserves all
original failures. No evaluation discrepancy was demonstrated in this family.

Unlike the earlier six originals, `stuck_rewrite_print` imports Base and is a
compiled candidate in the syntactic inventory. Actual upstream C and JS generation
both refuse its function-valued result, twice, with exit 1, empty stdout, the
function-printer diagnostic and no generated file. Its compiled cells therefore
use `not_required_upstream_function_printer_refusal`, backed by actual observations.
Other originals in this suite have no Base import and are not selected for
compiled lanes. The four authored cases add no inventory rows.

Adjacent checking evidence is recorded separately in the
[checking ledger](cpu-checking-evidence.csv). `proof/rewrite_type_family.bend`
has no main and reports `All terms check.\n` exactly in both implementations,
twice. Three invalid rewrites are refused at the expected phase and offending
construct: missing evidence annotation, cross-carrier type mismatch and obsolete
motive syntax. Their raw diagnostics differ and remain **unqualified**, not exact
parity passes. The [interpretation ledger](cpu-interpretation-evidence.csv) has
nine originals; combined with the 51 runtime rows, 60 have execution evidence and
1,242 lack those receipt kinds: 439 checker negatives, twelve check-only modules,
156 interpreter-only cases and 635 compiled candidates. One check-only module
and three negatives now also have the separately labelled checking observations.

## Validation and reproduction

Full standard gate: **791 passed, zero failed, 49 ignored**. Strict all-target,
all-feature Clippy passes. Focused CLI/kernel/serve checks pass 43 tests; all ten
normalization controls were rerun after test-only Clippy corrections. The final
normalization receipt contains 104 successful interpretation/CLI/structure calls
plus four expected upstream printer refusals, all bounded and repeatable.
The positive runtime renewal is **308/320 exact**, with twelve known Bun-host
failures; its runner exits 0 with explicit blocked cells. The negative renewal is
**44/56 exact**: eight resource-policy, two native-foreign and two Bun-host gaps,
required-parity exit 1. No timeout or new runtime mismatch occurred.

With the existing activated tools, `CARGO_NET_OFFLINE=true` and `BEND_GPU=off`:

```powershell
cargo test --locked --test kernel_normalization --test kernel_core --test cli_proofs --test serve_protocol
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo build --locked --features extended_observability --example compare_normalization --bin teamy-bend
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <reference> -OutputDirectory <fresh-normalization-dir> -Suite all -Repetitions 2 -RequireExactText
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <reference> -OutputDirectory <fresh-positive-dir> -Suite all -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <reference> -OutputDirectory <fresh-negative-dir> -Repetitions 2
```

For each adjacent checking fixture, run `node main.ts <absolute-source>` from
the pinned `bend2` directory and `teamy-bend run <absolute-source>` twice, retaining
stdout/stderr/status separately. The raw checking receipt hash is in its ledger.
Private `cpu-rewrite-printing` evidence includes snapshots, commands, streams,
timings, initial build/lint errors and corrected gate logs. Normalization receipt
SHA-256: `5b6abe0d7a470dc3dfc0df0932af2ff780968afce2b7c6c26a0b7e5c058a6ce7`;
positive: `9f3d86ecc805b5a93a65d02ad517c1fd5bcd4665c50903d7f63a73575888cfa4`;
negative: `044395c04f929541297196c41408295b072f5886d741ea7800a9064136f5956a`.
Each receipt identifies its actual source revision and binary hash. Earlier
receipts are not relabelled, and no new comptime, multiworker or platform claim
is made.

Disk headroom was checked before building. Only the rebuildable incremental cache
was placed on the existing temporary volume; compiler flags and toolchains did
not change, and no source or evidence was deleted. No dependency, lockfile,
upstream/docs pin, installation or push changed. Full CPU parity remains open.
The next independent implementation is the now-approved executable demand
contract, in a separate checkpoint, preserving erasure, branch selection and IO
ordering and recording the pinned C private-request wildcard anomaly explicitly.
