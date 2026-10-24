# Mixed-column reusable arguments: bounded CPU checkpoint

Tested source: `d48db566d46d0408bd2f172aca69ddf3c3d3cc23`. Target:
[Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
The [three originals](../tests/fixtures/cpu-mixed-column-reuse/README.md) retain
exact upstream bytes, Base imports, `+b` and Apache attribution. New wrappers and
tests use MPL-2.0. No production defect was demonstrated or implementation change
needed. CPU parity remains incomplete.

## Type, reuse and printer contract

| Original suffix | Pinned complete main type | Quantities |
| --- | --- | --- |
| `_003` | `@a:Nt -> @+b:Nt -> W` | Lone, Many |
| `_003_id` | `@n:Nt -> Nt` | Lone |
| `_003_sel` | `@a:Nt -> @b:Nt -> @w:Nt -> W` | Lone, Lone, Lone |

The pinned [quantity documentation](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/guide/GUIDE.md#L198)
defines ordinary binders as affine and `+` binders as reusable, requiring Data.
Nt is explicitly Data here. `_003` passes `b` to both the second argument and
`id(b)`, normalizing to `a => b => sel(a, b, b)`. The port parser maps `+` to
`Quant::Many`; the regression asserts the complete type text and directly compares
all binder quantities, domains and result structure. Normal-form and concrete-value
structure are checked separately. This preserves reuse without adding an evaluation
policy or making a claim about competing argument failures.

Actual generation was checked before the grid. These files import Base but still
return functions. The pinned [printer](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1204)
refuses function-valued pure mains. Both upstream C/JS and port C/JS/native
emitters return exit 1, empty stdout, the same complete unprintable-main diagnostic,
and no artifact. Unlike the preceding no-Base family, the original itself reaches
the function-printer boundary; no Base adapter is needed. Static compiled-candidate
classification does not imply the original is executable.

## Bounded application evidence

| Original suffix | Grid | Exact observed constructor depths |
| --- | --- | --- |
| `_003` | Nt depths 0..2 × 0..2 | `[0, 1, 1, 2, 2, 2, 2, 2, 2]` |
| `_003_id` | Nt depths 0..2 | `[0, 1, 2]` |
| `_003_sel` | Nt depths 0..2 cubed | first 3 results 0, next 6 results 1, remaining 18 results 2 |

Products are lexicographic, rightmost argument fastest. `_003_sel` varies the
unused third parameter independently; all values are valid terminating data.
The [39 exact constructor results](../tests/fixtures/cpu-mixed-column-reuse/applications.tsv)
are checked twice against the pinned interpreter, yielding 78 oracle values.
The port checked evaluator compares every value and actual core structure.
Function normal forms, CLI pure `run` text and JSON `eval` values also match.
No separate observation of the CLI writer's internal core is claimed.

Typed pure wrappers observe full Nt/W constructor depth. IO adapters print the
lists and assert `ExecutableEntry::Io` in the regression. Pure `run_main` uses
normalization; IO invokes the native executable driver. Native IO and generated
upstream/port C and JS agree on exact stdout, empty stderr and exit 0. This
qualifies the bounded applications, not the original function-valued mains or
imported-constructor spelling.

Direct upstream IO under installed Node returns exit 1, empty stdout and
`TypeError: require is not a function\n` because `import.meta.require` is absent.
Six calls remain host-blocked; no Bun or replacement host was installed. Three IO
adapters are excluded from strict proof checking, while executable acceptance is
separately established. Neither category is a runtime pass.

## Validation and provenance

| Check at tested source | Result |
| --- | --- |
| Full standard gate | 945 passed, 0 failed, 49 ignored; 108 result-bearing targets |
| Format / strict Clippy / all-feature build / TCP-UDP provider | Passed |
| Focused structured type/application test | 1 passed |
| Original normalization / type introspection | 18 / 3 observations |
| Direct oracle applications | 6 invocations, 78 values, 39 distinct fixture/input pairs |
| Original function-printer refusal | 30 exact observations; artifact absence checked |
| Accepted-source probes | 9, distinct from emission |
| Pure output / strict eval | 36 / 6 exact observations |
| Native IO and generated IO output | 30 exact observations |
| Direct IO oracle / strict IO proof | 6 host blocks / 3 exclusions |
| Generation / C build | 36 successes |
| Final matrix | 183 observations; result lanes repeated twice, no timeout or nondeterminism |
| Checking-status audit | 1,302 unchanged: 439 expected negatives rejected; others 366 accepted / 497 rejected |
| Positive / hole controls | 14 pass; 18 observations retain three unresolved cases |

The 49 ignores remain unrun: 47 hardware tests and two optional profilers. The
[kernel](../tests/kernel_mixed_column_reuse.rs) and
[compiler](../tests/compiler_mixed_column_reuse.rs) regressions ran in the full gate,
alongside existing proof, guard, cancellation and CLI boundary controls. The final
gate took 307.94 seconds from clean source. The captured binary uses
`default,extended_observability`, verified by exact artifact hash. The newest
fingerprint timestamp belonged to the Tracy build; that initial metadata selection
was corrected and retained separately, with no binary or execution change.

Initial applicability evidence uses the prior `b382a7683d2be3b06033cc4e6236595baed689f2`
binary and records 30 emitter probes plus three type observations. It is not
relabeled as current-source validation. The first gate at
`e3bf0570bc1ed20c14b34986b6a055d0f075e39a` was intentionally stopped at 36.63 seconds,
exit 143: review found that the existing value-shape helper excludes function
types. The new test was corrected to inspect type binders structurally, preserving
all existing value comparisons. The first focused restart was refused before
execution because headroom was 10.437 GiB, below the unchanged 11 GiB preflight.
Reclaiming inactive interrupted-build test caches restored it; the focused test
then passed before the renewed full gate. These are test-harness/storage events,
not suppressed language failures or changed safety budgets.

## Reproduction, storage and remaining work

```sh
source "$TOOLS_ROOT/activate.sh"
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --features extended_observability --test kernel_mixed_column_reuse --quiet
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

The primary C matrix uses the pinned Linux O3 profile; the Rust test helper
separately uses strict C flags with no explicit optimization switch. Host Rust
debug overrides do not change Bend optimization or budgets. Generated JS runs
under Node; port emission selects `--target javascript`. Private receipts preserve
exact commands, streams, statuses, source and artifact hashes; the
[public CSV](cpu-mixed-column-reuse-evidence.csv) keeps compact identities. Bounds
remain 30 seconds per matrix child, 60 per audit child, 20 per compiled test child
and 1,200 for the gate.

The corrected build began with 11.096 GiB free and its minimum
was 10.437 GiB. Build preflight remains 11 GiB, one GiB above
the 10 GiB reserve; stop floors remain 2 GiB workspace / 1 GiB temporary storage.
Only inactive rebuildable test executables were reclaimed: 104 after the stopped
build and 104 after final binary capture. Unique historical evidence,
source, dependencies and toolchains remain. Current headroom is
11.087 GiB. Recheck it before another build.

Source `d48db566d46d0408bd2f172aca69ddf3c3d3cc23` qualifies `flatten/mixed_computed_column_003`, `_id` and
`_sel` with **39 bounded applications**. Actual printer applicability was probed
before the grid: these Base-importing inventory candidates return functions,
which all compared emitters refuse with exact diagnostics and no artifact.
The original `@a:Nt -> @+b:Nt -> W` type retains reusable `b`; direct structured
binder/domain/result assertions and normalized `sel(a,b,b)` preserve that contract.
Normalization, checked applications, strict-proof scope and native IO evidence
remain distinct. No production implementation change was needed.
[Report](cpu-mixed-column-reuse.md).

Clean-source full gate: **945 passed, zero failed, 49 ignored**. The focused type/
application regression also passed. The final repeated matrix has **183 process
observations**, including six direct Node IO host blocks and three strict-proof
exclusions. All 1,302 checking statuses are unchanged; 14 positive controls pass,
and 18 observations retain three unwaived hole/TODO cases. Initial applicability
probes retain the prior binary identity; the interrupted first gate and refused
preflight are not successful validation receipts.

Detailed receipts now link **310 originals**, with **992** lacking detailed
receipts: **360 checker negatives, one checking-only module, 81 interpreter cases
and 550 compiled candidates**. Three interpretation rows are new; prior 124
interpretation, 93 runtime and 90 checking rows retain historical identities.
Function-emitter refusal does not give these originals a runtime pass. Linked
receipts include partial/blocked cells and do not establish CPU parity.
Imported-constructor printing/upstream JS timeouts, erased-hole/TODO,
host/resource/foreign/F32.read/runtime-error-order/API requirements and the
unfinished 688-vector exploration remain unwaived. Frozen dependencies,
references, toolchains, guards, budgets, cancellation and approved demand policy;
no installations, pushes, desktop or GPU work.

After this batch passed, source/applicability review selected
`shared_param_computed_000`, `cross_binder_promotion` and `var_row_column_rebind`
for a proposed 36-vector batch: ordinary parameter matching, cross-binder shadowing
and catch-all priority. No execution credit is assigned. The first fixture's old
comment mentions sharing syntax absent from its actual source; no sharing/Own
coverage is inferred from its name or comment.

Imported-printer [discrepancy and timeout evidence](cpu-ragged-rows.md#open-imported-constructor-printer-discrepancy)
remain at their earlier source `623c6bacfc82993e731c4c124306924c1d68d8eb`, not rerun
or relabeled here. The [CPU acceptance criterion](cpu-parity-acceptance.md) still
requires every applicable lane, API and failure/resource contract, not passing
counts or receipt links alone.

| Evidence identity | SHA-256 |
| --- | --- |
| Tested binary | `7a3a66d839f5b4f1a953de26d905e80c847bca9c078a8f190c0df87c8a7f425f` |
| snapshot.json | `af6e3a69c42ddf56ed95fb2d9acfd12a57df738af3e83bb19d4b25422070eda8` |
| snapshot/teamy-bend.gz | `f84b0d1a24122f8a8710d87136dcdc0f138fe946d5ffc2f4c2c33513d4667005` |
| full-gate-corrected.json | `56d6092b11be9ecb67996cb19c73829eb3d1302920c34bc4044a0c6df77f3cd3` |
| full-gate-corrected.log | `096bb5460ebaf70991234901af4ecc2af28d54b8cc741935038715653f60c837` |
| focused-types.json | `eb3400ac15d9a79d3a9b6feba2cdb00fc8dd77772754fa800f454eb45dc1fdc8` |
| qualified/receipt.json | `1f26073ec4462b5524bf7eaef9fda1cecd8eea9b83663464d2973006354ae661` |
| preflight-applicability/receipt.json | `838fcb60e7f75ea6f7cc41dc6a17156623198a862b25db3652dd82901ac5a24a` |
| audit/receipt.json | `4f58ad50b2102157733a966910672685f0fe61b564dfe2df4629d1ef4b15049a` |
| controls.json | `7c927ce9a5f8b7ee72ac3fe740eb122f7c52a9c86c14914dea37243bdf432f37` |
| coverage-summary.json | `c7898b98746c27d5b373166188a35a5efb6c1e5a3b44729af483b7eedca9a6df` |
| interrupted-cache-reclaimed.json | `c1254b40331ce4dfd32f70d851eea5344ad113fb15ed1a5bd12e91139313a843` |
| inactive-test-cache-reclaimed.json | `a96c112c24ebc710b627c9db6bee0b40bc652645a3ab6fe3bfa1ccad34018335` |
| qualify.py | `dec09037f4938d67b3db77ba6b5fde79b5037b2344c391a2b4b409bb82417b88` |
| types.mjs | `d2319d1a519234fbc7525a97f07d8ce456c3e3ef6aa14399e071265c54679709` |
