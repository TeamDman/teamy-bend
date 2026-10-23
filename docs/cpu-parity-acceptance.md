# CPU parity acceptance against the pinned Bend source

Target: [`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
The current evidence does **not** meet this criterion. Passing the repository
suite or a growing selection of examples is not sufficient to declare the pin
matched or begin adopting newer upstream code.

## Inventory and comparable execution modes

The checked-in [fixture inventory](cpu-parity-inventory.csv) contains all 1,302
pinned `.bend` fixtures, with source/expectation hashes. Its
[metadata](cpu-parity-inventory.json) records the upstream gate hash and counts:
439 expected checker-error fixtures, 1,196 fixtures declaring `main`, and 687 JS /
671 C *candidate* execution lanes. These are syntactic gate-selection counts,
not supported-feature counts or a denominator for a parity percentage.

The selection mirrors the pinned
[`gates/test.ts`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/gates/test.ts#L48):
no `main` or an expectation starting `Error:` selects checking; other mains also
select interpretation. Compiled lanes require Base and a suitable foreign twin.
An actual upstream refusal to print a function/type/erased/dependent result can
remove a compiled lane after verification. The inventory's hints cannot make
that decision or establish that a program is safe to execute.

Upstream [`book_run`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/main.ts#L454)
normalizes a pure main but dispatches IO mains through the effect driver. Map
these to the corresponding port normalizer/executable checker/native IO driver;
compare generated C and JS separately. The port's strict proof checker rejects
IO/foreign/numeric assumptions deliberately. Such a rejection is neither an
executable CPU defect nor a passing executable comparison.

The earlier 449 nonzero-exit expectations include ten runtime-failure cases,
whereas 439 expectations start with `Error:`. Therefore “449 rejections match”
does not establish negative parity. Runtime rejection at the wrong phase fails.
Expectations are stored as hashes of the original `#|` records joined with LF;
actual executions must retain separate raw stdout, stderr and process status.
Do not apply the upstream gate's combined-stream whitespace trimming to claim
that the port preserves raw program or serve-protocol bytes.

## Required evidence before calling CPU parity complete

All of the following must be satisfied for a named source commit and host profile:

1. **Complete lane review.** Every inventory row has an explicit decision for
   checker, pure evaluation, native IO, generated C and generated JS as
   applicable. Each required cell has a result. A lane may be `not_required`
   only with upstream-source or actual upstream-refusal evidence; absent port
   implementations, proof-mode exclusions used in the wrong lane, exhausted port
   budgets and unavailable oracle hosts cannot waive it. No unreviewed cells.
2. **Positive and negative behavior.** All required positive cases match their
   qualified oracle. Required negatives fail at the same language phase, name
   the same offending construct and preserve the relevant diagnostic contract.
   Raw diagnostic differences remain recorded until the CLI/API contract is
   resolved; a shared nonzero status alone never passes. No unexpected acceptance,
   crash, timeout, wrong error, altered stream or missing artifact.
3. **CPU runtime profiles.** C qualification records explicit worker/GPU flags;
   the current serial baseline is `--threads 1 --gpu off`. Parallel/scheduling
   families additionally need a declared multiworker profile and repeated runs.
   Deterministic programs must match byte-for-byte in both streams and status
   at least twice. For genuinely nondeterministic upstream behavior, establish
   the permitted outcomes/order constraints before judging the port; do not
   accept arbitrary output because a program uses concurrency. Host adapters
   and Node stack flags must be named, with unadapted failures retained.
4. **Resource and failure controls.** Keep the port's fixed safety limits,
   cancellation, writer-failure, ownership, collection and false-proof controls.
   If upstream completes an original in-scope fixture and the port exhausts a
   limit, that lane is a compatibility failure. Shortened inputs, skipped checks
   and larger port limits cannot qualify the original program. This operational
   corpus criterion is not a proof of equivalence for all possible programs.
5. **API and integration closure.** Complete a separate inventory of the pinned
   language/library/effect APIs and module/foreign/CLI/serve interfaces, including
   behavior not exercised by the fixture corpus. Every CPU requirement has an
   implementation and contract test. Arbitrary native FFI, loaders and remaining
   host effects are currently open; a function name merely existing is not
   implementation evidence. Preserve Teamy invocation/output conventions without
   silently waiving an upstream language or protocol contract. Any intentional
   compatibility exception must be named and accepted before it removes a gate.
6. **Reproducible candidate gate.** The standard repository gate, strict compiler
   warnings and applicable focused checks pass on the candidate. Record fixture,
   binary, source, dependency/toolchain and reference identities, exact commands,
   exit/status/stream bytes, deadlines and cleanup/artifact outcomes. Source or
   runtime changes require relevant receipt renewal; old platform results do not
   automatically qualify a later commit. Required failures, missing evidence and
   blocked lanes must all be zero before the corresponding scope is complete.

GPU hardware-only checks remain outside CPU qualification; CPU fallback for an
annotated program still needs testing. Headless Linux corpus qualification is a
named milestone, not full cross-platform CPU parity. Window/audio and other
host-specific CPU contracts need a suitable host; fresh Windows/macOS evidence
must remain separate. No missing CPU behavior becomes a GPU requirement.

## Evidence ledger and current open cells

The [runtime evidence ledger](cpu-runtime-evidence.csv) links **54 distinct
upstream fixtures** to source-hash-checked receipts: sixteen comptime, ten run,
eighteen IO, five Base, two representation and three parse originals. Linked
rows include unresolved negative and oracle-host cells. Two recovered port-only
regressions and an authored zero-timer control supplement the suites but add no
upstream inventory rows. Only passed cells qualify their recorded runtime/profile;
this is not whole-category, checker-diagnostic, API or multiworker completion.

There are **1,248 inventory rows without a linked runtime receipt**: 439 checker
negatives, twelve check-only modules, 164 interpreter-only cases and 633 compiled
candidates. These are different required modes, not 1,248 missing CPU runtime
tests. The historical [applicability classification](cpu-applicability.md) and
its CSV retain their earlier 1,251-row unlinked snapshot. Strict status-only
audits, source hints and related tests are not silently promoted to parity passes.
Complete per-lane applicability/printability and API review remain required.

The separate [interpretation ledger](cpu-interpretation-evidence.csv) retains
eighteen originals at `20170de`: eleven interpreter-only cases and seven compiled
candidates whose function-valued results are actually refused by upstream C/JS
generation. All eighteen match normalized core structure and raw output twice;
four authored controls add no inventory rows. Across runtime and interpretation,
**72 originals** have receipts and **1,230** lack those receipt kinds: 439 checker
negatives, twelve check-only modules, 153 interpreter-only cases and 626 compiled
candidates. Missing categories are not one runtime denominator.

The [parse-chain checkpoint](cpu-parse-chains.md), `ffebc4a`, renews 38 runtime
originals with pinned Linux C `-std=c11 -O3` and `-lpthread -lm` flags. Its three
new originals match all five primary runtimes twice (30 exact observations).
Direct upstream CLI adds two exact Boolean observations and four blocked IO
observations (`import.meta.require` missing under Node). The wider positive
matrix has 338/350 exact executions plus twelve Bun-FFI host failures. The negative
matrix remains 44/56 exact with eight resource-policy, two native-foreign and two
Bun-host gaps. Full gate: **807 passed, zero failed, 49 ignored**; strict Clippy
passes. Sixteen comptime originals retain `208b343` receipts and their
no-optimization-flag profile. Earlier O3 receipts used different link flags;
ledger/profile provenance and historical results remain explicit.

The separate [checking ledger](cpu-checking-evidence.csv) records one check-only
module with exact acceptance/report agreement and three negatives with matching
phase/offending construct but differing raw diagnostics. The latter remain
unqualified. No proof-only rejection is substituted for execution or diagnostic
parity. Structural equality retains every supported rewrite field and binding;
raw `pass_exact` is an independent requirement.

The historical [approved-demand checkpoint](cpu-approved-demand.md), `208b343`, renewed
all 51 runtime rows and nine interpretation rows at their recorded serial/host
profiles. The positive matrix remains 308/320 exact (twelve Bun-host failures),
and the six-fixture negative matrix remains 44/56 exact (eight resource-policy,
two native-foreign and two Bun-host gaps). All sixteen comptime originals pass
in the three port lanes and upstream C; default upstream JS retains four stack
failures, with exact supplemental stack-4096 results. Earlier multiworker and
direct-CLI observations retain their original receipts.

The user-approved strict live-argument, bounded-Nat and private-request rules
are implemented. The [authored demand ledger](cpu-demand-evidence.csv) distinguishes
exact results from the narrow accepted upstream-C wildcard exception. That
exception applies to `unused_rhs_error`, `ordinary_rhs_error` and the isolation
control `forced_request_wildcard`; their six C observations are explicitly not
exact matches. It does not waive the newly reproduced `nat_first` diagnostic-order
difference or the unoptimized harness profile's Clang register-allocation failure
for `word_live_tail`. The latter now agrees twice at O1/O2/O3, including pinned
cli_build's O3 profile; O0 remains a separate build limitation. See the
[source/profile investigation](cpu-failure-order.md). The new primary O3 demand
receipt at `ffebc4a` has 92/100 exact observations, six accepted wildcard
exceptions and two unresolved Nat-first diagnostic differences. Its runner
retains exit 1. Historical O2 columns keep their source/receipt identities;
earlier O0 failures remain separate. No exception extends to unreviewed failures.

Ledger states are literal: `pass` means the recorded stream/status contract
matched in the named profile; `pass_stack4096` names an explicit upstream JS
host flag; `blocked_bun_ffi` is missing upstream host support, not a pass.
`diagnostic_mismatch`, `safety_limit` and `unsupported_foreign` are unresolved
contracts, not successes. `not_required_js_only_companion` follows the pinned
gate selection, not a port exception. Candidate `cc0f3e8` identifies the
[U32 operation qualification](cpu-word-operations.md), which corrects native
packed division/modulo work exhaustion; its 48 rows retain their exact receipts.
Candidate `8e803a9` adds three [word representation](cpu-word-representation.md)
rows with all five runtime lanes passing twice. Production source is unchanged.
Direct CLI and broader checker/API coverage remain unverified. The two timer
rows still retain blocked upstream-JS lanes. Their retained C profile includes
one, two and four workers at three repetitions.
Earlier authored worker controls separately qualify request
and Nat failure outcomes at one, two and four C workers; broader profiles remain
open. Fully demanded add/double and the approved strict-demand controls are
recorded separately from this ledger's upstream-fixture rows. Only the three
named wildcard controls have an accepted C exception; competing-error order
remains unresolved. Authored controls never increase the inventory coverage count.
Private receipt hashes and binary snapshots are retained with the checkpoint
evidence; no machine paths appear here.

A direct upstream IO-CLI probe adds another distinct host limit: its
[`io_run`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1271)
passes Bun's `import.meta.require` to generated code. Under this Node host, all
three collection programs fail that direct path with `TypeError: require is not
a function`, even though separately emitted CommonJS programs pass. The
`upstream_direct_cli` column retains these blocked cells; generated-JS
success does not erase them. The parse-chain renewal separately records one
successful pure direct entry and two more blocked IO entries. No Bun install or upstream edit was
made. This is an oracle-host blocker for that lane, not a reason to stop unrelated
CPU verification or require a GPU desktop.

## Next bounded work

The [runtime-negative checkpoint](cpu-runtime-negatives.md) records all ten
runtime-negative fixtures across the ledger. Its native array-depth acceptance
defect and the [native Nat.mul budget gap](cpu-native-nat-mul.md) are corrected.
The [request fail-stop contract](cpu-request-fail-stop.md) now matches all five
runtimes twice, including exact streams/status and adjacent worker/cleanup checks.
The [Nat-overflow diagnostic](cpu-nat-failure-contract.md) also matches all five
runtimes twice. The [array/scalar correction](cpu-array-scalar-diagnostics.md)
closes all six diagnostic-only differences in the six-fixture matrix. Twelve
observations remain unresolved: eight resource-policy, two native foreign and
two oracle-host failures. These are not waivers or passes.
The [native add/double optimization](cpu-native-nat-add-double.md) originally
closed fully demanded arithmetic budget gaps while preserving lazy operands.
The subsequent [approved-demand implementation](cpu-approved-demand.md) now
closes the three unused/shallow overflow disagreements and applies private-request
refusal to the two formerly lazy RHS cases. Erasure, unselected branches, IO
ordering, first errors and fixed resource limits have regression coverage. Its
new constructor-tail compression keeps the original 10,000-element workloads
within those limits. The separately observed upstream-C competing-error order
remains unresolved; it has not been folded into the accepted wildcard exception.
The [source/profile investigation](cpu-failure-order.md) identifies C lowering
as the demonstrated cause: all O0/O1/O2/O3 profiles choose the same different
error, while a closure-call control and explicit IO binding restore agreement.
The core model specifies left-to-right reduction but excludes the runtime
numeric/foreign diagnostic cases, so it does not close that contract gap.
The Word control matches the pinned CLI-equivalent O3 build profile twice;
the separate O0 register-allocation failure remains an oracle build limitation.
Seven U32 fixtures and the operator/representation group retain exact five-runtime
results, renewed with an explicit O3 oracle profile. Eighteen original interpretation cases retain exact
structural and textual agreement, with four additional authored controls.
The `_000`, `_000_big` and `_001` Nat-fuel functions are now qualified,
including actual upstream C/JS function-printer refusals before value comparisons.
No C compilation/execution profile applies after that generation refusal. The
[application/grouping/spacing group](cpu-parse-chains.md) now has exact
five-runtime results, verified build applicability and separate direct-IO host
refusals. Its historical proof-check rejection did not predict execution failure.
Next is `parse/chain_brace_suffix`, `parse/chain_default_nat` and
`parse/chain_suffix_u32`, selected as compiled candidates from the applicability
ledger. Source expectations are not execution evidence. Applicable new C-oracle
builds use the pinned CLI's actual Linux O3/link profile; older receipts remain
distinct.
The sleep_order and
sleep_timer fixtures now have bounded four-runtime and multiworker evidence;
upstream JS needs its unavailable Bun poller. Different-duration spawn_sleep
remains unqualified: source-backed overdue registration order prevents treating
its normal-host golden as unconditional. Keep all limits
and treat C's allocation ceiling separately. Do not classify
runtime negatives as checker rejections or count phase-only agreement as exact
parity.

In parallel with later bounded families, complete lane/API review, resolve the
upstream direct-IO host requirement, and qualify multiworker/host-specific cases.
Do not declare the upstream pin matched while these required cells remain open.
New upstream adoption and any new dependencies remain separate decisions.

## Reproduction

The inventory generator only reads the clean pinned reference; it executes no
Bend program and refuses other revisions. Generate into a fresh evidence directory,
review the result, then copy the two inventory files into `docs/` deliberately:

```powershell
pwsh -NoProfile -File scripts/inventory-cpu.ps1 -Reference <reference> -OutputDirectory <fresh-inventory-dir>
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <reference> -OutputDirectory <fresh-results-dir> -Suite collections -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <reference> -OutputDirectory <fresh-negative-dir> -Repetitions 2
```

The comparison runner snapshots and hashes the port binary before execution.
It retains raw receipts and enforces 60-second child deadlines. `-Suite all`
includes baseline, effects, control, collections, scheduler, word and
representation checks. Frozen
toolchains, lockfile, pin and all previous evidence remain unchanged.
