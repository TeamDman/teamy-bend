<!-- SPDX-License-Identifier: MPL-2.0 -->
# Imported declaration target selection

The reviewed implementation is `757b2b571d53c66716ba93f444f3c6fd6764f19c`, starting
from published `9b1726c3d8b158b069c54de586818c53506526fe`. The final tested source is
`e0a09940c4d0d9317dc5a457af3f256105baf7f0`; its only additional changes are a scheduler
regression test and three authored fixtures. Production parser bytes are identical.
Upstream remains `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, docs remain
`50e89d6897ce2a65167ef65b17c5fa83fe849158`, and dependencies, toolchains, demand
policy, termination, sharing, guards, resource limits and cancellation are frozen.
Full CPU parity remains incomplete.

## Corrected behavior

Both historical authored reproducers are unchanged. An alias-qualified
`def D.value() -> U32: 4` introduces a new declaration, while expression lookup
through alias `D` still reaches the imported `defs.value() = 3`. The first
reproducer now returns `3`. In an imported module, `def Nat.add() -> U32: 7`
introduces that module's qualified name instead of attempting to overwrite Base;
the second reproducer now returns `7`. They previously failed during loading with
`duplicate definition`. These are authored cases, not pinned inventory originals.

Pinned [parse_def](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2452)
uses a resolved name to complete an eligible unfilled law; a new definition uses
its raw spelling qualified by the current namespace. [Qualification and resolution](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1646)
are distinct operations. The port previously selected any resolved definition,
including filled or foreign definitions belonging to a different declaration name.

The small parser correction selects the latest resolved event first, then retains
it only for eligible completion or an actual same-name duplicate. Filtering after
the latest event is essential: the book stores an unfilled law and its later
completion separately, and an older law must never be reopened. Existing freshness
checks, same-target duplicate error/cursor behavior, template instance handling and
expression lookup remain intact. No core term, ownership mechanism or budget changes.

## Behavior and rejection evidence

The [matrix](cpu-imported-declaration-results.json) records **396 bounded commands**:
**368 matching observations**, **four upstream JS startup failures**, eight
strict-proof IO refusals and sixteen foreign load/check observations. All repeated
streams/statuses are deterministic; no timeout is accepted as a pass.

Eight positive roots include the two unchanged reproducers, an already completed
imported law, an observer that calls both imported `3` and local `4`, repeated aliases
completing an open law, Base-shadowed template specialization, and two IO wrappers.
Six pure roots normalize to `3, 7, 3, 7, 6, 7`. Their original C-O3 and JS programs
execute successfully in both implementations. Pure native `run_main` is normalization,
not native IO execution. The wrappers separately establish native IO results `3` and
`7`. In total, actual execution has **64 successful attempts and four failures**:
60 successful C/JS executions and four successful native IO executions. Pure
normalization and emitted artifacts are counted separately.

The four failed attempts are two upstream JS wrappers, each run twice. The pinned
generator emits hyphens from the relative import path into a JavaScript function
identifier, and Node exits 1 with `SyntaxError: Unexpected token '-'` before Bend
execution. Upstream C and port native/C/JS execute those wrappers correctly. The
original pure reproducers' upstream JS executions also succeed. The wrapper failure
is a retained upstream code-generation limitation, not a missing dependency or
an approved exception; no source path or generated source was rewritten to hide it.

Eight negative roots have **144 byte-exact diagnostic comparisons and 32 separate
port phase probes**. They cover repeated qualified declarations, imported Base-name
duplicates, malformed filled/open completion headers, type/template conflicts,
an invalid new body and an unfilled qualified law. All applicable CLI modes reject
with status 1, empty stdout, exact pinned stderr and no emitted artifact. Compiled
execution is unrun. Invalid new bodies now reach checking rather than being rejected
prematurely as unrelated imported duplicates. Previous duplicate-declaration and
constructor goldens remain regression controls.

Foreign controls establish loading/checking only. Strict proof loading refuses
foreign code; executable loading/checking retains the imported foreign definition
and a distinct ordinary local declaration. The same-target foreign duplicate still
rejects with its original public error/cursor. The sixteen observations are not
exact diagnostic or foreign-runtime parity claims. The inert sidecar is never
qualified as a host implementation.

## Scheduler origin regression and compatibility

The initial full gate found an older test expecting imported `library.IO.spawn`
to collide with Base `IO.spawn`. The pinned oracle accepts that qualified import
and rejects an exact root collision. The independently reviewed replacement checks
that local `IO.spawn`, `IO.sleep` and `IO.now` have no builtin metadata and no Base
origin; global names retain their exact builtin metadata and Base origin. Root
collision, underscored-symbol and forged-IO controls remain. The focused test passes.
The six separate scheduler observations establish loading/checking and root phase
agreement; they do not establish foreign execution or exact root diagnostic parity.

No public Rust signature or error type changes. The intended loader acceptance
change is observable: valid new declarations now return successful books; invalid
new bodies reach their appropriate load/check failure. Same-target duplicate errors
retain their public fields and cursor. Formatted `invoke` refusals still use the
existing `Ok(CliOutput)` with `exit_code() == 1`; callers must inspect the status.
`src/lib.rs` consumes it and dispatch propagates it. External consumers remain
unaudited. No new cancellation capability is claimed.

## Validation and reproduction

Host build overrides are separate from the Bend C runtime's pinned O3 profile:

```bash
source "$TOOLS/activate.sh"
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test cpu_declaration_targets --test cpu_duplicate_declaration --test parser_templates --test parser
cargo test --locked --features extended_observability --lib scheduler_spelling_and_user_foreign_symbols_do_not_mint_builtin_origin
cargo clippy --locked --features extended_observability --test cpu_declaration_targets --test cpu_duplicate_declaration -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

The final clean standard gate passes **1,224 tests / 0 failed / 49 ignored**, 179 result groups, in 896.80 seconds. Formatting, strict production Clippy, all-feature build and real Node TCP/UDP checks pass. The four new integration tests and previous duplicate/provenance, packed-word, host-stack, sharing, work/depth, aggregate-composition and cancellation controls pass. Existing ignores and all safety limits remain unchanged. Independent source/matrix/census review found no blocker. The targeted Clippy and corrected foreign/scheduler controls also pass; failed attempts below remain separate.

The initial gate at `757b2b5` is retained as **286 passed / one failed / one ignored**;
later targets were unrun. It failed the obsolete scheduler-name rejection, not the
new namespace safety assertions. Earlier authored foreign test attempts used a
nonexistent executable proof-normalization method, then a forbidden non-IO foreign
signature; both failures are retained. Corrected boundary checking passes without
changing either restriction. Targeted Clippy initially rejected a test struct field
name; renaming the test struct fixed it without suppression. An exploratory harness
also incorrectly expected an emitted artifact to be absent after a successful Base
IO shadow probe; that disproven hypothesis and emitted artifacts remain local.

Before observations use the captured `f0ff543052518d156537ac3380764b342d9e5dd2` binary, production-identical to the starting documentation checkpoint `9b1726c`. Its full source identity and binary hash accompany the compact before receipt.

The differential matrix retains its actual `757b2b5` identity; the final gate and
census use `e0a0994`. The intervening commit changes tests/fixtures only. Prior receipts
are never relabelled as execution of the newer commit. [Validation receipts](cpu-imported-declaration-validation.json).

## Measured census and remaining work

The complete 1,302-fixture checking census at `e0a0994` records **366 both accepted, 368 exact rejections, 495 acceptance disagreements, 68 different rejections and five oracle failures**. Every port status, phase and stream is unchanged from `f0ff543`; the two corrected authored reproducers are outside this inventory. The 385 historical IO members renew byte-for-byte. The current residual is **178 = 110 acceptance + 68 rejection differences** outside that cohort. The 386th missing-IO symptom is not silently added to the historical exclusion.

The preceding residual was 176. `flatten/literal_rows_cubic` and `reg/dead_param_fork` changed from upstream stack failures to acceptance; their unchanged port Nat-literal-above-128 refusals are measurable again. `reg/borrow_fork_hold` still fails upstream, now at checking rather than loading. These are oracle variations, not two port regressions or improvements. Five current oracle failures remain: `check/bystander_hypothesis_cost`, `check/deep_hyp_rewrite`, `page/deep_list_walk`, `reg/borrow_fork_hold` and `rfc/share_loop`. The [three-row delta](cpu-imported-declaration-census-delta.csv) preserves both recovered observations and the separate phase-only oracle transition. [Measured reconciliation](cpu-imported-declaration-census.json).

These authored cases add no links to the 1,302-original inventory. Coverage remains
**471 linked / 831 unlinked**: 521 interpreter/compiled candidates, 296 checker
negatives, 13 interpreter-only cases and one checking-only module. Links are not
universal parity, and unlinked entries are not all missing CPU runtime tests.

A separate four-command authored probe identifies the next actionable acceptance
gap: import `defs.bend as D`, where `defs` has no `local`, then declare
`def D.local() -> U32: 9` and call `D.local()`. Pinned upstream accepts and normalizes
`main` to `9`. The port loads the declaration but rejects the call while checking:
`expected: a defined name; observed: defs.local`. Its executable phase also rejects
at checking. Pinned `parse_reso` falls back to the raw spelling when the aliased
member is unknown; the port's expression resolver still returns the alias target
unconditionally. This is outside the two corrected filled-target cases, remains
unqualified, and is not included in the 396-command matrix or the original inventory.
Generated execution is unrun. **Next CPU step:** correct that fallback with known-
member, missing-member, local/global collision and template-resolution controls.
[Compact exploration receipts](cpu-imported-declaration-exploration.json).

The earlier missing-parenthesis diagnostic order remains unresolved. Upstream's
additional exclusion of Base-marked open laws from completion requires separate
characterization: a small IO-shadow probe agreed in strict checking, pure normalization
and executable loading/emission, so this batch does not claim a new defect there or
universal completion-policy parity. Its compiled execution is unrun. `nat_first`
remains pending; compact Nat/string lowering remains unadopted. Prior resource,
source-hole/TODO, F32.read, foreign/host, printer, supplementary lint and external
invoke-audit gaps remain unwaived. The new JS-wrapper oracle failures remain explicit.

Minimum recorded free space: **11.040 GiB workspace / 4.306 GiB temporary**. The 11 GiB build preflight and 10/1 GiB stop floors remained enforced. About 19 GiB of workspace tools storage is historical evidence; no source, unique evidence or cache was deleted. Existing temporary build-cache placement and disabled host debug/incremental data keep new growth bounded. No timeout or disk-floor stop occurred.

Compact sources, expected diagnostics, exact command/stream receipts, hashes and this
report are in Git. Bulk logs, generated C/JS/executables, binary/gzip snapshots and
full census JSONL remain local under `$EVIDENCE`; Git publication does not preserve
them. Historical source and unique evidence remain intact. No installation, dependency
or toolchain change, desktop/GPU work, main merge or release occurred.
