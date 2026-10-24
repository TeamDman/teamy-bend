# Parameterized lists and shadowed-binder printing

Tested source: `a53eac657f82b3fa75efc3e46263f13a9be4274b`. Target:
[Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
Five [original fixtures](../tests/fixtures/cpu-list-constructors/README.md) retain
exact Apache-2.0 upstream bytes. New tests/wrappers use MPL-2.0; lowered-graph
oracle data is attributed separately. CPU parity remains incomplete.

## Demonstrated defect and scoped correction

`list_ctor_004` normalizes a branch to `b => b => c => D{a, b^1, b, c}` upstream.
The previous port printed `D{a, b, b, c}`, losing which `b` was referenced.
All 50 concrete applications still agreed before the fix. This was an ambiguous
presentation of correct binding identities, not a demonstrated evaluator defect.
The original mismatch is retained with binary source
`93d10bf440096736fd60a977032c3869d8ebc9c6`.

The [formatter](../src/kernel/term.rs) now carries a borrowed lexical scope through
existing precedence and constructor-sugar helpers. It uses semantic IDs to locate
bound variables, printing the **scope level**, not the internal ID, when a name
is shadowed. Shadowed globals receive `name^`; scheduling-marked references retain
`!`. Arrow domains and let values use the outer scope; bodies use their binders.
Rewrite motives include their two existing hidden binders. Sibling scopes do not
leak. The pinned [term_show implementation](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1297)
specifies this behavior. No term is rewritten or evaluated during printing.

The suffix is display notation, not accepted pinned source syntax. Therefore the
regression compares the actual normalized graph against a pinned oracle using
the existing bounded Shape vocabulary, retaining every bound-variable distance,
constructor, argument and match arm. Facet JSON decodes that graph; no dependency
was added. Exact golden text is checked separately. Nothing strips `^n`, relaxes
structure comparison or expands source acceptance.

[Focused formatter tests](../tests/kernel_scope_printer.rs) cover nonsequential
semantic IDs, globals, sibling scopes, arrows, lets, rewrite motives, applications,
annotations, equations and constructor sugar. Six synthetic scope examples match
pinned `term_show` exactly. Tests also confirm term debug structure and Rc ownership
counts are unchanged by formatting. Unbound variables printed without an external
context retain their earlier spelling; contextual diagnostics require separate
qualification.

The existing anonymous-rewrite regression also exposed previously lossy output:
`_ => _` in a rewrite motive references the hidden endpoint, because pinned
`parse_open` does not bind an anonymous `_`. Upstream prints `_ => _^1` there.
The regression now checks that exact output and compares the original binding
graph with an unambiguous alpha-equivalent term (`unused => _`). Four valid
round-trip cases remain. No parser change or weakening of structural comparison
was used to accommodate display-only notation. Consumers of normalized text may
now see `^n` / `^` where names were previously ambiguous; this is a presentation
compatibility change, and neither pinned nor port parsing accepts those suffixes
as ordinary identifiers.

## Type/kind and application contracts

Each original declares `List<a, -Elem: Kind(a)> is Kind(a)` with Nil/Cons.
The pinned type is `@-a:Quant -> @-Elem:Kind(a) -> Kind(a)`: both parameters are
erased. Cons retains affine head/tail fields, and its recursive tail uses the same
quantity and element identities. The tested `List<&2, Bit>` / `List<&2, Nat>`
instantiations have Data kind, while main's ordinary arguments remain affine.
See the pinned [kind documentation](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/guide/GUIDE.md#L226).
The [kernel test](../tests/kernel_list_constructors.rs) checks these identities
structurally in addition to complete main types and every normalized graph.

All originals return functions. Without Base, all compared emitters refuse with
`Error: a build needs import Base\n`; Base-importing aliases pass checking and
then fail at the actual function-printer boundary. Upstream C/JS and port
C/JS/native match complete stderr, exit 1, empty stdout and artifact absence.
Neither refusal gives the original an executable pass.

| Original | Bounded applications | Coverage |
| --- | ---: | --- |
| `_000` | 7 | Every Bit list of length 0..2; F/T head discrimination and retained tail |
| `_001` | 7 | Nat depths 0/2, lengths 0..2; predecessor head and full tail |
| `_002` | 7 | Nested Cons fallback reconstructs an unmatched head; second element and tail |
| `_003` | 7 | Empty/singleton/longer zero-headed lists and positive heads; all four tags |
| `_004` | 50 | Nat depths 0/2 × five list shapes squared; all four tags and separate payload lists |

The five selected shapes for `_004` are [], [0], [2], [0,2], [2,0]. The
[78 exact constructor results](../tests/fixtures/cpu-list-constructors/applications.tsv)
are checked twice against the pinned interpreter (156 oracle values). The Rust
checked evaluator also compares their complete structure. Local `Nil{}` prints
as `[]`, while local `Cons` stays explicit. The sample is not arbitrary-depth or
exhaustive list coverage.

Pure wrappers return `List<List<List<U32>>>`: the first inner field holds the
tag; each following field holds either one Nat depth or every list element in
order. Field boundaries distinguish an empty list from an absent field. IO
adapters print the same data; the test asserts `ExecutableEntry::Io` before using
the native executable driver. Native IO, pure observations and generated
upstream/port C and JS agree on stdout, empty stderr and exit 0. Projection does
not qualify the separately unresolved imported-constructor spelling.

Direct upstream IO under Node remains blocked: exit 1, empty stdout and
`TypeError: require is not a function\n` from absent `import.meta.require`.
Ten calls remain blocked. Five strict IO proof checks are deliberate proof-mode
exclusions, with executable acceptance checked separately. Neither is a runtime
pass; no host was installed or substituted.

## Validation and provenance

- Standard gate: **957 passed, 0 failed, 49 ignored**, 115 result-bearing targets;
  formatting, strict Clippy, all-feature build and socket-provider checks pass.
- Final matrix: **370 observations**; 30 original normalization, five main-type,
  five kind and five direct-graph probes; 10 application calls; 100 exact emitter
  refusals; 20 accepted-source probes; 60 pure output, 10 strict-eval and 50 native/
  generated IO output observations; 60 generation/build successes; 10 host blocks
  and five proof exclusions. Result lanes repeat twice, without timeout or
  nondeterminism. Generation success and phase probes are not diagnostic parity.
- Checking statuses: **1,302 unchanged**; 439 expected negatives rejected, other
  fixtures 366 accepted / 497 rejected. Fourteen positive controls pass; 18
  observations retain three unwaived hole/TODO cases. The checking audit also
  retains identical stdout/stderr to the preceding wide-scrutinee receipt;
  unchanged port diagnostics are not evidence of exact upstream diagnostic parity.
- Four focused tests passed at `c70775278f90c76296480bd138e0cabb1094f477`; they
  rerun within the final gate alongside existing guard, false-proof, cancellation
  and CLI-boundary controls. The 49 ignores remain unrun: 47 GPU hardware tests
  and two optional profilers.
- All 34 focused normalization/scope/list tests pass at `a53eac657f82b3fa75efc3e46263f13a9be4274b`.

The final gate took 314.49 seconds. Its binary uses
`default,extended_observability`, verified by matching Cargo artifact hash.
Initial probes retain the previous binary identity. Two ad hoc argument-encoding
attempts failed before producing application results; the corrected JSON builder
succeeded. The first focused build at `d92d78ade9e35830cea46f4c427df85a8c11437e`
failed because a scalar was mechanically wrapped as a term. After correction,
the first full gate at `c70775278f90c76296480bd138e0cabb1094f477` failed the
formatter's function-length lint. Extracting its existing match loop resolved
that without suppression. The following gate at
`07f819cb5f1ce0d54711ff1896560744354d625a` recorded 755 passed, one failed and
48 ignored before stopping: the anonymous-rewrite expectation still required
the old ambiguous spelling. Pinned parser/formatter evidence justified the
structural regression correction described above. Failed receipts are preserved
and not counted as passes.

## Reproduction, storage and remaining work

```sh
source "$TOOLS_ROOT/activate.sh"
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test kernel_scope_printer --test kernel_list_constructors --quiet
cargo test --locked --features extended_observability --test kernel_normalization --test kernel_scope_printer --test kernel_list_constructors --quiet
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
node "$BEND_REFERENCE/bend2/main.ts" "$ORIGINAL"
"$BEND_BINARY" run "$ORIGINAL"
"$BEND_BINARY" --output-format json eval "$ORIGINAL"
node "$BEND_REFERENCE/bend2/main.ts" "$WRAPPER" -o "$WORK/up.c"
clang-19 -std=c11 -O3 "$WORK/up.c" -lpthread -lm -o "$WORK/up"
"$WORK/up" --threads 1 --gpu off
"$BEND_BINARY" compile "$WRAPPER" --executable --target c --output "$WORK/port.c"
clang-19 -std=c11 -O3 -Wall -Wextra -Werror -pedantic "$WORK/port.c" -pthread -lm -ldl -o "$WORK/port"
"$WORK/port" --threads 1 --gpu off
```

The C matrix uses pinned Linux O3; the Rust test helper separately uses strict
flags without an explicit optimization switch. Generated JS uses Node and port
`--target javascript`. Host Rust debug overrides do not alter Bend optimization
or budgets. Private receipts keep exact commands/streams/statuses/hashes;
[public CSV](cpu-list-constructors-evidence.csv) distinguishes each contract.
Bounds remain 30 seconds per matrix/adjacent child, 60 per audit child, 20 per
compiled regression child and 1,200 per gate.

The final gate began at 11.030 GiB free and reached
10.322 GiB minimum. The unchanged preflight is 11 GiB,
one GiB above the 10 GiB reserve. After binary capture, 111
inactive rebuildable integration-test executables were retired before external
C builds. Unique source/evidence, dependencies and toolchains remain intact.
Current headroom is 11.026 GiB; preflight again before another build.
Emergency floors remain 2 GiB workspace / 1 GiB temporary storage.

Detailed receipts link **323 originals**; **979** lack detailed receipts:
**360 checker negatives, one checking-only module, 68 interpreter cases and 550
compiled candidates**. Five interpretation rows are new; prior 135 interpretation,
93 runtime and 90 checking rows retain historical identities. Linked receipts
include partial/blocked cells and do not establish CPU parity. Imported-printer/
JS-timeout, erased-hole/TODO, host/resource/foreign/F32.read/error-order/API gaps
and the unfinished 688-vector exploration remain unwaived. Dependencies,
references, toolchains, guards, budgets, cancellation and approved demand policy
remain frozen. No installations, pushes, desktop or GPU work.

Next bounded gap: `page/context_dedup_shadow.bend` still prints a free
contextual endpoint as `x` rather than pinned `x^0`. Both reject at status 1;
source location and context rows agree. The 12 adjacent observations also retain
an independent `eval/partial_app_snf.bend` difference: both succeed, but the
pinned normal form retains `snf9`/`wnf` references while the port unfolds a larger
match expression. Scope suffixes alone do not close that normalization question.
Both adjacent cases repeat deterministically without timeout; they are probes,
not newly qualified ledger rows. Investigate contextual formatter scope first,
then classify the normal-form difference structurally before changing evaluation.

Imported-printer/JS-timeout evidence remains tied to its earlier
[receipt](cpu-ragged-rows.md#open-imported-constructor-printer-discrepancy), not
rerun or relabeled here. The [acceptance criterion](cpu-parity-acceptance.md)
still requires every applicable lane and API/failure/resource contract.

| Evidence identity | SHA-256 |
| --- | --- |
| Tested binary | `b628287061b731e0089fc4e54d22e85c2ae5c7d08db2d4493d12e53f94d1bbf9` |
| snapshot.json | `0e7959dc2f76959778310403e82e3c06563686fcf92baa911e498af61e5e31f7` |
| snapshot/teamy-bend.gz | `901eff8a6252211415a4cc19c66dc03eb3934fe52c2c69401946de3440551daa` |
| full-gate-final.json | `4621d9fd39dc9b165b05f71582627e147204eb2b04ee5f3f3238dbb1d2eba34f` |
| full-gate-final.log | `60b328ec11a9e011676c7e20a09f75b1cd1f95a9da2877af7f54336fa23cb66a` |
| qualified/receipt.json | `ce232085ff71d66cf1884d77cacd44620113c2001e9acf693aed0318de853cd6` |
| preflight-applicability/receipt.json | `05066029756a6af5f2642c243c7648ea355adf94ffbf34bc1abd4b7bbdbcd6bb` |
| preflight-values.json | `0d153b5cd44ed634681f5cf1e2b92dc5feb85871b97144440b348ac332e987b2` |
| last-fixture-values.json | `519d8c19f8ad93688753871cc7180b11122d547792bd0d2664742eafc4261438` |
| focused-scope.json | `8e4ed413b93a3080ef44a5b516b667689632f2378971b103ade6301371a5695e` |
| focused-scope-corrected.json | `b5af0867e1cb34d90d00e612e65d06ccb03c68521386ab1083bfdc2500b4aa6a` |
| full-gate.json | `da4ee2f46ef3030523c687e321326ce9d6d49640aade8cc3175b034289d6cd93` |
| full-gate-corrected.json | `63c5f37bf7c1d33a251a61c00035152657d6aa190598d6208de1142a7a00abfb` |
| focused-rewrite.json | `e7a8efb126c82028b930564968b680ece56d141f125d28144b3314d10ee60112` |
| anonymous-rewrite.json | `e685050cd312de117ede4a4ae9e2d2979e5f9374af2d487ec350b3c08be1b492` |
| audit/receipt.json | `383a2a9ed445c2e0167702e652d83705988b3bdc4f2dc36a341b8d38f136ee26` |
| audit-delta.json | `e3c0db57c194157f2a9b20c394f3bcc7ca1a30fdd6e7e3e79e9d915d9b8dbe6c` |
| controls.json | `73ea959df249eb88abbb013af014e45fc6352a4bdd53f5d105e1fdd4853c2475` |
| adjacent.json | `29a04f1adf683d16195773b820437b135533a13649916b0d2240c53005f46bb0` |
| coverage-summary.json | `15cf3eb96e70f3fe51938f642fb091cf40b8df112828534ca4a1f0b0e5e6d523` |
| inactive-test-cache-reclaimed.json | `5c7496301121e4ef87d54a617e5a960e436cdc9de65887df291c15bbf400f00e` |
| qualify.py | `8602825ef399f0b8429027332a14dd8e5ab3c703a41a3f060e534326861b9072` |
| shape.mjs | `41d6cd12e79ded6a397292e3fac9727e1300b2202cd2808ef4af2aa7e490ae12` |
| scope-oracle.json | `3f436d38035c1a128eeeddb8491610ec31214b3601b7ad02f826bef270a540b0` |
