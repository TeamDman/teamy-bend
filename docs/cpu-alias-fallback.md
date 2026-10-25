<!-- SPDX-License-Identifier: MPL-2.0 -->
# Missing alias-member fallback

Tested source `4b868cfd9e5b9d5d0ab1c2d66b5fa7647e6684ed` starts from published
`9dfa4704ddc4c7f545f32e9f5a39aec97a1a812c`. Upstream remains
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, mounted docs remain
`50e89d6897ce2a65167ef65b17c5fa83fe849158`, and dependencies, toolchains,
demand/evaluation policy, guards, budgets and cancellation remain frozen.
Full CPU parity is incomplete.

## Correction

The unchanged authored reproducer imports `defs.bend as D`; that module defines
`other`, but no `local`. The root then declares `D.local()` returning `9` and calls
it from `main`. Previously the port reported undefined `defs.local`. It now accepts
and returns `9`, matching pinned upstream. Both original files are byte-identical
to the previous checkpoint's retained reproduction; they are not inventory originals.

Pinned [parse_reso](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1650)
selects one candidate: alias-qualified if an alias exists, otherwise qualified by
the current namespace. It uses that candidate only if known as a declaration,
constructor or template. Otherwise it returns the raw spelling. The port's alias
branch previously returned unconditionally. Both branches now share the existing
known-name check. An unknown aliased candidate does **not** trigger another local
namespace lookup. Existing imported members still win over colliding raw names.

Declaration freshness, latest-event law completion, template replay, lexical
variable precedence and bundled-origin checks are unchanged. No core term layout,
sharing mechanism, evaluation order, safety limit or new exception is introduced.

## Qualification

The [matrix](cpu-alias-fallback-results.json) contains **430 commands**:
**408 matching observations**, **four strict-proof IO refusals** and **18 phase-only
unknown-template observations**. There are no unexpected outcomes or timeouts.
Each repeated command has identical exit status, stdout and stderr.

Ten positive roots cover the original, known-definition precedence, a nested raw
Base-name fallback, raw-name law completion, template specialization, constructors,
known constructor/template collisions, an undotted alias head and a separate IO
program. Nine pure roots normalize to `9, 3, 5, 9, 9, 9, 9, 9, 5`. These **36 pure
normalizations** are distinct from execution. The nested control has a module-local
`Nat.add()` returning `4`, but the missing alias member must fall back to Base
`Nat.add(2n, 3n)` and return `5`. The nested negative confirms that an absent raw
name remains undefined even when a same-spelled local qualified declaration exists.

All **82 actual executions** pass: 80 port/upstream C-O3 and JS executions and two
native IO executions returning `9\n`. The original pure fixture passes generated
C/JS execution as well as normalization. Native pure `run_main` is normalization;
the IO control separately exercises the native runtime. Port C retains strict
warnings. Upstream C uses O3, one CPU worker and GPU off. Eight emitted port C/JS
artifacts for known definitions, constructors, templates and undotted names are
byte-identical to baseline; baseline emission is not counted as runtime execution.

Seven exact negatives contribute **126 byte-exact diagnostic comparisons and 28
separate port phase probes**. They cover genuinely missing names, nested raw fallback,
duplicate definitions, typed import collisions, conflicting completion, unknown
constructors and forward references. Every applicable CLI lane rejects with status
1, empty stdout, exact pinned stderr and no artifact. Negative executable runs are
unrun. Source windows and imported declaration labels remain checked.

The unknown-template case remains a diagnostic mismatch: upstream expects a term
at `~`, while the port retains its existing requirement for a previously declared
template head. Both reject during loading. The eighteen observations establish
phase/refusal behavior and unchanged port streams only, not exact upstream text.
No new exception is granted. This is the next bounded diagnostic correction to
investigate at the existing parser refusal boundary.

## Validation and compatibility

The clean standard gate passes **1,228 tests / zero failed / 49 ignored**, 180 result groups, in 917.74 seconds. Formatting, strict production Clippy, all-feature build and real Node TCP/UDP checks pass. Existing ignores and safety limits are unchanged. [Exact validation receipts](cpu-alias-fallback-validation.json).

Host build settings are separate from Bend's O3 runtime compilation:

```bash
source "$TOOLS/activate.sh"
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test cpu_alias_fallback --test cpu_declaration_targets --test parser_templates --test parser
cargo clippy --locked --features extended_observability --test cpu_alias_fallback --test cpu_declaration_targets -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

The focused suite passes **57 tests / zero failures / zero ignored**; targeted
strict Clippy passes. Existing declaration-target, duplicate, builtin-origin,
source-provenance, packed-word, sharing, host-stack, aggregate-work, depth and
cancellation regressions remain in the full gate. Nine positive controls also
compare tracked/plain checking work, normalized terms and fresh-ID behavior.
Independent source and evidence reviews found no blocker.

An initial authored nested positive placed an import after a definition. Both
parsers correctly rejected it. Its source and complete initial baseline remain
retained; the corrected positive uses a valid nested Base-name fallback instead.
No failing original, assertion or safety policy was removed. The final baseline
has separate receipts. Before observations use the captured `e0a09940c4d0d9317dc5a457af3f256105baf7f0`
binary, production-identical to starting documentation checkpoint `9dfa470`.
[Compact pre-fix evidence](../tests/fixtures/cpu-alias-fallback/before.json).

No public Rust signature or error type changes. Loader/checker outcomes intentionally
change for names formerly misresolved; genuine missing-name diagnostics now identify
the raw name required upstream. Formatted `invoke` failures still return
`Ok(CliOutput)` with exit code 1. Repository entrypoint handling is unchanged;
external invoke consumers remain unaudited. Existing unavailable-metadata, writer
failure and cancellation `Err` behavior remains covered. No mid-normalization
cancellation or foreign host-execution capability is claimed.

## Census and remaining scope

The renewed 1,302-fixture checking census records **366 both accepted, 368 exact rejections, 494 acceptance disagreements, 68 different rejections and six upstream stack failures**. Every port status, phase and stream is unchanged from the preceding `e0a09940` census. The authored fallback fixes are outside this inventory. All 385 historical IO members renew exactly; the measured residual outside that cohort is **177 = 109 acceptance + 68 rejection differences**. Symptom counts do not broaden the historical cohort.

The change from 178 to 177 comes solely from `reg/dead_param_fork` changing from upstream acceptance to a load-phase stack failure. Its unchanged port refusal is temporarily unmeasurable against a successful oracle. `reg/borrow_fork_hold` remains stack-blocked, changing from checking to loading. These are oracle variation, not a port improvement or a waiver. The other four host failures remain `check/bystander_hypothesis_cost`, `check/deep_hyp_rewrite`, `page/deep_list_walk` and `rfc/share_loop`. The [two canonical delta rows](cpu-alias-fallback-census-delta.csv) preserve these observations without rewriting historical receipts. [Measured reconciliation](cpu-alias-fallback-census.json).

No new upstream inventory fixtures are linked by these authored controls. Coverage
remains **471 linked / 831 unlinked**: 521 interpreter/compiled candidates, 296 checker
negatives, 13 interpreter-only cases and one checking-only module. Links are not
universal parity; unlinked entries are not all missing CPU execution tests.

Unknown-template diagnostics and earlier missing-parenthesis order remain unwaived.
`nat_first` remains pending, compact Nat/string lowering remains unadopted, and
prior resource, source-hole/TODO, F32.read, foreign/host, printer, supplementary
all-target lint and external invoke-audit gaps remain. This batch's successful JS
executions do not resolve the prior hyphenated-import JS generator failures or
printer timeouts. Foreign execution remains unqualified by this batch.

Minimum recorded free space: **11.065 GiB workspace / 4.209 GiB temporary**. The 11 GiB build preflight and 10/1 GiB stop floors remain enforced; no timeout or disk-floor stop occurred. Approximately 19 GiB of workspace tools storage is historical evidence. Before builds, only inactive rebuildable host build/example caches were moved to temporary storage, with their original paths preserved as symlinks. This recovered about 50 MiB of workspace allocation; no source or evidence was removed. Existing dependency-cache placement and disabled host debug/incremental settings remain.

Compact sources, exact commands/statuses/streams, hashes and reports are in Git.
Bulk logs, generated artifacts, binary/gzip snapshots and full census JSONL remain
local under `$EVIDENCE`; pushing Git does not preserve those files. No source or
unique historical evidence was deleted. No installation, dependency/toolchain
change, desktop/GPU work, main merge or release occurred.
