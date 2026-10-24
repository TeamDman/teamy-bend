# Typed definitions, list literals and constructor shadowing

Tested source `8218011c74b8d6878e314caaae3890f0382a702b` qualifies unchanged
parse/def_typed and parse/list_literal_sugar against upstream
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. No new acceptance or evaluation
mismatch was found. The existing constructor/parse diagnostic gap also affects
the list controls below.
The change adds six tests, original/derived attributed fixtures and the bounded
parse-typed-list comparison suite. Production source, acceptance rules,
evaluation, approved demand semantics, guards and budgets are unchanged.

## Applicability and exact results

Both originals actually generate C and JS, verified twice before comparisons:
eight successful generation observations with artifact hashes. Both have IO
mains that print U32 results. The list fixture's historical comment about a
List result does not make its scalar main non-executable. The original files
are preserved byte-for-byte under `tests/fixtures/cpu-typed-list` with Apache-2.0
attribution. Their `_pure` derivatives change only the IO main to return its
original U32 expression; they add no upstream inventory rows.

| Original | Covered behavior | Generated/native result |
| --- | --- | --- |
| parse/def_typed.bend | Typed definition telescopes, erased type argument, reusable list duplication, pair destructuring and recursive lengths | stdout `6\n`, empty stderr, status 0 |
| parse/list_literal_sugar.bend | List literal/cons construction, recursive reverse, patterns and decimal fold | stdout `3510\n`, empty stderr, status 0 |

Pinned generated C/JS, port generated C/JS and native IO all match. The **44
exact observations** comprise 32 generated-program executions, four native IO
runs, four port pure-CLI normalizations and four pinned pure-CLI normalizations.
The runner labels the port lane native, but [run_main](../src/kernel/executable.rs)
dispatches pure entries to canonical normalization. This distinction is preserved
for both derivatives.
The four original direct-IO observations remain blocked under Node by upstream's
import.meta.require behavior; no Bun installation or host adaptation is used.
The runtime runner returns 0 for completed supported lanes and retains
matched=false on the two host-blocked original rows. This is not complete
six-lane qualification.

Three pure entries (both derivative mains and list_literal_sugar_pure.main.out)
have **18 exact normalization, strict-report and actual-core observations**.
Strict eval of the original IO books remains excluded, with four repeated
refusals. These are proof-mode boundaries, not substitutes for the execution
results. No generated execution of returned functions or proof values is claimed.

## List sugar under local shadowing

The pinned [sugar table](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L56)
and [list parser](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1893)
construct fixed global Nil/Con names. The
[cons operator lowering](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2132)
also constructs global Con. Explicit constructor spellings still resolve through
the current module. The port already implements these distinctions; the Nat
literal correction is not needed again for lists.

For example, an imported helper can define its own types:

```bend
type Item is Data:
  Val{}
type List is Data:
  Nil{}
  Con{head: Item, tail: List}
def value() -> List: [Val{}]
```

`[Val{}]` still introduces global Con/Nil, so it rejects against helper.List.
Explicit `Con{Val{}, Nil{}}` is valid and normalizes as
`helper.Con{helper.Val{}, helper.Nil{}}`. With Base imported, literal constructors
belong to the global List, which remains distinct from helper.List.

Initial probes used the retained 67cdcdf binary while the documentation-only
bddc7a3 descendant was checked out. The final comparison uses a fresh immutable
binary from clean 8218011. Initial receipts keep their original identities.

Thirteen authored controls cover empty/single/two-element literals and cons
syntax with and without Base shadowing, literal patterns, explicit local
constructors/patterns, valid Base literals and root-level custom constructors.
Each runs twice through upstream direct/C/JS front ends and port check/run/eval.
The **156 observations** are:

- 108 rejections across nine invalid cases and six lanes, each twice.
- 32 successful checking/interpretation observations across four positive cases.
- Four successful C/JS generations of the valid Base control.
- Twelve build-applicability refusals for three valid no-Base controls.

Upstream's shared front end gives identical stderr/status before direct execution
or either emitter for every invalid case. Twenty-six separate structured probes
confirm eight checking failures, one parse/load failure and four acceptances,
each twice. The port's load/check regressions independently preserve those phase
boundaries. Invalid numeric/list patterns do not become executable merely
because they use local constructor names.

Positive run output and typed check/eval reports agree in their respective
contracts. The Base List control really generates C/JS; this observation alone
is not a claim about its generated execution. Valid no-Base controls check but
cannot use upstream's emitters, which require Base. None of these refusals is
counted as a successful execution.

Raw constructor and parse diagnostics remain different and **unqualified**:
upstream presents declared-constructor or List-versus-helper.List information
with Bend source excerpts; the port retains constructor errors and host context.
The [authored list ledger](cpu-list-resolution-evidence.csv) preserves this
boundary. There is no new exception. The previous 11-case Nat resolution matrix
is also renewed (132 observations) and keeps the same correct acceptances,
rejections and unresolved diagnostic distinctions. Its separate upstream phase
probe retains its historical 67cdcdf receipt; it is not relabelled as a new probe.
All these repetitions are
deterministic with no timeout at the 60-second bound.

## Validation at 8218011

| Check | Result |
| --- | --- |
| Standard Linux gate | **858 passed, 0 failed, 49 ignored**, exit 0 |
| Focused parser/template/CLI/cons/typed-list checks | **55 passed** |
| Internal depth/work guard checks | **5 passed** |
| Strict all-target/all-feature Clippy and pinned formatter | Passed in the standard gate |
| Typed/list execution/normalization suite | 44 exact, four direct-IO host blocks; supported-lane runner exit 0 |
| Typed/list normalization controls | 18 exact core/report/normalization observations; four strict IO exclusions |
| Shadowing matrix and phase probes | 156 outcomes plus 26 phase observations; acceptance and cause agree, raw diagnostics remain unqualified |
| Existing Nat resolution matrix | 132 renewed observations; no status changes |
| Five-case baseline, twice | 50 exact observations |
| 42-case normalization suite, twice | Exact structure/text; 330 successes, 108 printer refusals, six F32 strict-proof exclusions |

The six added tests are three parser and three typed/list tests. Existing 47 CUDA
and two optional-profiler ignores remain. The full 1,302-case checking-status
audit, six runtime-negative cases, ten demand controls and detailed checking
controls are not rerun by this test-only change; their receipts retain 67cdcdf
or their older actual source identities. Their resource, foreign/host,
competing-error and raw diagnostic gaps are not closed by this passing gate.
GPU, desktop and fresh Windows/macOS validation remain unrun.

Five runtime rows (two new originals and three baseline originals), all 38
interpretation rows and the authored Nat-resolution rows are current. Other
80 runtime rows, demand and checking receipts remain historical. The runtime
ledger contains **85 originals**. Runtime/interpretation union is **123**, leaving
1,179 without those receipt kinds: 439 checker negatives, twelve checking-only,
153 interpreter-only and 575 compiled candidates. These are applicability
categories, not a count of missing executable tests. The
[acceptance criterion](cpu-parity-acceptance.md) remains unsatisfied.

## Reproduction and separate host/runtime profiles

Use the existing activation, offline Cargo and BEND_GPU=off. The temporary host
Rust overrides are **CARGO_PROFILE_DEV_DEBUG=0** and
**CARGO_PROFILE_TEST_DEBUG=0**, with four build jobs. They omit Rust debug symbols;
they do not change Rust optimization or overflow checking, or Bend execution
semantics/budgets. They are not a Bend C optimization profile.

Separately, the upstream C oracle uses its pinned CLI **O3**, -lpthread -lm,
--threads 1 --gpu off. Port C retains -std=c11 -Wall -Wextra -Werror -pedantic,
links -pthread -lm -ldl and uses the same execution flags; no -O3 is silently added
to that port lane. Both JS lanes use the installed Node. These identities and
exact argv are retained in private cpu-typed-list receipts.

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
cargo test --locked --features extended_observability --test parser --test parser_templates --test cli_checker_diagnostics --test cpu_cons --test cpu_typed_list --quiet
cargo test --locked --features extended_observability --lib depth_tests --quiet
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo build --locked --all-features --bin teamy-bend --example compare_normalization
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <fresh-programs> -Suite parse-typed-list -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <fresh-baseline> -Suite baseline -Repetitions 2
python3 <private-evidence>/compare-checking.py --output <fresh-resolution> --binary <snapshot-binary>
python3 <private-evidence>/phase.py
python3 <private-evidence>/typed-list-normalization.py
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <clean-pin> -OutputDirectory <fresh-normalization> -Suite all -Repetitions 2 -RequireExactText
```

The Nat renewal uses the preceding imported-Nat comparison script and unchanged
cases, with the new immutable binary and a fresh destination. Exact hashes,
raw streams/statuses, source/command identities, deadlines and deterministic
comparisons remain private. No source/reference/dependency/toolchain or security
policy was changed, and no installation or push occurred.

Mandatory preflight initially refused the build: inactive root products alone
could not restore the **10-GiB reserve**. The identified additional growth was
1.26 GiB of Cargo incremental products. After confirming no active Cargo/rustc
or target executables and no tracked target files, only that rebuildable cache
was removed. Protected historical snapshot/receipt hashes were unchanged.
The successful retry restored 10.98 GiB before formatting/building; no budget was
lowered and the failed preflight evidence remains. Stop floors stay **2 GiB
workspace / 1 GiB temporary**. No disk exhaustion occurred. The full gate completed in 184.64 seconds with minimum headroom
9.61 GiB workspace / 4.38 GiB temporary. After validation,
headroom was 9.92 GiB workspace / 4.39 GiB temporary. Source and all historical evidence
are preserved; inspect headroom before the next build.

Next independent family: parse/match_indent_column and parse/ctor_row_layout.
Both are unlinked pure interpretation candidates without Base. Their source
comments describe case-column ownership and inline constructor-row layout;
qualification must establish actual normalization and emitter applicability.
They were inspected only, not executed. Existing resource/foreign/host,
F32.read, competing-error, constructor diagnostics, loader/API/platform and
wider-corpus requirements remain open.

## Receipt hashes

| Receipt/input | SHA256 |
| --- | --- |
| printer/receipt.json | `2a60a30af7b426f32051a10ded78e8b12f7ec96c499ea1543bdbd6498aa29b37` |
| initial-list-resolution/receipt.json | `dd6a4ebd2f815205949f190a93e76e14b7fd4b70b1dac8632922a26b076fe6e1` |
| list-resolution/receipt.json | `2e4951a5f0f12bd5e24dc7afbff87c68ee4a681d6ca471d139cc67d06534aa82` |
| phase/receipt.json | `60de625aeb232f1afe923393f329f243c446151e99148d225ba49dd0c6a29147` |
| nat-resolution/receipt.json | `75338f87c6c175ea01d982fd57768879ba43e2d2b654208f1e2e1b71cf4a3465` |
| parse-typed-list/receipt.json | `0ca6fb8cfafe4cc039f8e409d0ba57c244e4441b2b7218f8365d805ed557701d` |
| typed-list-normalization/receipt.json | `0ce35150f6e4de61490aabdac0a983adf6e0a621a6c2290be5283fc31272b11b` |
| baseline/receipt.json | `7e096eb7ecd494be12493d88d875e6335cedc5f522a804e167d2fc651906f9be` |
| normalization/receipt.json | `2b6c43498954cfc913b99e902a3317c8e407baeca98bcc7a604935d66c711d42` |
| full-gate.json | `20efb028c508d1465cf7e30115e3a49c476d181ecc4f14915241e1bc7214907d` |
| focused-clean.json | `5875bd9a0bd2d14e443f63deacca24f8afbe06af57559417f74fb3bd59467d69` |
| guards-clean.json | `791e89622c78858e9c05ff995d0cebd63bf6f8fc043715f7db4e3b3f9f0a9846` |
| preflight-focused.json | `140a162315c2bd6fda3e472cdf00a8ce7e180888eefe312b9daa35145e66f8f4` |
| incremental-clean.json | `e0b0ff749b3a6441a6cfc255ed2aa18b87322def033d316b52f841eacfadcb9f` |
| verified-summary.json | `3787253f532133c2e4c24e82da42b47da1f4b1f1d1775bbd807f8ca59a1b6e75` |
| ledger-counts.json | `e167d518f9b687fad370e8798e9c3bf98693ebd9cd9b2538d3c98f509965449d` |
| snapshots/teamy-bend | `57f3835c8ca14d516f76111ac23b19421a9a3384d80c2672d363fa9215dcc58c` |
| snapshots/compare_normalization | `7fd59feadcafa5846dd87171f74794a6e07da1a93177d0147df89651fdb97d1c` |
| Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |
