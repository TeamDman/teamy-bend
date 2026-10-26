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

Tested source `1fce12435a4b714f5be4ca8efcf0825fae7fc8b3` qualifies `check/computed_match_specialize`,
`cop_bare_trio_000` and `fits_residual_fallback`: **11 original definition graphs,
34 bounded applications and 15 separate executable aggregate/proof/adapter mains**.
[Report and exact receipts](cpu-match-contracts.md).

Two port defects were corrected: ordinary law completion kept the law's
provisional arity instead of the actual `def` parameters; anonymous residual
matching then failed to apply a named fallback to its residual constructor.
Checked telescope metadata now comes from bounded type exposure. Named fallback
saturation is corrected while chained empty-match reconstruction remains intact.
Independent review found no remaining blocker. Eleven new unit regressions
cover aliases, dependencies, erasure/reuse, returned closures, forged metadata,
exact work/depth boundaries and unselected fallback demand. No safety limits,
input pins, dependency or cancellation policy changed.

Full gate **1094 / 0 / 49**; focused **43 / 0 / 0**.
**216 structural**, **132 applicability/text**, **64 exact negative** and **eight
phase-only negative** observations pass. Runtime matrix has **174 exact executions,
90 successful generation/builds, 6 unresolved upstream-JS timeouts,
0 C build failures and 0 unrun C cells**. Matrix exit is
`1`; full parity is not claimed.
All 1,302 audit statuses/streams remain unchanged; 439 negatives reject. Fourteen
positive controls pass; eighteen observations preserve three hole/TODO differences.

Public Rust `DefDecl` adds `completion_parameters`: external struct literals need
updating, and unchecked source consumers must use `declared_arity()` for actual
arity. Repository callers are updated; external consumers are unaudited. Previous
external-emitter, foreign/host/resource, F32.read, failure-order, minimal-IO C,
dependent-pair C and imported-JS failures remain unwaived. No mid-normalization
cancellation is claimed. The interrupted 688-vector exploration remains unqualified.

Detailed receipts link **390 originals**; **912 remain unlinked**: **357 checker
negatives, one checking-only module, 17 interpreter cases and 537 compiled
candidates**. These are applicability categories, not missing runtime-test counts.
Links include historical, partial and blocked evidence and do not establish full
CPU parity or close the API/integration inventory.

Next proposed substantive batch: `grade/accept_data`, with applicability-led
checks of reusable Nat/list/pair/equality values, phantom tags, affine function
lists and weakening. Empty-type elimination must remain a normalization/checking
observation without fabricating an inhabitant. This is a proposal, not executed
coverage. `reg/framed_cell_dep` is a later dependent-worker/IO control.

Earlier checkpoint evidence follows and is historical unless explicitly
renewed by a later receipt.

Historical bitwise/brace evidence follows; its recorded diagnostic gap is closed
by the separate reflexivity-diagnostic checkpoint and its 67cdcdf renewal.

The [bitwise/brace checkpoint](cpu-bitwise-braces.md), source `d5d0585`, qualifies
two unchanged originals with exact outputs 671 and 42 across their available
lanes. Actual C/JS generation succeeds. Bitwise direct upstream IO remains
host-blocked and the IO book stays excluded from strict proof mode. A separately
attributed pure-entry derivative and five named definitions have exact
normal-form, strict-report and actual-core evidence. No production fix was needed.

Full gate: **838 passed, zero failed, 49 ignored**; 29 focused tests, five guards
and the 50-observation baseline pass. Complete capture: 34 exact executions,
two host blocks, four proof exclusions, 30 normalization/report/core matches,
eight false-proof rejections and two phase probes. The unused false brace proof
rejects at refl before execution, but normalized-endpoint/Bend-span versus
symbolic-endpoint/host-context diagnostics remain different and unwaived.
The supplemental runner therefore returns 1.

Five runtime rows identify this source; 76 retain historical identities. All 38
interpretation and ten demand rows remain at their previous receipts. Broader
normalization, checking audit, negative/demand, read/comptime and full runtime
matrices are not renewed. The two older checking controls remain at 0093d01;
the new brace control is separately recorded at d5d0585. Budgets, evaluators and
demand policies are unchanged; no new exception was introduced. Next: parse/cons_operator and
parse/cons_plus_pattern, with actual printer applicability first.

The preceding [array-read checkpoint](cpu-array-reads.md), 0f83aea, retains its
834-pass gate and exact historical evidence. It repaired composed Word.adc
normalization, distinct from the earlier [array-depth correction](cpu-array-depth.md)
at e9410bf/6644368. No prior receipt is relabelled as a fresh comparison here.

The preceding [comparison/index checkpoint](cpu-paren-index.md), `694a210`, qualifies
two unchanged function originals with strict actual-core/text agreement and eight
repeated C/JS printer refusals, after four initial applicability checks. Concrete
comparison inputs and array reads are not claimed. The 42-fixture normalization
matrix passes: 330 successes, 108 printer refusals, six F32 proof exclusions.
Full gate: **825 passed, zero failed, 49 ignored**; 59 focused tests, strict Clippy
and the pinned formatter pass. No engine, policy or budget change. Historical
non-normalization receipts and all unresolved diagnostic cells remain intact.

The preceding [parenthesized-function checkpoint](cpu-paren-functions.md), `5037d9a`,
qualifies three unchanged originals with strict actual-core/text agreement and
twelve verified upstream C/JS printer refusals. No concrete invocation or
generated execution is claimed. The 40-fixture normalization suite passes:
314 successes, 100 printer refusals and six F32 proof exclusions. Full gate:
**825 passed, zero failed, 49 ignored**; 59 focused tests, strict Clippy and the
pinned formatter pass. No engine, policy, comparison-vocabulary or budget change.
Non-normalization ledgers retain their historical receipts; existing diagnostic
mismatches remain unwaived.

The preceding [spacing-function checkpoint](cpu-spacing-functions.md), `f485b65`, qualifies
three unchanged originals with strict actual-core/text agreement and twelve
verified upstream C/JS function-printer refusals. No generated execution or
concrete function invocation is claimed. The 37-fixture normalization suite
passes: 290 successes, 88 printer refusals, six F32 proof exclusions. Full gate:
**825 passed, zero failed, 49 ignored**; 59 focused tests, strict Clippy and the
pinned formatter pass. No engine, demand, comparison-vocabulary or budget change.
Checking/diagnostic-control/runtime/demand and broader receipts remain historical;
the two false-list diagnostic mismatches stay open at `0093d01`.

The preceding [default-Nat function checkpoint](cpu-default-nat-functions.md), `0093d01`,
qualifies two unchanged returned functions with strict actual-core/text agreement
and actual C/JS function-printer refusals. The full 34-fixture normalization suite
passes: 266 successes, 76 printer refusals and six F32 proof exclusions. Full gate:
**825 passed, zero failed, 49 ignored**; 58 focused tests and strict Clippy pass.
No engine, dependency, demand-policy or budget change was needed.

The no-main list evidence is renewed. Its two false-proof controls parse/load,
then reject at the same reflexivity proofs; a structured oracle probe confirms
phase, endpoint values and Bend spans. Upstream normalizes stored endpoints for
display; the port formats original expressions and adds host error context.
These [diagnostic differences](cpu-checking-control-evidence.csv) remain open,
without a new exception. Runtime/demand and broader audit receipts stay historical.

The preceding [checking-only list checkpoint](cpu-list-reverse-sort.md), `c900e3f`, adds
exact acceptance/report agreement for the unchanged eval/list_reverse_sort.
Strict checking and five named-definition actual-core/text comparisons pass twice.
Two authored false equalities are rejected at the intended proofs, but raw
diagnostics differ and remain unqualified. No main, runtime lane or new exception
is invented. Full gate: **825 passed, zero failed, 49 ignored**; 58 focused tests
and strict Clippy pass. The 32-fixture normalization suite is renewed; runtime,
demand and broader audit receipts retain their actual older identities.

The preceding [Nat literal checkpoint](cpu-nat-literal-roundtrip.md), `4083b32`, qualifies the unchanged
eval/nat_literal_roundtrip original. The actual core already agreed; open Nat,
right-nested tuple and complete-list spelling now also match exactly. Both upstream
printers refuse its compound function-containing result without artifacts. Strict
eval and actual-core comparison remain independent of raw text. A standalone
open-Nat function now matches `n => 1n+n` twice. The distinct F32 executable-only
observation/core-regression mode and its six strict-proof refusals remain intact.
No comparison vocabulary, evaluation contract or safety budget changed.

The renewed normalization suite covers 32 fixtures: 250 successful observations,
68 actual printer refusals and six F32 exclusions. Full gate: **823 passed, zero
failed, 49 ignored**; 69 focused tests and strict all-target Clippy pass. The
five-runtime baseline gives 50 exact executions across three originals and two
authored regressions. Six runtime negatives remain 44/56 exact; ten demand
controls remain 92/100 exact, six approved wildcard exceptions and two unwaived
error-order observations. Their runners correctly exit 1. Nine runtime rows and
ten demand rows identify `4083b32`; the other 66 runtime rows and broader
comptime/read/checking/audit receipts retain their historical source identities.

Bounded cache preflight and gate monitoring prevented disk exhaustion: the final
gate kept at least 4.89 GiB workspace headroom. These are build safeguards, not
changes to Bend's limits. Deep raw constructor input still reaches the existing
64-level parser guard, independently of compact display of an internal spine.
That refusal is not promoted to upstream acceptance parity.

Next independent family: parse/array_read_annotation and parse/array_read_write,
requiring actual printer applicability and runtime qualification of annotated
reads and chained read/write extraction. Their checking audit status alone does
not qualify executable behavior.
Full API, lane, resource, host, foreign, F32.read, error-order and platform
closure remains required; no broader printer or arithmetic parity is claimed.

The earlier [checker-depth checkpoint](cpu-checker-depth.md),
`2f349c8`, repairs redundant
checking-to-inference phase accounting. The unchanged combined float observation
source now matches all five primary runtimes twice: ten exact stream/status
observations, repeated in the complete suite. The depth/input guards and work
budget are unchanged. Its [ledger](cpu-float-specials-depth-evidence.csv) retains
both historical refusals and current passes. Helper factoring was not substituted
for qualifying the original. Full gate: **818 passed, zero failed, 49 ignored**;
32 focused tests and strict all-target Clippy pass.

At that production checkpoint, all 75 runtime originals, the then-eighteen
interpretation originals and ten authored demand controls received primary
receipts at `2f349c8`. Positive matrix:
**618/630 exact** executions and twelve Bun-FFI failures, across 53 originals and
ten authored controls. Separate direct entries give four exact and 44 host
failures, plus six independently renewed collection host failures. Negative:
**44/56 exact**, with eight resource-policy, two native-foreign and two Bun-host
gaps. Demand: **92/100 exact**, six approved wildcard exceptions and two unwaived
Nat-first diagnostic-order differences. Comptime: **156/160 primary exact**;
default upstream JS fails hand/tmpl four times, while four supplemental
stack-4096 observations pass. Negative/demand/comptime runners retain exit 1.
All repeats are stable; no comparison timeout occurred.

The renewed [read ledger](cpu-float-read-evidence.csv) still records three
unresolved upstream C/JS contracts: decimal double-rounding, hexadecimal input
and embedded NUL. Native/port C match C; port JS matches JS. A fourth final-newline
input returns None everywhere. All ten executions exit 0 with empty stderr, but
the probe retains exit 1. Corresponding-backend agreement does not close these
cross-backend cells. No parsing policy, tolerance or new exception is chosen.

Current C matrices use pinned Linux O3/link flags. Historical O0/O2, multiworker,
platform and failure-order investigations retain their actual commits and do
not qualify this new source. The renewed status-only strict audit accepts 366
expected positives, refuses 487 and rejects all 449 nonzero-expectation fixtures,
including all 439 checker negatives. No acceptance status changed from bd6a228;
no abnormal exit or timeout occurred. Historical applicability metadata stays
unchanged; these statuses do not qualify unexecuted runtime or raw diagnostic cells.

The separate [checking ledger](cpu-checking-evidence.csv) records two check-only
modules with exact acceptance/report agreement: rewrite_type_family at `2f349c8`
and list_reverse_sort renewed at `0093d01`. Its three earlier negatives retain matching
phase/offending construct but differing raw diagnostics at `2f349c8`. The new
list checkpoint's two authored false-proof controls also retain diagnostic
differences and add no inventory rows. None is promoted to exact negative parity.
The twelve no-main entries absent from runtime/interpretation ledgers therefore
include two explicitly qualified checking positives, not twelve missing runtime
tests. Structural equality and raw exactness remain separate requirements.

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
receipt renewed at `2f349c8` has 92/100 exact observations, six accepted wildcard
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
Broader direct CLI, checker and API coverage remains unverified. The two timer
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
successful pure direct entry and two more blocked IO entries; operator-chain
adds three blocked IO entries and float syntax adds one pure and two blocked IO
entries. The float-runtime, float-math, float-text, float-arithmetic and
float-specials groups each add three blocked IO entries. All twenty-four
parse/compile/base/run-float direct-entry rows were renewed at `9b18fbb`.
No Bun install or upstream edit was made. This is an oracle-host blocker for
that lane, not a reason to stop unrelated CPU verification or require a GPU desktop.

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
[application/grouping/spacing group](cpu-parse-chains.md) and
[brace/numeric operator chains](cpu-parse-operators.md) have exact five-runtime
results with verified generation applicability and separate direct-IO host
refusals. Their proof-check exclusions did not predict executable failure.
The [float syntax/pattern group](cpu-parse-floats.md) retains its three scoped
corrections. The [float-runtime group](cpu-float-runtime.md) covers rounding,
accumulation and F32 storage. The [math group](cpu-float-math.md) now has exact
text and bit observations for twelve unary/trig/hyperbolic result expressions,
with no tolerance or cross-platform precision claim. The [text group](cpu-float-text.md)
now qualifies run/float_ops, float_printing and float_text, separating exact
original results from three unresolved runtime-read contracts. The
[arithmetic group](cpu-float-arithmetic.md) now qualifies base/float_roundtrip,
compile/float_roundtrip and run/float_arithmetic, with exact bit/text controls.
The [specials group](cpu-float-specials.md) now qualifies run/float_text_roundtrip,
compile/float_compare and run/float_specials, without a NaN payload claim.
Its combined authored control remains blocked by checker depth and needs a
bounded traversal/expansion investigation. An independent next family is
`parse/chain_suffix_f32_f`, `_g`, `_h`: function-valued mains whose normalized
structure/text and actual printer applicability remain unqualified. Candidate
compiled hints do not establish runnable results. New C-oracle builds retain
pinned Linux O3/link flags; older profiles remain separate.
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
includes baseline, effects, control, collections, scheduler, word,
representation, parse-chain/operator/float, float-runtime, float-math, float-text,
float-arithmetic and float-specials checks. The separate
`scripts/compare-float-read.py` probe records backend matches and cross-oracle
disagreement independently; it keeps exit 1 until the read contract is resolved.
Frozen toolchains, lockfile, pin and all previous evidence remain unchanged.

The `-Suite float-specials-depth` probe retains a failure for the combined
authored checker-depth reproducer. Its exclusion from the positive suite is not
a waiver; the separate depth ledger remains a required open resource cell.

## Constructor/import diagnostic checkpoint

Source `6442b2ca1362e36f84b74044091a0151fb17726d` qualifies five additional
original checker negatives and repairs the shared tracked-constructor
presentation boundary. The [report](cpu-constructor-diagnostics.md) retains
four unresolved diagnostic controls; phase/status agreement does not waive
them. Full gate: 866 passed, zero failed, 49 ignored. The renewed 1,302-fixture
status audit has zero transitions, including rejection of all 439 expected
negatives. It remains status-only evidence.

The category ledger now links 140 originals, including historical/unresolved
cells; 1,162 lack detailed receipts. This does not meet complete lane/API review
or establish full CPU parity. Existing resource, host, foreign, F32.read and
first-error-order requirements remain open. The next bounded diagnostic step
is explicit provenance for rebuilt constructors, not changed language policy.

## Constructor provenance checkpoint

Source `d986964456afaf7622628144da2e2745db740691` closes the four diagnostic
controls left by the previous constructor checkpoint. The
[provenance report](cpu-constructor-origins.md) records 34 exact negatives,
eight unchanged positives and 84 phase probes across a 42-case matrix. The
scoped summary exits 0; prior failing receipts remain historical evidence.
Full gate: 871 passed, zero failed, 49 ignored; the 1,302-fixture status audit
has zero transitions. Origin tracking leaves term structure, sharing, evaluation
and budget accounting unchanged in the qualified private load/check path.

The inventory has no new original rows: 140 detailed, including historical/
unresolved cells, and 1,162 without detailed receipts. All broader lane/API,
resource, host, foreign, F32.read and error-order requirements remain unmet where
previously recorded. This diagnostic closure does not establish full CPU parity.

## No-main and multi-scrutinee checkpoint

Source `2417a27107e4adb5d5546ace6cd4e854fc2a0a92` adds nine checking-only and
seventeen interpreter-only originals; [report](cpu-no-main-interpreter.md).
No-main checking and thirty named normal forms match. Executable emitters now
refuse absent main; their prior successful-stub API behavior is intentionally
corrected, with external consumers unaudited. Compilation stderr differences
and one false-fusion rewrite type-mismatch diagnostic remain unmet, not waived.

Full gate: 876/0/49; 66 normalization cases and 50 baseline executions pass.
All 1,302 checking statuses are unchanged. Detailed coverage is 166 originals,
including historical/unresolved cells, versus 1,136 without detailed receipts.
No prior resource, host, foreign, API, F32.read or error-order requirement is
removed. These results do not establish full CPU parity.

## Compilation/rewrite and nested-field checkpoint

Source `02b244e529e978a683b7f77a79ae04089665748d` closes the preceding no-main
compilation stderr and false-fusion type/source diagnostic gaps. The
[report](cpu-compile-rewrite-diagnostics.md) records preserved rejection, the
repository caller audit and explicit external API caveats. Ten unchanged
nested-field originals match exact core/text; 171 concrete application vectors
are verified against the pinned interpreter. Function printing remains inapplicable.

Full gate **882 passed, zero failed, 49 ignored**; 76 normalization cases,
306 no-main observations, 60 rewrite/phase observations, 144 prior proof controls
and 50 baseline executions pass. All 1,302 checking statuses are unchanged.
Coverage is 176 detailed originals / 1,126 without detailed receipts, including
historical and unresolved cells. The existing resource, host, foreign, API,
F32.read and error-order requirements are unchanged; full CPU parity is incomplete.
Next independent family: four nested_multi_scrutinee originals. No push.

## Nested patterns checkpoint

Source `062c7ec150168922ff642d07b1fc72ce5c14144c` qualifies four nested multi-scrutinee,
five nested-constructor and five fallback-reconstruction originals. All fourteen
match exact interpreted core/text and 476 concrete application vectors agree
with the pinned interpreter. The [report](cpu-nested-patterns.md) distinguishes
unprintable function results from generated execution. No production source or
language contract changed.

Full gate **883 passed, zero failed, 49 ignored**; 90 normalization cases,
306 no-main observations, 60 rewrite/phase controls, 144 prior proof controls
and 50 baseline executions pass. All 1,302 checking statuses remain unchanged.
Coverage is 190 detailed originals / 1,112 without detailed receipts, including
historical and unresolved cells. Full CPU parity remains incomplete; no prior
host/resource/foreign/API/F32.read/error-order requirement is waived. Next:
field-match-order and destructuring-let applicability/qualification. No push.

## Field-order/destructuring checkpoint

Source `de31e54e8e35f35b1c0fa4c05415d8dd91ca9b10` qualifies eighteen unchanged field-order/destructuring
originals: sixteen function mains with exact core/text and 399 concrete applications,
one IO main with ten exact generated/native executions, and one exact parse/load
negative. The private binder-origin diagnostic fix preserves public ParseError,
terms, evaluation, budgets and rejection. [Report](cpu-destructure.md).

Full gate **889 passed, zero failed, 49 ignored**; 106 normalization cases,
306 no-main observations, 60 local-binder and 60 rewrite observations,
144 prior proof controls and 50 baseline executions pass. All 1,302 checking
statuses are unchanged. Direct upstream IO remains host-blocked; the 688-vector
exploratory remainder and historical unmet cells remain unqualified.

Coverage is 208 detailed originals / 1,094 without detailed receipts, including
historical/unresolved cells. Category counts are 9/11/58/130 and remaining
430/1/106/557. Full CPU parity remains incomplete; host/resource/foreign/API,
F32.read and competing-error requirements are unwaived. Next: the reusable
plus-binder family with its negative companion. Final headroom 10.04 GiB;
all evidence retained, frozen inputs/policies, no push.

## Plus-binder checkpoint

Source `e9d9b30c5029e18dc5600b407e5e0abab94a8b99` qualifies four original plus-binder IO mains and their
negative companion. Forty generated/native executions and ten helper core/text
normal forms match. The shared diagnostic boundary now preserves marker spelling,
import-filtered source excerpts and executable-compilation rejection without
changing term identity, acceptance or safety budgets. [Report](cpu-plus-binders.md).

Full gate **896 passed, zero failed, 49 ignored**. Renewed: 216 new negative,
588 constructor/origin, 306 no-main, 144 proof, 60 local-binder and 60 rewrite
observations; all 106 normalization cases and 50 baseline executions. All
1,302 checking statuses are unchanged. Direct upstream IO remains host-blocked.
The prior 688-vector exploratory remainder remains unqualified.

Coverage: 213 detailed originals, including historical/unresolved cells, versus
1,089 without detailed receipts. Remaining categories: 429 checker negatives,
one checking-only module, 106 interpreter cases and 553 compiled candidates.
Full parity remains incomplete. Next: the three recorded rewrite diagnostic
differences. Shared snapshots reduce new evidence duplication; all historical
and failure evidence survives. Frozen inputs/policies, no installs or pushes.

## Rewrite checking checkpoint

Source `4c90907a78db96cc9e46da6d25659f10b4d978cc` closes exact diagnostics for rewrite_non_equation,
rewrite_ctor_brace and dead_motive_syntax without changing their failure phases.
The first was already exact; the other two now retain normalized content and
original source through the existing failure boundary. Equality printing uses
pinned operand precedence. [Report](cpu-rewrite-checking.md).

Full gate **901 passed, 0 failed, 49 ignored**. The new 260-observation
matrix and renewed normalization/constructor/proof/no-main/runtime controls pass.
All 1,302 checking statuses remain unchanged. Coverage stays 213 detailed originals
and 1,089 without detailed receipts; linked rows include historical/unresolved
cells. Twenty checking, 99 interpretation and seven runtime originals are renewed.
The 688-vector exploration remains unqualified; existing host/resource/foreign,
F32.read, error-order and API requirements remain open. External invoke callers
must inspect exit_code; they are not audited here.

Next: annotated non-equation evidence, generated rewrite proof-binder provenance,
and missing-IH diagnostics. Frozen inputs/policies; historical evidence preserved;
no installs or pushes.

## Rewrite proof checkpoint

Source `abd6d22b1a0d53420a6f60423b381b37137cccc3` closes annotated non-equation, substituted proof-use and
non-function application diagnostic gaps. Five unchanged originals and fifteen
complete adjacent controls match the pinned checker; five prior controls also
renew. The [report](cpu-rewrite-proof.md) preserves the separate hole-preflight
ordering and computed-scrutinee load diagnostic mismatches as unresolved.

Full gate **905 passed, zero failed, 49 ignored**; 500 negative observations,
four positive no-main checking observations and all 1,302 checking statuses pass
with zero status transitions. Detailed coverage is 218 originals / 1,084 without
receipts: 424 checker negatives, one checking-only module, 106 interpreter cases
and 553 compiled candidates. Linked rows include historical and unresolved cells.
Only nine checking originals are current; unrelated detailed lanes retain their
actual source identities. The 688-vector exploration remains unqualified.

The existing acceptance criterion and external invocation-status caveat remain.
No semantic exception, dependency/toolchain/pin or budget change; no push.
Next: computed-match scrutinee diagnostic provenance and adjacent negatives.

## Computed scrutinee checkpoint

Source `75085cb0a27dd8fb01f6bf50bb5e03198dddda80` fixes computed-scrutinee provenance for two unchanged
originals and the three preceding hole-order defects. Structural limits remain
upfront; checker rule order selects the error; saved source-hole refusals prevent
erased/unreachable holes from escaping. [Report](cpu-computed-scrutinee.md).

Full gate **911 passed, zero failed, 49 ignored**; 560 negative observations
(504 exact stream/status results plus 56 phase probes), 14 positive controls,
and all 1,302 checking statuses pass. Five separately retained controls remain
unresolved: constructor/global scrutinee diagnostics and erased-hole/TODO policy
or diagnostic differences. No new exception or runtime demand change is adopted.

Detailed coverage: 220 originals / 1,082 without receipts, split as 422 checker
negatives, one checking-only module, 106 interpreter cases and 553 compiled
candidates. Eight checking rows are renewed; other detailed lanes retain historical
identities. The 688-vector exploration and existing host/resource/foreign/F32.read,
runtime-error-order/API/lane requirements remain open. External invoke callers must
inspect exit_code. Next: substituted/consumed/out-of-order scrutinee diagnostics.
Frozen inputs and safety limits; historical evidence preserved; no installs/pushes.

## Scrutinee occurrence checkpoint

Source `1175aa37f317db553a2e596d16d0089297f1fa40` closes substituted, consumed and out-of-order scrutinee diagnostics
for eleven unchanged originals, plus constructor and bare-global matches. Per-use
surface metadata preserves source locations without changing core terms, sharing,
acceptance, evaluation, approved demand semantics or safety limits.
[Report](cpu-scrutinee-origins.md).

Full gate **915 passed, zero failed, 49 ignored**. The 31-case repeated matrix
contains 558 exact stream/status observations and 62 separate phase probes;
14 positive controls pass. All 1,302 checking statuses remain unchanged.
The three erased-hole/TODO controls remain unresolved and unwaived. Single probes
also retain matched-value checking and row-arity diagnostic gaps, not qualifications.

Detailed coverage: **231 originals / 1,071 without detailed receipts**: 411 checker
negatives, one checking-only module, 106 interpreter cases and 553 compiled
candidates. Eighteen checking rows are current; 21 other checking, 99 interpretation
and 93 runtime rows retain historical identities. Linked receipts include unresolved
cells and do not establish universal parity. The incomplete 688-vector exploration,
host/resource/foreign/F32.read, runtime-error-order/API/lane requirements remain open.
External invoke consumers must still inspect exit_code; no external audit is claimed.
Next: reconstructed matched-value and row-arity diagnostics. Frozen inputs/policies;
no installations, pushes, desktop or GPU work.

## Matched-value and row-arity checkpoint

Source `77116c8ab4ab8eabbaea599679996941e41ed3b4` closes reconstructed matched-value and row-arity diagnostics for fourteen
unchanged originals, with fifteen authored controls. Per-use return provenance is
carried through generated match/lambda/let edges outside core terms. Constructor
checking and row-arity rejection keep their original failure decisions and budgets.
[Report](cpu-matched-values.md).

Full gate **920 passed, zero failed, 49 ignored**. The 37-case repeated matrix
contains 666 exact stream/status observations and 74 separate phase probes;
14 positive controls pass. All 1,302 checking statuses remain unchanged from
1175aa3. Three erased-hole/TODO disagreements remain unresolved and unwaived.
Next bounded probes expose empty-match inference, missing-case and unsupported-pattern
diagnostics; these probes are not counted as qualifications.

Detailed coverage: **245 originals / 1,057 without detailed receipts**: 397 checker
negatives, one checking-only module, 106 interpreter cases and 553 compiled
candidates. Twenty-two checking rows are current; 31 other checking, 99 interpretation
and 93 runtime rows retain historical identities. Linked receipts include unresolved
cells and do not establish universal parity. The incomplete 688-vector exploration,
host/resource/foreign/F32.read, runtime-error-order/API/lane requirements remain open.
External invoke consumers must still inspect exit_code; no external audit is claimed.
Frozen dependencies, references, toolchains, guards and demand policy; no installations,
pushes, desktop or GPU work. Independent CPU progress is not blocked on a decision.

## Empty-match and pattern diagnostic checkpoint

Source `d82417425b9c46f6ac7a63a39858987ca965b5bf` closes empty-match inference, missing-case and unsupported-pattern
diagnostics for **29 unchanged originals**, with fifteen authored controls. Private
match provenance survives flattening and substitution; diagnostic contexts use
binder admission order without changing semantic lookup or dead-context traversal.
[Report](cpu-match-failures.md).

Full gate **926 passed, zero failed, 49 ignored**. The repeated 52-case matrix
contains 936 exact stream/status observations and 104 separate phase probes;
32 positive controls pass. All 1,302 checking statuses remain unchanged from
77116c8. Three erased-hole/TODO disagreements remain unresolved and unwaived.
Bounded probes expose adjacent match-domain, mismatched/repeated-constructor and erased
scrutinee diagnostics; those single probes are not qualifications.

Detailed coverage: **274 originals / 1,028 without detailed receipts**: 368 checker
negatives, one checking-only module, 106 interpreter cases and 553 compiled
candidates. Thirty-seven checking rows are current; 45 other checking, 99 interpretation
and 93 runtime rows retain historical identities. Linked receipts include unresolved
cells and do not establish universal parity. The incomplete 688-vector exploration,
host/resource/foreign/F32.read, runtime-error-order/API/lane requirements remain open.
External invoke consumers must still inspect exit_code; no external audit is claimed.
Frozen dependencies, references, toolchains, guards and demand policy; no installations,
pushes, desktop or GPU work. Independent CPU progress is not blocked on a decision.

## Match guard diagnostic checkpoint

Source `a241e18b5b262453397e6195733e790ed83ee244` closes match-domain, wrong/repeated-constructor and erased-scrutinee
diagnostics for **eight unchanged originals**, with sixteen authored controls.
Private constructor spans survive flattening/substitution; message-only checker
reports use the existing status-bearing boundary. Rejection and evaluation remain
unchanged. [Report](cpu-match-guards.md).

Full gate **929 passed, zero failed, 49 ignored**. The repeated 33-case matrix
contains 594 exact stream/status observations and 66 separate phase probes;
32 positive controls pass. All 1,302 checking statuses remain unchanged from
d824174. Three erased-hole/TODO disagreements remain unresolved and unwaived.

Detailed coverage: **282 originals / 1,020 without detailed receipts**: 360 checker
negatives, one checking-only module, 106 interpreter cases and 553 compiled
candidates. Seventeen checking rows are current; 73 other checking, 99 interpretation
and 93 runtime rows retain historical identities. Linked receipts may include open
cells and do not establish universal parity. The incomplete 688-vector exploration,
host/resource/foreign/F32.read/runtime-error-order/API requirements remain open.
Next substantive batch: five `flatten/ragged_rows_000..004` originals, covering
first-match priority, overlapping defaults, reconstructed values and bounded recursion.
Initial applicability probes remain unqualified; structural and concrete-application
comparisons are required. Dependencies, references, toolchains, budgets and approved
demand policy remain frozen. No installations, pushes, desktop or GPU work.

## Ragged-row concrete application checkpoint

Source `623c6bacfc82993e731c4c124306924c1d68d8eb` qualifies five unchanged `flatten/ragged_rows_000..004`
originals through exact normal forms and an **81-vector** direct application grid.
Separate pure observation wrappers and IO adapters compare every constructor ID
and returned field depth across applicable runtimes. The IO adapters explicitly
exercise the native executable driver; pure CLI results exercise normalization.
Known printer refusals now use the structured language diagnostic boundary.
[Report](cpu-ragged-rows.md).

Full gate: **933 passed, zero failed, 49 ignored**. The repeated pure matrix has
140 process observations; the IO matrix has 100, including ten unavailable direct
Node IO-oracle calls and five deliberate strict-proof exclusions. The 14-case
printer matrix has 112 exact comparisons plus 28 oracle captures, with 56 separate
accepted-source phase probes. All 1,302 checking statuses remain unchanged.

Detailed receipts cover **287 originals**, with **1,015** lacking detailed receipts:
**360 checker negatives, one checking-only module, 101 interpreter cases and 553
compiled candidates**. Five interpretation rows are renewed; the other 99 interpretation,
93 runtime and 90 checking rows retain historical identities. Wrappers do not
create executable qualification for the original function-valued mains.

Imported-datatype printing is a newly minimized open discrepancy: the interpreter,
upstream C and native normalizer print qualified names, port C/JS print local names,
and pinned JS reaches an unmatched-tag printer loop. This is not an exception.
Erased-hole/TODO, host/resource/foreign/F32.read/runtime-error-order/API requirements
and the incomplete 688-vector exploration remain unwaived. Next independent batch:
six `computed_ref_call_000/001` originals and their `_f`/`_step` companions, initially
20 concrete vectors; selected from source and ledger, not executed or qualified.
Frozen inputs, budgets, guards and demand policy; no installations, pushes, desktop
or GPU work.

## Computed reference-call checkpoint

Source `8deca787667b9d01d40a865ca71fba6d527fdfdd` qualifies six unchanged `flatten/computed_ref_call_000/001`
originals and their `_f`/`_step` companions through exact normal forms and **20
bounded concrete applications**. Checked-core structure and text, pure CLI
normalization, and native IO execution are separate observations. The original
builds require Base; separate Base-enabled function adapters are unprintable.
Concrete pure/IO observation wrappers pass the available generated C/JS lanes.
No production behavior change was needed. [Report](cpu-computed-reference-calls.md).

The clean-source full gate passed **936 tests, zero failed, 49 ignored**. The
repeated matrix has **426 process observations**, including 12 direct upstream
Node IO host blocks and six strict-proof exclusions. All 1,302 checking statuses
are unchanged; 14 positive controls passed, and 18 observations preserve the
three unwaived hole/TODO disagreements. The interrupted initial gate is not a
candidate receipt. Historical source and receipt identities remain unchanged.

Detailed receipts now link **293 originals**, with **1,009** lacking detailed
receipts: **360 checker negatives, one checking-only module, 95 interpreter cases
and 553 compiled candidates**. Six interpretation rows are new; the 104 other
interpretation, 93 runtime and 90 checking rows retain historical identities.
These counts include partial/blocked cells and do not establish CPU parity.

The imported-constructor spelling disagreement and pinned JS printer timeout
remain separately open at their earlier tested source. Erased-hole/TODO,
host/resource/foreign/F32.read/runtime-error-order/API requirements and the
unfinished 688-vector exploration remain unwaived. Next independent batch:
six `stacked_computed_match_000/001` originals and `_nat`/`_not`/`_sel` companions,
with a proposed 39-vector grid for two/three computed columns. Source and
applicability were reviewed; none of that new grid was executed or qualified.
Dependencies, references, toolchains, guards, budgets, cancellation and approved
demand semantics remain frozen. No installations, pushes, desktop or GPU work.

## Stacked computed-match checkpoint

Source `63f4e30c7764a2cb72a78323e8aecc545dad4aa4` qualifies six unchanged `flatten/stacked_computed_match_000/001`
originals and `_nat`/`_not`/`_sel` companions through exact normal forms and **39
bounded applications**. The grid covers constructor depths 0..2 and every Boolean
combination for the three-column functions. Actual checked-core structure, CLI
text, concrete values, pure observation and native IO evidence remain separate.
No production behavior change was needed. [Report](cpu-stacked-computed-matches.md).

The clean-source full gate passed **939 tests, zero failed, 49 ignored**. The
repeated matrix contains **426 process observations**, including 12 direct Node
IO-oracle host blocks and six strict-proof exclusions. All 1,302 checking statuses
are unchanged. Fourteen positive controls passed; 18 observations preserve three
unwaived hole/TODO disagreements. Original missing-Base refusals and Base-enabled
function-printer refusals are exact, with no emitted artifact; neither supplies
an executable pass for the original function-valued main.

Detailed receipts now link **299 originals**, with **1,003** lacking detailed
receipts: **360 checker negatives, one checking-only module, 89 interpreter cases
and 553 compiled candidates**. Six interpretation rows are new; the previous
110 interpretation, 93 runtime and 90 checking rows retain their tested identities.
These counts include partial/blocked cells and do not establish CPU parity.
Imported-constructor printing and upstream JS timeouts remain separately open at
their earlier source. Erased-hole/TODO, host/resource/foreign/F32.read,
runtime-error-order, external API and unfinished 688-vector requirements remain
unwaived. No new exception or resource policy was adopted.

Next independent batch: eight `mixed_computed_column_000..002` originals and
companions, with a proposed 63-vector grid for parity recursion, value-column
threading and computed middle arguments. Only source/applicability review is
complete; no execution credit. Dependencies, references, toolchains, guards,
budgets, cancellation and approved demand semantics remain frozen. No installs,
pushes, desktop or GPU work.

## Mixed computed-column checkpoint

Source `b382a7683d2be3b06033cc4e6236595baed689f2` qualifies eight unchanged `flatten/mixed_computed_column_000..002`
originals and companions with **63 bounded applications**. The grid covers parity
recursion, value-column threading, full returned constructor depth, and all Bit
combinations for computed middle arguments. Normalized text, actual checked core,
concrete values, strict-proof applicability and native IO execution remain distinct.
No production defect was demonstrated or implementation change needed.
[Report](cpu-mixed-computed-columns.md).

The clean-source gate passed **942 tests, zero failed, 49 ignored**. The repeated
matrix records **568 process observations**, including 16 direct Node IO host
blocks and eight strict-proof exclusions. All 1,302 checking statuses are unchanged;
14 positive controls pass and 18 observations retain three unwaived hole/TODO
cases. Original missing-Base and separate function-printer refusals match exactly,
with no artifact; wrappers do not supply executable passes for original functions.

Detailed receipts now link **307 originals**; **995** lack detailed receipts:
**360 checker negatives, one checking-only module, 81 interpreter cases and 553
compiled candidates**. Eight interpretation rows are new. The prior 116
interpretation, 93 runtime and 90 checking rows keep their actual tested identities.
Linked receipts include partial/blocked cells and do not establish CPU parity.
Imported-constructor spelling and upstream JS timeout evidence remain separately
open at the earlier source. Erased-hole/TODO, host/resource/foreign/F32.read,
runtime-error-order, external API and unfinished 688-vector requirements remain
unwaived. No new semantic exception or resource policy was adopted.

Next independent batch: `mixed_computed_column_003`, `_id` and `_sel`, proposed
39 vectors. These import Base and are compiled candidates in the inventory;
actual function quantity/printability must be established, preserving the `+b`
reuse annotation. Only source/applicability review is complete, with no execution
credit. Dependencies, references, toolchains, guards, budgets, cancellation and
approved demand semantics remain frozen. No installs, pushes, desktop or GPU work.

## Mixed-column reuse checkpoint

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

## Binder-priority checkpoint

Source `e46434c5417252287478158e45260b3cbe6603eb` qualifies three unchanged `flatten` originals:
`shared_param_computed_000`, `cross_binder_promotion` and `var_row_column_rebind`.
The **36 bounded applications** cover predecessor matching, cross-binder shadowing
and catch-all priority with complete payload retention. Original normalization,
checked applications, pure projections and native IO execution are separate.
The first original's old sharing comment describes absent syntax; no sharing/Own
coverage is claimed. No production defect or implementation change was needed.
[Report](cpu-binder-priority.md).

Clean-source gate: **948 passed, zero failed, 49 ignored**. The repeated matrix
records **216 process observations**, including six direct Node IO host blocks
and three strict-proof exclusions. Original missing-Base refusals and separate
Base-enabled function-printer refusals match exactly, with no artifact. All 1,302
checking statuses are unchanged; 14 positive controls pass and 18 observations
preserve three unwaived hole/TODO differences.

Detailed receipts link **313 originals**; **989** lack detailed receipts:
**360 checker negatives, one checking-only module, 78 interpreter cases and 550
compiled candidates**. Three interpretation rows are new; the earlier 127
interpretation, 93 runtime and 90 checking rows retain historical identities.
Linked receipts include partial/blocked cells and do not establish CPU parity.
Imported-constructor printing/upstream JS timeouts, erased-hole/TODO,
host/resource/foreign/F32.read/runtime-error-order/API requirements and the
unfinished 688-vector exploration remain unwaived. Frozen dependencies,
references, toolchains, guards, budgets, cancellation and approved demand policy;
no installations, pushes, desktop or GPU work.

Next independent batch: the five `wide_scrutinee_000..004` originals. Source
review proposes 96 vectors using every combination of depths 0 and 2 across
four or five columns. This would exercise nonadjacent discriminators, ignored columns,
default reconstruction and complete payload forwarding. Actual printability
must be probed first; only source/inventory review is complete, with no execution
credit assigned. The grid is a bounded branch/payload sample, not arbitrary-depth
or exhaustive semantic coverage.

## Wide-scrutinee checkpoint

Source `93d10bf440096736fd60a977032c3869d8ebc9c6` qualifies five unchanged `flatten/wide_scrutinee_000..004`
originals with **96 bounded applications** across four or five columns. The
sample varies every argument independently over constructor depths 0 and 2,
preserving nonadjacent discriminators, ignored columns, default reconstruction
and complete ordered payloads. Function normalization, checked applications,
pure projections and native IO evidence remain distinct. No production defect
or implementation change was needed. [Report](cpu-wide-scrutinee.md).

Clean-source gate: **951 passed, zero failed, 49 ignored**. The repeated matrix
has **360 process observations**, including ten direct Node IO host blocks and
five strict-proof exclusions. Original missing-Base refusals and Base-enabled
function-printer refusals match exactly, without artifacts. All 1,302 checking
statuses are unchanged; 14 positive controls pass and 18 observations retain
three unwaived hole/TODO differences.

Detailed receipts link **318 originals**; **984** lack detailed receipts:
**360 checker negatives, one checking-only module, 73 interpreter cases and 550
compiled candidates**. Five interpretation rows are new; prior 130 interpretation,
93 runtime and 90 checking rows retain their historical identities. Linked
receipts include partial/blocked cells and do not establish CPU parity.
Imported-constructor printing/upstream JS timeouts, erased-hole/TODO,
host/resource/foreign/F32.read/runtime-error-order/API requirements and the
unfinished 688-vector exploration remain unwaived. Dependencies, references,
toolchains, guards, budgets, cancellation and approved demand policy remain
frozen. No installations, pushes, desktop or GPU work.

Next independent batch: five `list_ctor_000..004` originals, with a proposed
78-vector sample. The first four use all lists of length 0..2 over Bit F/T or
Nat depths 0/2; `_004` uses both Nat depths and five selected list shapes in each
list argument (50 combinations). This would exercise parameterized constructors,
nested head/tail defaults and complete ordered list payloads. Actual printability
and type/kind applicability must be established first. Only source/inventory
review is complete; no execution credit is assigned to this next batch.

## List-constructor and scope-printer checkpoint

Source `a53eac657f82b3fa75efc3e46263f13a9be4274b` qualifies five unchanged `flatten/list_ctor_000..004`
originals with **78 bounded applications**. Type/kind probes preserve the erased
quantity and element parameters, dependent `Kind(a)` and `&2` instantiation.
Complete constructor values, ordered list payloads, normalized text/graphs and
native IO execution remain separate. A demonstrated printer defect is fixed:
shadowed bound variables retain upstream's scope-level suffix, including `b^1`.
No parser, evaluator, core identity or sharing behavior changed.
[Report](cpu-list-constructors.md).

Clean-source gate: **957 passed, zero failed, 49 ignored**. The repeated matrix
contains **370 process observations**, including ten direct Node IO host blocks
and five strict-proof exclusions. Both original missing-Base and separate
function-printer refusals match exactly, with no artifact. All 1,302 checking
statuses are unchanged; 14 positive controls pass and 18 observations preserve
three unwaived hole/TODO differences. All 34 focused normalization/scope/list
tests pass at the tested source and run again in the final gate. Earlier compile,
lint and stale-printer-expectation failures remain separate failed receipts.

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

## Contextual scope and partial-normalization checkpoint

Source `0dec12b6b2261ea82a27da454dda3272b8128a12` corrects contextual diagnostic scope: expected/observed
terms distinguish outer `x^0` from shadowing `x`, while each context-row type sees
only its preceding binders. Two unchanged page originals and three authored
shadowing/import controls are covered; four earlier diagnostic controls are
renewed. No evaluator, sharing, term identity, acceptance, demand or budget change.
[Report](cpu-context-scope.md).

Gate: **959 passed, zero failed, 49 ignored**. Focused validation: 288 passed,
zero failed, one ignored. The repeated diagnostic matrix has **180 observations**:
162 exact failure contracts and 18 phase probes. All 1,302 checking statuses remain
unchanged; exactly one stderr changes, the intended pinned `context_dedup_shadow`
correction. Fourteen positive controls pass; three hole/TODO cases remain unwaived.

`eval/partial_app_snf` remains **unqualified**. Actual normalized graphs differ
(27 versus 166 nodes in the bounded comparison representation); a constructor-field
match minimizes the issue to `f(Box{x})` versus an unfolded stuck match. The
pinned weak-head contract retains the named call. Twenty primary graph observations
and 20 corroborating all-feature observations agree on that diagnosis. Ninety
closed-value observations pass across direct interpretation, port pure CLI and
generated C/JS, but do not close the open-normal-form mismatch. No evaluator fix
was attempted in this checkpoint. Next: preserve named-call reconstruction when
matching becomes blocked on a constructor field, with guard/accounting controls.

Detailed receipts link **325 originals**; **977** lack detailed receipts:
**358 checker negatives, one checking-only module, 68 interpreter cases and 550
compiled candidates**. Only two checking rows are new; prior 140 interpretation,
93 runtime and 90 checking rows retain historical identities. Linked rows include
partial/blocked evidence and do not establish CPU parity. Imported-printer/JS-timeout,
erased-hole/TODO, host/resource/foreign/F32.read/error-order/API gaps and the unfinished
688-vector exploration remain unwaived. Dependencies, references, toolchains,
guards, budgets, cancellation and approved demand policy remain frozen. No installs,
pushes, desktop or GPU work.

## Blocked named-call normalization checkpoint

Source `c123b3eaafe12f9ef40880aaa9922c22ec5465bc` corrects named-call reconstruction when a constructor-field
match blocks. The unchanged `eval/partial_app_snf` now agrees with pinned upstream
in exact normalized text and structure: **27 graph nodes on both sides**, replacing
the earlier port's 166-node residual match. Nine authored adjacent cases cover
first/nested/second fields, fuel, fallback, computed constructors, callee handoff,
partial applications and anonymous matches. No printer workaround, raised limit,
new semantic exception or public API is introduced.
[Report](cpu-blocked-named-calls.md).

Clean-source gate: **966 passed, zero failed, 49 ignored**. Forty repeated graph
observations agree. The 270-observation open/applicability matrix separates exact
normalization from type/phase probes and actual emitter refusals. Ninety closed
observations agree; persistent tests separately cover native execution and both
generated targets. The original's function-valued `Trm` still refuses executable
printing, exactly as upstream; wrapper execution is not an original runtime pass.

The 180-observation diagnostic renewal passes. All 1,302 checking statuses remain
unchanged, including rejection of all 439 expected negatives. Two shift proofs
now get beyond former budget exhaustion to the existing strict-proof `IO`
exclusion; 20 separate observations confirm pinned checking and upstream/port JS
emission succeed. No shift program execution is claimed. Fourteen positive
controls pass; three hole/TODO differences remain unwaived.

Detailed receipts link **326 originals**; **976** lack detailed receipts:
**358 checker negatives, one checking-only module, 68 interpreter cases and 549
compiled candidates**. Only one interpretation row is added; prior 140 interpretation,
93 runtime and 92 checking rows retain historical identities. Linked receipts
include partial/blocked cells and do not establish CPU parity. Imported-printer/
JS-timeout, erased-hole/TODO, host/resource/foreign/F32.read/error-order/API gaps
and the unfinished 688-vector exploration remain unwaived. Frozen dependencies,
references, toolchains, guards, budgets, cancellation and approved demand policy;
no installations, pushes, desktop or GPU work.

Next independent source-reviewed batch: `eval/eta_expand_print`,
`eval/toplevel_ref_print` and `eval/hoas_binder_refresh`, distinguishing a retained
bare reference from eta-expanded and shadowed-binder values. Actual printer
applicability and bounded applications still require qualification; no new
execution credit is assigned to those three sources.

## Function-valued normalization checkpoint

Source `19b35fa85c6e6a510df7863f0a6991c7c4bfc3f8` qualifies the unchanged `eval/eta_expand_print`,
`eval/toplevel_ref_print` and `eval/hoas_binder_refresh` originals. Exact normalized
structure distinguishes the eta-expanded lambda/application from bare `id`, while
the shadowed result refers to the inner binder. Seventeen bounded applications
preserve complete values. No production defect or implementation change was needed.
[Report](cpu-function-values.md).

Clean-source gate: **969 passed, zero failed, 49 ignored**; focused controls:
**285 passed, zero failed, one ignored**. The open matrix records 60 observations,
the graph matrix 16 (including renewed `partial_app_snf`), and the closed C/JS/
interpretation matrix 54. Native executable observations have separate persistent
tests. The function-valued originals refuse actual executable printing exactly;
wrapper execution is not an original runtime pass. Named-call reconstruction,
structural/work/depth guards, demand and cancellation remain under regression.

All 1,302 checking statuses and streams match the preceding port receipt; all
439 expected negatives reject. Fourteen positive controls pass; 18 observations
retain the three unwaived hole/TODO cases. Strict `IO` proof exclusions remain
separate from executable checking. The previous shift-program evidence remains
checking/emission only; those programs were not executed in this checkpoint.

Detailed receipts link **329 originals**; **973** lack detailed receipts:
**358 checker negatives, one checking-only module, 68 interpreter cases and 546
compiled candidates**. Three interpretation rows are added; earlier 141 interpretation,
93 runtime and 92 checking rows retain their actual identities. Linked rows include
partial/blocked cells and do not establish CPU parity. Imported-printer/JS-timeout,
erased-hole/TODO, host/resource/foreign/F32.read/error-order/API gaps and the
unfinished 688-vector exploration remain unwaived. Dependencies, references,
toolchains, budgets, guards and approved demand policy remain frozen. No installs,
pushes, desktop or GPU work.

Next source-reviewed batch: `eval/printer_fixpoint`, `eval/printer_fresh_binder`
and `eval/printer_fresh_binder_pick`, covering a constructor continuation and
fresh binders generated by nested patterns. The middle original uses IO, so its
strict-proof exclusion, actual host applicability and executable results must
remain distinct. Only source/inventory review is complete; no execution credit
is assigned to that proposed batch.

## Constructor-continuation and binder checkpoint

Source `0cabee21d28aa6e8fb96164b860249db29510210` qualifies the unchanged `eval/printer_fixpoint`,
`eval/printer_fresh_binder` and `eval/printer_fresh_binder_pick` originals under
separate applicable contracts. Exact graphs/text preserve the constructor's
continuation, generated `_4` binder and three distinct match-arm identities.
Fourteen bounded applications retain complete values. No production defect or
implementation change was needed. [Report](cpu-constructor-binders.md).

Clean-source gate: **973 passed, zero failed, 49 ignored**; focused controls:
**286 passed, zero failed, one ignored**. The open/applicability matrix has 52
observations. Twelve paired graph observations include renewed `partial_app_snf`;
the IO helper's execution-context graph comparison is separately covered by the
private unit test, without exposing a proof API. The 72-observation runtime matrix
contains 46 successful exact outputs, 24 generation/build successes and two known
direct-Node IO host blocks. The unchanged IO original prints `2` in native and
upstream/port generated C and JS. The two function-containing originals refuse
executable printing exactly; their wrappers do not fabricate original runtime passes.

All 1,302 checking statuses and streams match the preceding port receipt; all
439 expected negatives reject. Fourteen positive controls pass; 18 observations
retain three unwaived hole/TODO cases. Strict IO checking remains excluded,
separate from successful executable checking. No shift programs are executed.

Detailed receipts link **332 originals**; **970** lack detailed receipts:
**358 checker negatives, one checking-only module, 68 interpreter cases and 543
compiled candidates**. Two interpretation rows and one runtime row are new;
earlier 144 interpretation, 93 runtime and 92 checking rows keep their historical
identities. Linked receipts include partial/blocked cells and do not establish
CPU parity. Imported-printer/JS-timeout, erased-hole/TODO, host/resource/foreign/
F32.read/error-order/API gaps and the unfinished 688-vector exploration remain
unwaived. Frozen dependencies, references, toolchains, guards, budgets,
cancellation and approved demand policy; no installs, pushes, desktop or GPU work.

Next source-reviewed batch: `eval/underscore_print`, `eval/wildcard_chain_print`
and `eval/efq_arm_value`. These distinguish fresh wildcard binders and a selected
empty-match value from a genuinely blocked named call. No inhabitant of `Empty`
may be fabricated to claim a closed runtime test. Only source/inventory review
is complete; actual applicability and results remain unqualified.

## Wildcard and empty-match value checkpoint

Source `49aeac60e8794b0f7f0644cbcc0450a5a355054a` qualifies the unchanged `eval/underscore_print`,
`eval/wildcard_chain_print` and `eval/efq_arm_value` originals. Exact normalized
graphs/text preserve distinct unused wildcard binders, a selected empty-match
function and the separately blocked named call. Twelve complete wildcard
applications, three partial applications and two Boolean-to-function selections
have separate evidence. No inhabitant of `Empty` is fabricated; the empty-match
function is never applied. No production or public API change was needed.
[Report](cpu-wildcard-values.md).

Clean-source gate: **977 passed, zero failed, 49 ignored**; focused controls:
**289 passed, zero failed, one ignored**. The open/applicability matrix records
62 observations; 16 paired graphs include renewed `partial_app_snf`; the closed
matrix records 36 observations for the two wildcard wrappers only. All original
executable printers refuse function-containing results exactly. Native executable
wrapper observations are separately covered by the persistent test; pure CLI
normalization is not relabeled as native executable evidence.

All 1,302 checking statuses and streams match the preceding port receipt; all
439 expected negatives reject. Fourteen positive controls pass; 18 observations
retain three unwaived hole/TODO cases. Private-effect wildcard refusal, source-hole
refusal, sharing, demand order, cancellation and resource guards remain under
regression. The new differential matrices contain no IO or shift programs;
earlier oracle receipts retain their actual source and lane identities.

Detailed receipts link **335 originals**; **967** lack detailed receipts:
**358 checker negatives, one checking-only module, 68 interpreter cases and 540
compiled candidates**. Three interpretation rows are added; earlier 146
interpretation, 94 runtime and 92 checking rows retain their identities. Linked
rows include partial/blocked cells and do not establish full CPU parity.
Imported-printer/JS-timeout, erased-hole/TODO, host/resource/foreign/F32.read/
error-order/API gaps and the unfinished 688-vector exploration remain unwaived.
Dependencies, references, toolchains, guards, budgets, cancellation and approved
demand policy remain frozen; no installs, desktop or GPU work. Publication is
recorded separately after explicit approval.

Next source-reviewed batch: `eval/open_closure`, `eval/open_closure_pipe` and
`eval/stuck_split_mint`. The first has an IO main; the others exercise open
higher-order bindings and blocked computed calls. Only source/inventory review
is complete; applicability and execution remain unqualified.

## Open closure and function-domain checkpoint

Source `cbfe84fb6f76c17fcb3022fc86d3e62f01a3cafa` qualifies the unchanged `eval/open_closure`,
`eval/open_closure_pipe` and `eval/stuck_split_mint` originals under separate
normalization, printer and IO contracts. Function-domain diagnostics now retain
the pinned parentheses. Graphs/text and binder identities agree; 54 individual
composition calls and five recursive Nat applications preserve complete results.
The IO original prints `12` in native and upstream/port generated C and JS.
Its direct upstream Node lane remains host-blocked and strict proof checking
still excludes IO. [Report](cpu-open-closures.md).

Full gate: **983 passed, zero failed, 49 ignored**; focused controls:
**300 passed, zero failed, one ignored**. The 54-observation open matrix and
12-observation graph matrix pass, including renewed `partial_app_snf`. The
72-observation execution collection has **42 exact outputs, 24 generation/build
successes, four aggregate work-budget failures and two direct-Node host blocks**.
It exits 1; it is not a passing full runtime matrix. Both unchanged authored
27-result composition wrappers exhaust the single pure-normalization budget,
although individual calls and generated C/JS aggregates agree. This new required
resource gap remains open; no evaluator switch, larger budget or policy exception
is used to qualify it.

All 1,302 checking statuses are unchanged; all 439 expected negatives reject.
Only `check/capture_soundness.bend` changed its checking stream. It now matches
exact repeated upstream status/streams and rejects during checking. Phase-only
probes remain separate. Fourteen positive controls pass;
18 observations retain three unwaived hole/TODO differences. Existing source-hole,
sharing, named-call, demand, cancellation and work/depth/structural controls pass.
The initial test-comparator expansion broke two refusal controls; it was reverted
and a type-only comparison was kept local. The failed receipt remains historical.

Detailed receipts link **339 originals**; **963** lack detailed receipts:
**357 checker negatives, one checking-only module, 68 interpreter cases and 537
compiled candidates**. Two interpretation rows, one original-IO runtime row and
one exact checking row are added; earlier 149 interpretation, 94 runtime and
92 checking rows retain source identities. Linked evidence includes partial/blocked cells, not universal
CPU parity. Aggregate work, imported-printer/JS-timeout, erased-hole/TODO,
host/resource/foreign/F32.read/error-order/API gaps and the incomplete 688-vector
exploration remain unwaived.

Next bounded work: account for the observed **23-result pass / 24-result budget
failure** boundary, separating genuine accumulated work from avoidable repeated
reduction. Preserve the exact 27-result reproducers, individual and generated
results, fixed budgets and guards. The printer correction is independent of this
resource investigation. Frozen dependencies, toolchains and reference pins;
no installations, desktop, GPU work or new pushes. Previously published branch
heads remain at the earlier approved checkpoints.

## 2026-10-24 composition work-sharing checkpoint

Source `20fec64bebb656c3fea84896e177f6cda8a9e75d` closes the demonstrated 23-pass/24-fail composition
work gap. Both unchanged authored 27-result wrappers now match pinned upstream
in the pure CLI and generated C/JS lanes. Substitution preserves unaffected
application sharing; one normalization call memoizes only demanded constructor
heads, retaining raw fields and the existing guards and work charges.
[Diagnosis, commands and receipts](cpu-composition-work.md).

The final full gate passes **990 tests, zero failures, 49 ignored**. The renewed
72-observation runtime matrix contains **46 exact output matches, 24 successful
generation/build steps and two unchanged upstream direct-Node IO host blocks**.
The 12 prefix and 12 graph observations pass. All 1,302 checking statuses and
streams are unchanged from the preceding checkpoint; all 439 expected negatives
reject. Fourteen positive controls pass and 18 observations preserve the three
unwaived hole/TODO disagreements. Original failures and earlier receipts retain
their original source identities.

Coverage remains **339 linked originals and 963 without detailed receipts**:
**357 checker negatives, one checking-only module, 68 interpreter cases and 537
compiled candidates**. This repairs an existing required cell rather than adding
new inventory coverage. Links include partial, blocked and historical evidence;
full CPU parity remains incomplete. Imported-constructor printing/JS timeout,
erased-hole/TODO, other resource/host/foreign gaps, F32.read, competing failure
order, downstream API caveats and the incomplete 688-vector exploration remain
unwaived. No new semantic exception is introduced.

Next independent batch: `flatten/ctor_construct_000`, `_001`, `ctor_continuation`,
`ctor_continuation_go` and `ctor_default`. Establish actual printer/type
applicability, then compare graphs and bounded constructor applications. The
local `Io` datatype in the continuation fixtures is not evidence of executing
Base IO. These cases remain unqualified. Frozen inputs and local-only commits;
no installations, new pushes, desktop or GPU work.

## Constructor/continuation checkpoint

Tested source `7209617167158e9fd6338e59f7ac73f505e4b88b` adds five unchanged
constructor/continuation originals and seven regression tests, with no production
change. An independent source review found no demonstrated blocking defect in
the preceding sharing repair; the two recommended cache-hit/captured-binder and
live weak-key ownership controls now pass. The exact 27-result aggregate stays
under its unchanged budget. [Scope, commands and receipts](cpu-constructor-continuations.md).

The full gate passes **997 tests, zero failures, 49 ignored**. Five originals
match normalized graphs and text; 25 direct original applications, six nested
continuation applications and 36 closed projected observations are distinct
coverage. All 20 graph observations, 150 open/applicability observations and
90 closed-runtime observations complete. The latter contain **60 exact
status/stdout/stderr matches and 30 successful generation/build steps**, using
upstream and port C O3 and generated JS separately. Original no-Base refusals
and adapted function/Type printer refusals are exact; they are not executable
passes. Local `Io` constructors do not execute Base IO. All 1,302 checking
statuses and streams are unchanged; 439 expected negatives reject. Fourteen
positive controls pass; 18 observations preserve three hole/TODO disagreements.

Detailed evidence now links **344 originals**, including five new interpretation
rows; **958** remain without detailed receipts: **357 checker negatives, one
checking-only module, 63 interpreter cases and 537 compiled candidates**. The
156 interpretation, 95 runtime and 93 checking links include historical,
partial and blocked cells. They are not 344 fully qualified programs and do not
establish CPU parity. Existing imported-constructor printing/JS timeout,
erased-hole/TODO, resource/host/foreign gaps, F32.read, competing failure order,
downstream API caveats and the incomplete 688-vector exploration remain unwaived.

Next bounded work: renew the minimal imported-constructor printer reproducer
from [ragged rows](cpu-ragged-rows.md#open-imported-constructor-printer-discrepancy).
Historically the interpreter, upstream C and port pure CLI print `helper.Box{7}`,
while port generated C/JS print `Box{7}` and upstream JS times out. This batch
uses scalar projections and does not qualify that constructor spelling or
waive its timeout. Preserve those controls before any targeted correction.
Frozen dependencies, toolchains, pins, budgets, guards and cancellation remain;
no installations, pushes, merges, desktop or GPU work.

## Imported constructor display checkpoint

Tested source `936d299d3f6db320b7ae0fcc82c33bfa934bcbbf` corrects the demonstrated port
imported-constructor display gap. C and JS pure output now use canonical
qualified names while retaining local runtime/foreign tags and constructor IDs.
The independent source review found no blocking defect. Imported `Con`, `Nil`
and `Tuple` also retain their qualified spelling instead of receiving Base sugar.
[Diagnosis, commands and receipts](cpu-imported-printer.md).

The full gate passes **998 tests, zero failures, 49 ignored**; focused controls
pass **268 / 0 / 1**. Eight authored printer cases produce **84 exact execution
matches and 48 successful generation/build steps**. The **12 upstream-JS timeout
observations remain unresolved**; the complete printer matrix exits 1. This is
an upstream descriptor/tag search defect reproduced under Node, not a missing
host dependency or an approved exception. The minimal historical source and
old failed receipts are preserved. All port observations match pinned interpreter
and C O3 output; generated-JS oracle agreement is not claimed for imported cases.

The renewed aggregate matrix has **46 exact outputs, 24 successful builds/
generation steps and two unchanged direct-upstream-Node IO host blocks**. The
unchanged 27-result aggregate still costs 1,338,629 steps, below the same
2,000,000-step/128-depth guards. All 1,302 checking statuses and streams remain
unchanged; 439 expected negatives reject. Fourteen positive controls pass;
18 observations preserve three unwaived hole/TODO disagreements. No
mid-normalization cancellation or general resource-equivalence claim is added.

Coverage remains **344 originals linked / 958 without detailed receipts**:
**357 checker negatives, one checking-only module, 63 interpreter cases and 537
compiled candidates**. Authored printer reproducers do not increase original
inventory coverage. Links include historical, partial and blocked evidence.
Upstream imported-JS printing, erased-hole/TODO, resource/host/foreign gaps,
F32.read, competing failure order, downstream API caveats and the unfinished
688-vector exploration remain unwaived. CPU parity is incomplete.

Next independent batch: `flatten/simple_ctor_000`, `_001`,
`single_ctor_type_001`, `_mk` and `nested_literal`, first establishing actual
printer/type applicability, then normalized graphs/text and a bounded 33-value
application grid. These are ledger-selected interpreter cases; no new execution
credit is claimed. Dependencies, pins, toolchains, guards, budgets and security
policies remain frozen. Local checkpoints only; no publication, merge,
installation, desktop or GPU work. No new user decision was needed for this fix.

## Simple/single/nested constructor checkpoint

Tested source `0b041df0fa8de2a50787026082ee59afa253070b` adds five unchanged simple,
single-constructor and nested-constructor originals with a bounded **33-input**
grid. Original graphs/text, complete application results, scalar projections
and executable output are compared separately. No production change or new
port semantic mismatch was found. [Scope and receipts](cpu-constructor-matches.md).

The full gate passes **1,003 tests, zero failures, 49 ignored**; final focused
controls pass **23 / 0 / 0**. The **60 graph observations** and **150 original/
applicability observations** complete. Ten wrapper mains have **110 exact
executions and 60 successful generation/build steps**. The **ten upstream-JS
full-constructor printer timeouts remain unresolved**, so the complete runtime
matrix exits 1. All projected-output lanes agree. Port native/C/JS full values
match the pinned interpreter and C O3; this is not upstream-JS output parity.
Original Base-required and adapted function-printer refusals are exact and do
not become executable passes.

The initial new test aborted while recursively decoding its large graph oracle,
before Bend evaluation. The independently reviewed test-only repair compares
the entire canonical serialized graph, with no dropped nodes or structural
relaxation. Failed receipts remain tied to `5a210c3`; the clean final gate uses
this tested source. No host stack, Bend work/depth guard, evaluator or policy
change occurred. Aggregate sharing/work/demand/cancellation regressions pass in
the full gate; no mid-normalization cancellation is claimed. All 1,302 checking
statuses and streams remain unchanged; 439 expected negatives reject. Fourteen
positive controls pass; 18 observations retain three hole/TODO disagreements.

Detailed evidence now links **349 originals**, with **953** without detailed
receipts: **357 checker negatives, one checking-only module, 58 interpreter
cases and 537 compiled candidates**. Five interpretation rows are new; prior
rows retain their historical identities. Links include partial/blocked evidence,
not completed parity. Imported-JS printing, erased-hole/TODO, resource/host/
foreign gaps, F32.read, competing failure order, downstream API caveats and the
unfinished 688-vector exploration remain unwaived.

Next independent family: `flatten/literal_default_000..004`, covering literal
ladders, three-column defaults and reconstruction of the whole matched default
value. Establish applicability before a bounded **45-input** grid. These remain
unqualified. Frozen dependencies, toolchains, pins, budgets, guards and
cancellation; local checkpoints only, no pushes/merges/installations/desktop/GPU.
No new user decision was needed for this bounded qualification.

## Literal/default constructor checkpoint

Tested source `eeb7b2814ae9fa21cf68b8c0c06cf236c419b75b` adds five unchanged `flatten/literal_default_000..004`
originals and a bounded **45-input** grid. Original function graphs/text, direct
applications, full imported constructor values and scalar projections remain
separate. No production divergence was found. [Scope and receipts](cpu-literal-defaults.md).

Full gate: **1,009 passed, zero failures, 49 ignored**. Clean focused controls:
**24 / 0 / 0**. Sixty structural and 150 original/applicability observations
complete. Runtime observations comprise **112 exact executions, 60 successful
build/generation steps and eight unresolved upstream-JS printer timeouts**.
The complete runtime runner exits 1; full values match native Rust, port C/JS,
pinned interpretation and upstream C O3, without claiming upstream-JS parity.

A new test initially overflowed during recursive graph JSON encoding, after
Bend evaluation and Shape construction succeeded. A reviewed test-only explicit
stack now preserves every constructor name/ordered child and rejects other
nodes. Failed receipts remain tied to `cf2e86a`; no evaluator, stack-size,
guard, budget or cancellation change occurred. Current aggregate controls pass;
no mid-normalization cancellation is claimed. All 1,302 checking statuses and
streams remain unchanged, with 439 expected negatives rejecting. Fourteen
positive controls pass; 18 observations retain three hole/TODO disagreements.

Detailed receipts link **354 originals**, leaving **948 unlinked**: **357 checker
negatives, one checking-only module, 53 interpreter cases and 537 compiled
candidates**. Links include historical, partial and blocked observations, not
completed parity. Imported-JS printing, erased-hole/TODO, resource/host/foreign
gaps, F32.read, competing failure order, downstream API caveats and the unfinished
688-vector exploration remain unwaived.

Next proposed batch: `flatten/dead_residue_elision_000`, `_001`, `_002` and `_004`.
Establish original function/concrete-value applicability first, then **29**
applications of `main` or `f` as appropriate. No next-batch execution credit.
Dependencies, toolchains, upstream references, budgets, guards and cancellation
remain frozen; no installs, pushes, merges, desktop or GPU work. No new decision
is required by this checkpoint.

## Dead-residue and standalone-build checkpoint

Tested source `dcf26ff64a9c093cd5a5b123b3b5c6df599e5bc5` adds four unchanged `flatten/dead_residue_elision`
originals (`_000`, `_001`, `_002`, `_004`) and **29 bounded applications**.
Applicability exposed nine incorrect standalone-build acceptances: the three
concrete originals compiled in port C/JS/native despite lacking the exact
top-level `IO` declaration required by pinned upstream. A shared, independently
reviewed preflight now rejects them before lowering, with exact status/stderr
and artifact preservation. Pure interpretation, strict emitters, evaluation,
sharing and safety limits are unchanged. [Details and receipts](cpu-dead-residue.md).

Full gate: **1,018 passed, zero failures, 49 ignored**. Focused controls:
**83 / 0 / 0**. All **72 complete graph** and **132 normalization/applicability**
observations complete. The runtime matrix records **120 exact executions,
66 successful generation/build steps and 12 unresolved upstream-JS printer
timeouts**. All port-required cases pass; the complete matrix exits 1. Full
values agree in native Rust, port C/JS, pinned interpretation and upstream C O3;
scalar projection success does not qualify the failed upstream-JS full printers.

Aggregate sharing/work/demand/cancellation regressions pass in the full gate;
no mid-normalization cancellation is claimed. All 1,302 checking statuses and
streams remain unchanged, including 439 rejecting negatives. Fourteen positive
controls pass; 18 observations retain three hole/TODO disagreements. Minimal
ordinary-IO scaffold C compiler failures and a foreign-source checking diagnostic
difference remain separately unqualified. External emitter consumers can now
receive Err for no-IO builds; repository callers handle status 1, but external
consumers have not been audited.

Detailed receipts link **358 originals**, leaving **944 unlinked**: **357 checker
negatives, one checking-only module, 49 interpreter cases and 537 compiled
candidates**. These are applicability categories, not missing runtime-test
counts. Links include historical, partial and blocked evidence, not completed
parity. Imported-JS printing, erased-hole/TODO, resource/host/foreign gaps,
F32.read, competing failure order, downstream API caveats and the unfinished
688-vector exploration remain unwaived.

Next proposed independent batch: `flatten/nested_computed_match_001`, `_dec` and
`_zro`; establish actual printability, then **12** bounded applications. This is
source/ledger selection only, without execution credit. Dependencies, toolchains,
references, budgets, guards and cancellation remain frozen. Local checkpoints
only; no installs, pushes, merges, desktop or GPU work. No user decision is
required by this correction.

## Nested computed matching checkpoint

Tested source `cc3818eaa394ab3be74d9edd14b00e7763c11da1` adds three unchanged
`flatten/nested_computed_match_001`, `_dec` and `_zro` originals and **12 bounded
applications**. Original function graphs/text preserve `dec(zro(p))`; closed
constructor results, scalar projections and execution are distinct comparisons.
No production defect or code change was found. [Scope and receipts](cpu-nested-computed.md).

Full gate: **1,024 passed, zero failures, 49 ignored**. Focused controls:
**33 / 0 / 0**. All **36 complete graph** and **90 normalization/applicability**
observations pass, including 60 exact standalone-build refusals without artifacts.
Six authored executable mains produce **72 exact repeated executions and 36
successful generation/build checks**, with zero matrix timeouts. Their result
constructors are nullary; this success does not resolve the historical
field-bearing imported-constructor upstream-JS printer failures.

Aggregate sharing/work/native/C/JS/demand/cancellation regressions pass in the
full gate; no mid-normalization cancellation is claimed. All 1,302 checking
statuses and streams remain unchanged, including 439 rejecting negatives.
Fourteen positive controls pass; 18 observations retain three hole/TODO
disagreements. The IO emitter correction's API caveat remains: no-IO executable
compilation may now return Err for external consumers, which have not been
audited. Minimal-IO upstream C compiler failures and foreign checking diagnostic
differences remain separately unqualified; prior receipts are not renewed by
matching cases in this batch.

Detailed evidence links **361 originals**; **941 remain unlinked**: **357 checker
negatives, one checking-only module, 46 interpreter cases and 537 compiled
candidates**. Links include partial, blocked and historical evidence, not
completed parity. F32.read, competing failure order, resource/host/foreign gaps,
erased-hole/TODO, downstream API caveats and the incomplete 688-vector exploration
remain unwaived. Missing API/integration requirements still require review.

Next proposed coherent family: seven `flatten/annotated_computed_scrutinee_000`
originals/companions, actual applicability first, then **77** bounded applications
covering list equality/length, annotation, selection and Boolean helpers.
This is source/ledger selection only; no next-batch execution credit.
Dependencies, pins, toolchains, guards, budgets and cancellation stay frozen.
Local checkpoints only; no installs, publication, merge, desktop or GPU work.

## Annotated computed scrutinee checkpoint

Tested source `39634e208333393e6cf0b47502eb4669ba2fd590` adds seven unchanged
`flatten/annotated_computed_scrutinee_000` originals/companions and **77 bounded
applications** covering annotated list length/selection, all Boolean cases and
all 49 ordered pairs of seven lists. Original graphs/text, complete constructors,
scalar projections and executable output remain separate. No production change
was needed. [Scope and receipts](cpu-annotated-scrutinee.md).

Full gate: **1,030 passed, zero failures, 49 ignored**. Focused controls:
**35 / 0 / 0**. All **84 complete graph** and **210 normalization/applicability**
observations complete, including 140 exact build refusals without artifacts.
The runtime matrix records **162 exact executions, 84 successful generation/
build checks and 6 unresolved upstream-JS printer timeouts**; runner exit
**1**. All required port observations pass. Projection or nullary-result
success does not qualify the field-bearing full-value upstream-JS printers.

Independent review found no blocking concerns or weakened comparisons. The source
annotation is checked; its elimination from normal form agrees with upstream.
Aggregate sharing/work/native/C/JS/demand/cancellation controls pass, with no
mid-normalization cancellation claim. All 1,302 checking statuses/streams remain
unchanged, including 439 rejecting negatives. Fourteen positive controls pass;
18 observations preserve three hole/TODO disagreements. External consumers of
the IO emitter correction remain unaudited: no-IO compilation may return Err
where it previously produced source. Earlier minimal-IO upstream C failures,
foreign diagnostics, resource/host gaps, F32.read, competing failure order and
the incomplete 688-vector exploration remain unwaived.

Detailed receipts link **368 originals**; **934 remain unlinked**: **357 checker
negatives, one checking-only module, 39 interpreter cases and 537 compiled
candidates**. Links include historical, partial and blocked results; they do not
establish complete CPU parity or close the separate API/integration inventory.
Next proposed batch: four `flatten/statement_sequence` originals/companions,
applicability first, then eight closed function inputs and two nullary proof
observations. No execution credit is assigned to that proposal. Dependencies,
toolchains, pins, budgets, guards and cancellation remain frozen. Reviewed
feature-branch preservation is now authorized; no main merge or release. Private
raw receipts, binaries and generated failures remain outside Git and must be
retained separately. No installs, desktop or GPU work.

## Data-grade and diagnostic-origin checkpoint

Final gate/test source `a73825e91bdc8b5184b7bcc04597773c2757c503`; matrix engine
`96f4d4759a91c8ba81c04550a238aa476269fcd4`. Their only difference is an equivalent test assertion.
This batch qualifies the unchanged
`grade/accept_data` through ten original definition graphs, 26 direct applications,
eight consuming observations and two full aggregates. Proof/function graphs retain
all children. Empty elimination has no fabricated runtime inhabitant.
[Scope, commands, compatibility and receipts](cpu-data-grades.md).

Quantity failures now report the pinned outer-context binder and measured use,
with source spans for generated lambdas/lets and multiline datatype/arrow heads.
The audit also exposed an older stale source-address defect: weak origin roots
now reserve diagnostic identity across transformation/sweeping without retaining
terms or children. The unchanged `proof/goal_ladder_state_027` now reports the
actual Type at line 234. Plain/tracked checker decisions and work/depth/fresh-ID
accounting agree; no evaluation, demand, sharing, budget or cancellation change.
Independent review found no blocking issue. There is no mid-normalization
cancellation claim.

Full gate **1,104 / 0 / 49**; focused **46 / 0 / 0**. The **728-observation**
matrix completes with no timeout: 184 full-graph observations, 184 exact text
observations, 52 original calls, 280 exact rejection observations, 18 strict-pure
native/C/JS executions and ten build/generation controls. All 359 repeated groups
are deterministic. Upstream executable generation requires Base; both Base import
orders reject in both implementations. Upstream executable runs therefore remain
unrun. Port strict-pure successes are compared to the pinned interpreter, not
claimed as upstream executable C/JS parity.

All 1,302 checking statuses remain unchanged. Of 58 changed diagnostic streams,
48 match upstream exactly. Ten cells remain unresolved: six constructor-label
differences, three pattern-field source anchors, and the existing `unsafe_many`
acceptance disagreement. Fourteen positive controls pass; eighteen observations
retain three erased-hole/TODO differences. The supplementary all-target Clippy run at the matrix engine source failed
on seven observed pre-existing test-only lints outside the standard gate; it
is fail-fast, not an exhaustive lint inventory. The two new test targets pass
focused strict Clippy. Historical gates
at `47deb3b` are retained separately, not relabeled as current validation.

The prior public `DefDecl::completion_parameters` field still requires external
Rust struct-literal updates. Repository callers were inspected; none needs an
additional update here. External consumers remain unaudited. Existing CLI exit
status and executable-emitter Err compatibility caveats remain explicit. Prior
imported-JS printer timeouts, dependent-pair undefined-CID C failure/two unrun
executions, minimal-IO C, F32.read, failure-order, host/foreign/resource/API gaps
and the incomplete 688-vector exploration remain unwaived.

Detailed links cover **392 originals**, with **910 unlinked**: **356 checker
negatives, one checking-only module, 16 interpreter cases and 537 compiled
candidates**. These are applicability categories, not missing runtime-test counts.
Historical/partial/blocked receipts do not establish CPU parity. Acceptance still
requires closing or explicitly reviewing applicable unresolved contracts; no new
exception is introduced.

Next independent substantive candidate is `reg/framed_cell_dep`: actual
proof-function/IO applicability, four bounded worker inputs (0, 1, 3, 8), unchanged
IO output and false-evidence rejection. No next-batch execution credit is assigned.
The nine remaining source/label diagnostic cells are also concrete bounded repair
candidates. Dependencies, toolchains, pin, safety limits and cancellation remain
frozen. Disk preflight requires 11 GiB; only completed inactive rebuildable test
caches are reclaimed. Source, raw receipts, historical failures and CLI snapshots
remain intact; ignored raw evidence is outside Git preservation. Reviewed feature
checkpoint publication is authorized, with no main merge or release.

## Framed dependent worker checkpoint

Tested source `d1adf2ae5291faa59a30f605982e4b0d069d05f9` qualifies unchanged
`reg/framed_cell_dep`: two original worker graphs and four inputs (0, 1, 3, 8)
through worker and caller. Every closed Nat graph child is checked. The erased
evidence binder preserves its reference to the earlier Nat parameter. Two
negative adapters reject misplaced dependent evidence and false reflexivity.
No production correction was required. [Scope and exact receipts](cpu-framed-cell.md).

The original IO main prints `3` with status 0/empty stderr in native Rust and
both implementations' generated C/JS. Actual function printers refuse exactly;
strict proof checking/evaluation excludes imported IO. These are distinct
contracts. Upstream direct IO execution under Node still fails with `require`
unavailable; generated JS success does not close that host lane. Function alias
graph fixtures remain upstream-only observations; the original and eight closed
port graphs are asserted by executable-core unit tests, not a public proof API.

Full gate **1,109 / 0 / 49**; focused **51 / 0 / 0**; focused strict Clippy passes.
The matrix records **298 observations**, including 106 exact runtime/interpreter
matches, 54 successful generation/build checks, 48 exact diagnostic/printer
refusals and 52 strict-proof IO exclusions. There are no timeouts or new port
mismatches. Two direct-host failures remain unresolved; eight upstream function
execution cells are unrun after verified printer refusal. All repeated process
groups are deterministic. Independent review found no blocking concern.

Forty renewed observations preserve all ten unresolved diagnostic/acceptance
cells exactly, including `unsafe_many`. Fourteen positive controls pass; eighteen
observations preserve three hole/TODO differences. The previous supplementary
all-target Clippy failure is historical, unwaived and not rerun here; standard
and focused lint success do not close it. The 1,302-status audit is also retained
as historical, not mislabeled as a fresh census.

Detailed links cover **393 originals**, with **909 unlinked**: **356 checker
negatives, one checking-only module, 15 interpreter-profile cases and 537 compiled
candidates**. Static source profiles are hints; this original now has demonstrated
C/JS applicability despite its earlier interpreter-only classification. Links
include partial/blocked/historical evidence and do not establish full CPU parity.
Previous imported-JS, dependent-pair/minimal-IO C, F32.read, error-order,
host/foreign/resource/API gaps and the incomplete 688-vector exploration remain
unwaived. External DefDecl/emitter consumers remain unaudited. There is still no
mid-normalization cancellation claim.

Next proposed batch: `io/main_forged_io` and `import/base_prelude`, applicability
first, to distinguish actual Base IO identity from a user family named IO and
prelude-free constructor evaluation. No execution credit is assigned. Inputs,
dependencies, toolchains, limits and cancellation remain frozen. Inactive test
caches alone are reclaimable; unique evidence is retained. Raw receipts and CLI
snapshots remain outside Git preservation. Reviewed feature checkpoint publication
is authorized, without a main merge or release.

## Base identity and undefined-reference checkpoint

Tested source `81ec997d1e3c93c24a525548ff21396297390f27` qualifies unchanged
`io/main_forged_io` and `import/base_prelude`, with four bounded depth inputs,
full original graphs/text, strict-pure constructor results and actual executable
applicability. A Base-import adapter confirms user `IO` identity stays pure.
[Scope, commands, hashes and failures](cpu-base-identity.md).

Undefined references now attach source diagnostics at the existing failed lookup:
expected `a defined name`, observed reference/context/window. Exact negatives
cover direct/imported/type/erased positions and missing-head priority. Internal
error messages, acceptance, sharing, budgets and demand semantics remain intact;
exhausted presentation preserves the original failure. Independent review found
no blocker. The existing status-bearing `Ok(CliOutput)` path now handles more
undefined-reference failures; repository entrypoints consume `exit_code`, but
external callers remain unaudited. Previous emitter/DefDecl caveats remain.

Full gate **1,118 / 0 / 49**, including aggregate sharing, demand, cancellation
and guard controls; new integration-target strict Clippy passes. The matrix has
246 observations with no timeout: exact available executions and negatives,
separate graph/text/check evidence, four IO proof exclusions, two repeated
upstream forged-IO C generation failures and two Node direct-IO host failures.
Failed C generation has no build/execution credit. No new semantic exception.

A fresh 1,302-status census preserves every prior outcome, including all 439
negative rejections. The 489 changed stderr streams compare at checking phase:
26 exact diagnostics, 385 strict IO exclusions, 78 unwaived differences (64
upstream accepts/port rejects and 14 differing rejections). Six of the latter
are generated-reference/source-label/substitution presentation gaps; eight are
strict IO/foreign exclusions preempting upstream errors. These phase probes are
not 489 newly qualified runtime fixtures. The ten older cells renew exactly in
40 observations. Fourteen positive and eighteen unfinished-hole controls retain
their previous outcomes. Historical supplementary all-target lint failures remain
unwaived, not rerun and not implied closed by standard/focused lint success.

Detailed links now cover **395 originals**, with **907 unlinked**: 356 checker
negatives, one checking-only module, 13 interpreter-profile cases and 537 compiled
candidates. Links include partial/blocked/historical receipts and do not establish
CPU parity. Existing imported-JS timeouts, dependent-pair/minimal-IO C failures,
F32.read, error order, host/foreign/resource/API gaps and the incomplete 688-vector
exploration remain unwaived. Mid-normalization cancellation is not claimed.

Next bounded independent correction: generated `Exists`, `Empty` and `Box.bind`
source provenance across four original negatives. Next substantive candidate:
`parse/datatype_declaration_form`, `empty_datatype`, `empty_param_group`, actual
applicability first. Neither future batch has qualification credit. Dependencies,
toolchains, pin, safety limits and security remain frozen. Disk preflight/floors
are enforced; only completed inactive rebuildable caches are reclaimed. Unique
raw receipts, binary snapshots and historical failures remain outside Git,
explicitly separate from reviewed feature-branch preservation. No main merge
or release.

## Synthesized-reference provenance checkpoint

Tested source `8a14fa7d51554a4479fb9ea7fe8fb1af3060be83` adds weak source origins to
synthesized `Exists`, `Empty` and resolved monadic `bind`/`pure` references.
Four unchanged negative originals now match exact upstream checking diagnostics.
Eight multiline/imported/adjacent controls preserve first-error location and
status; two positive programs retain complete constructor results across native
strict evaluation and pure C/JS. [Report and exact receipts](cpu-synthesized-origins.md).

All four originals load, then reject during checking. No generated execution is
claimed after rejection. An additional census-only match in
`check/monad_do_destructure` moves the existing pattern diagnostic from the outer
`do` to its bind statement; loading fails and checking/execution remain unrun.
That supplementary observation does not add a completed fixture-family link.
Core terms, name resolution, argument/binder order, evaluator policy, budgets,
guards and cancellation remain unchanged. Independent review found and closed a
test metadata-lifetime issue; the first gate was intentionally interrupted and
preserved, then the corrected source received a complete new gate. No partial
run counts as passing. External CLI consumers must still inspect `exit_code()`;
previous emitter/DefDecl caveats remain.

Final standard gate: **1,122 / 0 / 49**; focused strict Clippy passes. The new
270-observation matrix and renewed 246-observation prior matrix total **516**,
with zero timeouts and deterministic repeated groups. Existing aggregate/sharing,
demand, cancellation and guard regressions pass. The old forged-IO upstream C
generation failure and Node direct-IO host failure reproduce separately; failed
C generation still has no execution credit. Strict IO proof exclusions remain
distinct from executable success.

A fresh census preserves all **1,302 statuses**, including all 439 negative
rejections. Five stderr streams improve to exact upstream matches: four checker
locations and one load location. Renewal of the prior 489 cases yields **30 exact
diagnostics, 385 strict IO exclusions and 74 unresolved differences**; four old
generated-reference gaps are closed. The remaining 74 include 64 acceptance
differences and ten differing rejections (two source/label cases and eight
IO/foreign preemptions). All upstream streams renew unchanged. The ten separately
tracked old cells remain exact to their prior outcomes in 40 observations;
14 positive and 18 unfinished-hole controls preserve their outcomes. Historical
supplementary all-target lint failures remain unwaived and were not rerun.

Detailed links cover **399 originals**; **903 remain unlinked**: 352 checker
negatives, one checking-only module, 13 interpreter-profile cases and 537 compiled
candidates. Only four checking links were added, with no fabricated runtime
passes. Linked/partial/historical coverage is not full CPU parity. Existing
printer/host/foreign/resource/API, F32.read, error-order and incomplete-grid
requirements remain unwaived. Mid-normalization cancellation is not claimed.

Next independent diagnostic work is constructor-definition labels, including
`check/field_type_bound`, and the `comptime/later_def` substitution source site.
The next substantive candidate remains the datatype-declaration/empty-datatype/
empty-parameter-group batch, applicability first. No proposed work is counted as
qualified. Frozen dependencies/toolchains/pin/limits remain; no installs, desktop
or GPU work. Disk preflight and floors are enforced; only verified inactive
rebuildable caches are reclaimed. Unique historical evidence is preserved.
Raw captures and receipts remain outside Git; reviewed feature publication
preserves source and compact evidence without main merge or release.

### Constructor-definition provenance checkpoint

Tested source `1e8c0c3fba2315ecfeecfd86574044b3feb39608` corrects constructor-domain diagnostic labels without
changing Lhs, internal errors, checking order or work. Reduced non-kind reports
also preserve alias-body locations and parameter context. Seven unchanged
originals and eleven adjacent controls match exactly at repeated CLI/compiler
boundaries; eight further census-only changes remain distinct. All 15 changed
stderr streams match the pinned checker. [Report](cpu-definition-provenance.md).

The standard gate passes **1,126 / 0 / 49**. New and renewed matrices contain
**687 observations**, zero timeouts and deterministic repeats, including retained
C-generation and Node host failures; failed generation still has no execution
credit. All 1,302 checking statuses remain unchanged. The old 489-case subset is
31 exact / 385 strict IO exclusions / 73 unresolved. Six of ten prior diagnostic
cells are now exact; three pattern anchors and unsafe_many remain unwaived.
Supplementary all-target lint failures and all other historical disagreements
remain open. Repository consumers handle status-bearing output; external
consumers remain unaudited.

`comptime/later_def` and four adjacent reproductions remain true source-location
mismatches. Shared Ref identities protect compile binding behavior, so fresh-leaf
and call-head-only shortcuts were rejected by review. Next is private per-use
provenance carried separately from core terms; no new semantic policy is assumed.

Coverage is **406 linked / 896 unlinked**, with unlinked categories 345 checker
negatives, one checking-only, 13 interpreter-profile and 537 compiled candidates.
No new runtime credit is assigned to negative fixtures or supplementary census
observations. Dependencies, upstream, toolchains, budgets, guards and cancellation
boundaries are unchanged. Disk preflight/floors and inactive-cache controls remain;
unique raw evidence and captures are retained outside Git. Reviewed feature-branch
preservation is separate from main merge, release or parity acceptance.

### Compile-time occurrence provenance checkpoint

Tested source `59f04c46f6b3a503367c95884338f9404216f7d1` closes `comptime/later_def` and four retained adjacent
source-window differences without copying reference nodes or changing pointer-based
macro behavior. Private occurrence anchors survive forwarding, lambda substitution,
constructor fields and quantified datatype syntax. Independent review corrected
three propagation omissions before final acceptance. [Report](cpu-template-occurrences.md).

The full gate passes **1,135 / 0 / 49**. Twenty exact negative cases and two positive
programs are qualified; two additional macro-rebinding load refusals still have
unqualified diagnostic differences. New/renewed matrices contain **696 observations**,
zero timeouts and deterministic repeats. All 1,302 checking statuses are unchanged;
only `later_def` changes stderr, now exactly upstream. The prior 489-case subset is
32 exact / 385 strict IO proof exclusions / 72 unresolved. Historical host/generation
failures, unrun executions, semantic disagreements and supplementary lint failures
remain explicit. No higher limits, new policy exceptions or cancellation claims.

Coverage is **407 linked / 895 unlinked**: 344 checker negatives, one checking-only,
13 interpreter-profile and 537 compiled candidates remain unlinked. Links include
partial/blocked/historical evidence, not universal parity. Next are the three
remaining pattern-anchor cells; the substantive applicability-first candidates are
`parse/datatype_declaration_form`, `empty_datatype` and `empty_param_group`.
The two earlier candidates' host-stack failures were corrected by extracting unchanged
reference/parser rules; both failed gates remain historical evidence. No stack or budget
was increased. All frozen inputs and API caveats remain. Reviewed publication preserves
Git content; unique raw evidence and binary captures remain outside Git.

### Pattern binder source-location checkpoint

Tested source `3b144f16f1a6dd5344a668dc0bd31a488c602aeb` closes the three retained usage-diagnostic location gaps
in `check/negative_data_field_000`, `check/negative_data_reuse` and
`halt/contraction_growth`. Eleven exact negatives and two positive graph/value
controls cover named, reusable and synthesized field binders, imported/multiline
locations and selected/fallback-row priority. The adjacent reusable-function-field
kind location remains explicitly unresolved. [Report](cpu-pattern-locations.md).

The standard gate passes **1,138 / 0 / 49**. Focused default-stack, shared-reference
and checker-accounting controls pass. The matrices contain 516 observations with
zero timeouts and deterministic repeats; historical upstream generation/Node host
failures remain separate from unrun execution. All 1,302 checking statuses are
unchanged; only the three target stderr streams improve, each exactly upstream.
The old 489-case subset remains 32 exact / 385 strict IO exclusions / 72 unresolved;
nine of the ten separate old cells are exact, with `unsafe_many` still unresolved.
No semantics, guards, budgets, dependencies, toolchains, cancellation boundary or
public API changed. Independent review completed; full parity remains incomplete.

Coverage is 410 linked / 892 unlinked: 341 checker negatives, one checking-only,
13 interpreter-profile and 537 compiled candidates. Links include partial/blocked
and historical evidence. Next is the reusable-let kind-source diagnostic; the
substantive datatype-declaration/empty-datatype/empty-parameter batch remains
unqualified. Compact unique original failure streams are now committed with the
fixtures; bulk raw evidence and captured binaries remain outside Git. Disk floors
and inactive-cache safeguards remain. Feature checkpoint only, no merge or release.

### Reusable-let locations and full checking census

Tested source `ddd4f0a9bcc6d8b224b1c17d45fe63b91f6712ac` closes reusable-let expected-kind source locations,
including unchanged `check/cop_let_formation`. Six exact controls plus identity,
unsourced fallback, default-stack, sharing, work and cancellation regressions pass.
The standard gate is **1,141 / 0 / 49**; 606 bounded matrix observations preserve
existing failed/unrun lanes. [Report](cpu-let-kind-locations.md).

The full 1,302-fixture checking census now measures **366 both-accept, 243 exact
rejections, 193 differing rejections, 494 acceptance disagreements and six upstream
Node stack failures**. Of 687 differences, 385 exactly renew documented strict-IO
proof exclusions: **302 differences remain beyond that subset, plus six oracle
failures**. The older 489 subset remains 32 exact / 385 IO / 72 other; 72 is not the
global residual count. All port statuses are unchanged. Of 196 stderr changes,
195 are host-source-line shifts; one language diagnostic improves exactly. No
comparison was weakened to ignore those shifts.

Family links remain 410 / 892 unlinked (341 checker negatives, one checking-only,
13 interpreter-profile, 537 compiled candidates); census evidence is separate from
runtime qualification. Four exploratory diagnostic/phase differences and all prior
semantic/oracle/API gaps remain unwaived. Frozen inputs/limits and the acceptance
criterion remain intact. Reviewed source and compact receipts are pushed only to a
feature branch; bulk logs and binary captures remain outside Git. Next independent
work is nested kind provenance or the substantive datatype-declaration batch.

### Nested-kind provenance and exhaustive checking triage

Tested source `244b659c78caa56c4cc2d1df25daa9e693e591b8` closes nested-kind/annotation child provenance with
three metadata additions; checking/runtime semantics and frozen limits are unchanged.
Thirteen exact negatives, ownership/source-priority units and tracked/plain work
controls pass. Independent review completed. Standard gate **1,143 / 0 / 49**;
840 matrix observations retain existing failed/unrun lanes. [Report](cpu-nested-kind.md).

The complete baseline 302 is explicitly triaged: 151 diagnostic-only candidates,
109 acceptance mismatches and 42 other cause/phase/preemption differences. These
are observational categories, not exact qualification or 302 CPU defects. Six Node
oracle stack failures remain outside 302. Renewed full-census non-IO differences:
300; all 1,302 port statuses remain unchanged. Historical receipts retain
original source identities. No extra IO or resource waiver is introduced.

Classification selects a substantive next task: a pure `1n++k` pattern yields 4n
upstream but load-rejects as String.append in the port. Ordinary affine and spaced
controls remain refused; 12 process observations and six phase probes are retained.
No fix is included yet. Family coverage stays 410 linked / 892 unlinked; checking triage is
separate from runtime qualification. No dependencies/toolchains/limits changed,
no desktop/GPU/merge/release. Compact receipts are pushed; bulk evidence stays local.

### Reusable Nat pattern acceptance and execution

Tested source `b9fc3a7febb2a1a0a3ddef53395ee3ee69847196` corrects adjacent `1n++k` numeric-tail parsing,
with transient token restoration for template replay. No checker/runtime policy or
safety limit changes. Independent review completed. The unchanged original prints
`22` in port native/C/JS and upstream C/JS; upstream direct Node remains host-blocked.
Nine positive complete graphs and six negative cases retain precise lane results.
[Report](cpu-reusable-nat.md). Standard gate **1,150 / 0 / 49**; 346 bounded matrix
observations retain native String presentation, two diagnostic differences and the
separate four-plus parser gap. Earlier failed test-harness attempts remain evidence.

All 1,302 checking statuses are unchanged. One phase/diagnostic improves from load
rejection to strict IO proof refusal; no extra IO waiver is added. Current residuals
remain 300 beyond 385 historical IO exclusions, plus six upstream stack failures.
Baseline 302 and current 149/109/42 classification identities remain distinct.
Coverage is 411 linked / 891 unlinked (341 checker negatives, one checking-only,
13 interpreter-only, 536 compiled candidates). Links are not universal parity.
Frozen pins/dependencies/toolchains/budgets and all historical disagreements remain.
Reviewed feature checkpoint only; no desktop/GPU, merge or release. Compact receipts
are in Git; bulk unique evidence and snapshots remain local.

### Repeated-prefix grammar and runtime qualification

Tested source `ead74b8db6e720be3047a434685cbaf55dc69dac` closes generic repeated-prefix parsing, including
`1n++++k`, through one context-aware character consumer. Template replay/error
restoration, ordinary infix append and declaration quantity rules remain intact.
Twelve complete value graphs and five executable lanes qualify applicable cases;
IO proof exclusions, direct Node host failure, native String display and six
negative diagnostic differences remain separate. [Report](cpu-prefix-markers.md).

Standard gate **1,155 / 0 / 49**; 490 process observations and 92 phase probes,
no timeouts. Independent source review completed. All 1,302 census phases/statuses and raw output streams match
the prior checkpoint exactly: 300 differences outside 385 historical IO
exclusions, plus six upstream stack failures. Current baseline-triage residuals
remain 149 diagnostic / 109 acceptance / 42 other; no new waiver or runtime link.
Coverage remains 411 linked / 891 unlinked with applicability categories unchanged.
Frozen dependencies/toolchains/limits and all historical disagreements remain.
Disk floors and inactive-cache controls preserved all source and unique evidence.
Reviewed feature checkpoint only, no merge/release. Next substantive gap is the
`base/list_fold` bundled `List.for_each(~...)` load refusal; not qualified here.

### List.for_each executable qualification and strict Base boundary

Tested source `2dacf522eba2665652036b7b6ed369a2ba2c16d3` renews the unchanged `base/list_fold.bend` across all
five executable runtimes, with two repetitions and durable regression/ledger
coverage. Its strict load refusal comes from the execution-only Base template;
this was already executable at milestone 5.21, not a newly fixed runtime defect.
Independent review found no justified production change. [Report](cpu-list-for-each.md).

Seven authored positives include pure full-graph companions, empty/direct/nested
callbacks and halt-after-output. Six negatives preserve executable rejection and
strict-mode separation. Standard gate **1,159 / 0 / 49**; 288 matrix observations,
84 phase probes, zero timeouts. Four executable diagnostics, direct upstream
Node host failures and two generated-JS Bun-dependent cases remain explicit.
All 1,302 port phases/statuses/streams and census classifications are unchanged;
one known upstream stack failure varies from load to check. There remain 300
differences beyond 385 historical IO exclusions plus six upstream stack failures. No new waiver; current triage residuals stay 149 / 109 / 42.

One renewed original runtime link gives 412 linked / 890 unlinked (341 checker
negatives, one checking-only, 13 interpreter-profile, 535 compiled candidates).
Links do not establish full parity. Frozen inputs/limits and historical failures
remain; inactive-cache safeguards retain source and unique evidence. Reviewed
feature checkpoint only. Next substantive gap: radix/list-map-fold originals,
with applicability established before execution; unqualified here.

### [x] 5.121 Complete native radix sort within fixed work limits

Tested source `d224e79d33b4a0ab26f701b57f448056b63b790a` adds sealed packed U32.shr/and execution through
the existing lazy fallback, without changing checking, demand, accounting or
budgets. The unchanged radix original now prints 6; list/map-fold remains
1111111. Five executable lanes match both originals and the companion’s 12 boundary
output lines, each program run twice. Independent review completed. [Report](cpu-list-sort-map.md).

Standard gate **1,162 / 0 / 49**; focused 16 unit and 59 integration controls;
66 matrix and 18 phase observations, no timeouts. Ten before/after reductions
retain earlier budget failures. Direct Node host failures and strict IO checking
exclusions remain distinct. All 1,302 port census phases/statuses/streams are
unchanged: 300 differences beyond 385 historical IO exclusions plus six oracle
stack failures. Historical triage identities and all unresolved contracts remain.

Coverage is **414 linked / 888 unlinked**: 533 compiled candidates, 341 checker
negatives, 13 interpreter-profile cases, one checking-only. Links do not establish
full CPU parity. Frozen dependencies/toolchains/pin/limits retained, no new API
changes. Safe-cache reclamation preserved unique evidence; bulk raw artifacts
remain outside Git. Reviewed feature checkpoint only, no desktop/GPU/merge/release.
Next independent runtime candidates: map_set_ops and list_string_lookups.

### [x] 5.122 Qualify Map/Set and list/string executable originals

Tested source `6dc0d456e2a296bab513f8c067e5e70009d41e93` adds tests only. The unchanged map/set and list/string
originals retain 11111 and 1111 across five executable lanes twice. Two companions
add 20 output lines for empty/NUL keys, replacement/deletion, Unicode, Set duplicates,
list bounds and fixture-local search/trim. No production defect or fix was found.
Independent review strengthened repeated-pop checking. [Report](cpu-map-set-lookups.md).

Standard gate **1,163 / 0 / 49**; 35 focused tests; 88 matrix and 24 phase
observations, zero timeouts. Direct Node host failures and strict IO checking
exclusions remain separate. The full checking census was not rerun: its last
measured source remains d224e79, with 300 differences beyond 385 historical IO
exclusions plus six upstream stack failures. Production tree identity is verified;
old measurements are not relabelled. All earlier unresolved contracts remain.

Coverage is **416 linked / 886 unlinked**: 531 compiled candidates, 341 checker
negatives, 13 interpreter-profile cases and one checking-only module. This is not
full CPU parity. Frozen inputs, budgets, guards and cancellation boundaries retained.
Compact evidence is pushed; unique bulk artifacts remain outside Git. No desktop,
GPU, merge or release. Next bounded candidates: string_kit and string_algorithms.

### [x] 5.123 Qualify bundled string kit and local algorithms

Tested source `237c527d099da0e8af42642620b8d48159a7a5b1` adds tests only. Unchanged string_kit (36 lines) and
string_algorithms (`4`) plus two companions (23 edge output lines) match across
five executable lanes twice. No production defect or semantic change was found.
Unicode/ASCII, empty fields, CRLF, internal NUL and local overlap/fuel/splice
contracts are covered. [Report](cpu-string-kit-algorithms.md).

Standard gate **1,164 / 0 / 49**; 36 focused tests; 88 matrix observations with
40 exact executions; 24 phase observations; zero timeouts. Direct Node host
failures and strict IO checking exclusions stay separate. The full census was
not rerun: the last measured source remains d224e79, with 300 differences beyond
385 historical IO exclusions plus six upstream stack failures. Production tree
identity is verified; historical measurements and all unresolved contracts remain.

Coverage is **418 linked / 884 unlinked**: 529 compiled candidates, 341 checker
negatives, 13 interpreter-profile cases and one checking-only module. Links are
not full parity. Inputs, guards, budgets and cancellation boundaries stay frozen.
Reviewed compact evidence is pushed; unique bulk evidence remains outside Git.
No desktop/GPU, merge or release. Next candidates: base/string_ops and
eval/string_roundtrip, whose actual entry points are IO.

### [x] 5.124 Qualify local string operations and bundled round trips

Tested source `a6857164bbcd4fb6a0b3aecdac0344ba3c63583a` adds tests only. Originals retain 11 and 341 across
five executable lanes twice; two companions add 28 output-line observations.
Space-only local trim, Unicode split/search/slicing, empty-list join/split and NUL
round-trip preservation agree. [Report](cpu-string-ops-roundtrip.md).

Standard gate **1,165 / 0 / 49**; 37 focused passes; 40 exact executions among
88 matrix observations, plus 24 checking probes. Strict IO exclusions and direct
Node host failures remain separate. No production, dependency, policy or limit
change. Full census not rerun: d224e79 remains the measured source with 300
differences beyond 385 historical IO exclusions and six upstream stack failures.

Coverage is **420 linked / 882 unlinked**: 527 compiled candidates, 341 checker
negatives, 13 interpreter-profile cases and one checking-only module. All older
unresolved contracts remain; links do not imply completed parity.

The next substantive gap is now reproduced independently: pure
parse/string_pattern_sugar matches "hi" upstream, but port loading refuses at the
64-level pattern guard. Fourteen separate bounded observations retain interpreter/
C O3/JS success, port checking/emission refusal and unrun port generated execution.
No guard change or exception is approved by this report. Compact unique failure
evidence is in Git; bulk artifacts remain local. No desktop/GPU, merge or release.


## String-pattern depth investigation: unresolved original

At `a6a62afea10d57d6142855a3ceaf2d71f9e14c26`, the standard gate is
**1,168 / 0 / 49**, with 70 exact executions for seven bounded literal-pattern
companions and 278 total matrix observations. This does not qualify the original
`parse/string_pattern_sugar.bend`: upstream accepts and executes `True{}`, while
production still refuses loading at the 64-level pattern compiler guard.
[Report, sources and compact failure evidence](cpu-string-pattern-depth.md).

A separately archived, unaccepted field-fragment experiment retires compiler
working states and loads the original, but reveals a second refusal: the complete
upstream-equivalent alpha-tree has depth 138 (root 0), exceeding the unchanged 128
kernel input guard. Independent review recommends no production adoption. The
next substantive work is a reviewed compact-lowering design that preserves
constructor, checking, normalization and demand semantics plus all guards; a
graph-preserving traversal adjustment alone cannot close this fixture. No limit
change, skipping of shared nodes, unsupported-host waiver or new exception was
made. Candidate generated execution remains unrun because emission is refused.

The original checking disagreement was renewed twice. The full census remains
historical at d224e79: 300 differences beyond 385 historical IO exclusions plus six
upstream stack failures. Coverage remains 420 linked / 882 unlinked; failed or
exploratory observations do not create qualified inventory coverage. The retained
wrong-type companion matches the checking diagnostic; Nat65 and 65-column guard
controls retain their distinct acceptance/phase disagreements. All earlier open
semantics, host, foreign, API and supplementary-lint cells remain unwaived.

The initial gate's disk-floor stop and the incomplete 110-observation harness run
are preserved separately from the successful renewed gate and completed matrix.
Inactive rebuildable caches were reclaimed/relocated with byte verification;
unique historical evidence was retained. No dependencies, toolchains, upstream
pins, budgets, cancellation or demand policy changed.

### Compact-pattern research and list slicing qualification

Tested source `fff765af30bf3ee4bf4b5632a2c73962917586fa`: production unchanged. A typed local continuation
reduces the experimental input to depth 72 and preserves expanded/open alpha-trees,
but whole-function normalization still refuses at 128. The unchanged string-pattern
original still refuses at load 64; port generated executions remain unrun.
[Design, independent review and precise decision](cpu-compact-patterns.md). No guard
exemption, changed budget, new semantic exception or production adoption.

Independent base/list_slice and base/list_ops plus two companions pass 40 exact
five-lane executions, twice per program; 88 observations and 24 phase probes.
Standard gate **1,169 / 0 / 49**, targeted new-test Clippy passes.
[Report](cpu-list-slice-pipelines.md). Direct-Node host and strict IO refusals remain
separate. Coverage **422 linked / 880 unlinked**: 525 compiled candidates, 341 checker
negatives, 13 interpreter profiles and one checking-only module. Links do not imply
full parity. Full census remains historical at d224e79: 300 differences beyond 385
historical IO exclusions plus six upstream stack failures. Earlier unwaived
semantics/host/API/oracle/lint cells and unqualified 688-vector exploration remain.
Next independent substantive candidate is base/read_bounds, source-inspected only.
Frozen dependencies/toolchains/reference/budgets retained; no desktop/GPU or release.

### Decimal read bounds and retained aggregate work gap

Tested clean source `7305b890a201ead43dd3029a6e8e40c9aba22409`; no production change. Original read_bounds
matches native/port C/JS/upstream C; upstream JS prints its numeric prefix then
fails importing bun:ffi. Original show_read matches all five lanes. Nat/helper
controls match; file-error control retains the same JS host failure. The complete
word aggregate fails native after five lines while all four generated lanes pass.
[Report and contracts](cpu-read-bounds.md); no failed aggregate or host cell waived.

Matrix 132 observations: 54 exact executions, 4 JS host failures, 2 native budget
refusals; 36 separate checking probes. Native isolation 40 observations: 16 inputs
individually pass twice, prefixes 4/5 pass and 6/16 fail. This is cumulative resource
exhaustion, not proof of an accounting defect. Next measure packed/ordinary U32.sub
and subsequent arithmetic fallback before considering a demand-preserving fix.

Gate **1,170 / 0 / 49**, targeted test Clippy passes. Historical full census remains
d224e79: 300 differences beyond 385 historical IO exclusions and six oracle stack
failures; new phase evidence does not redate or waive that set. Coverage
**424 linked / 878 unlinked**: 523 compiled candidates, 341 checker negatives,
13 interpreter profiles, one checking-only module. read_bounds explicitly retains
blocked upstream JS. Compact-pattern research and older disagreements remain open.
Frozen inputs/budgets/guards retained; unique failures preserved; no desktop/GPU
or install, merge or release.


### Packed subtraction and combined word-reader work

Tested clean source `7bf8cec77c94866ff5ad467335526f19b7ccaa86`. The unchanged 16-result word aggregate
now matches all five executable lanes, twice. The measured packed-only `U32.sub`
correction avoids downstream structural arithmetic fallback: native completes in
739,206 successful budget charges, versus refusal at 2,000,000 after five lines.
The two-million budget, guards, charging sites, ordinary fallback, approved demand
and cancellation semantics are unchanged. [Report and exact evidence](cpu-word-reading-work.md).

Gate **1,173 / 0 / 49**; targeted strict Clippy passes. Matrix 198 observations:
66 exact successful and 20 exact intentional-failure executions, with remaining
host/proof cells explicit. All 40 native individual/prefix controls pass; 54 phase
probes distinguish strict load/check refusal from executable acceptance. New
ordinary/lazy, wraparound, private-request/Nat-overflow, GC, cancellation and genuine
budget-exhaustion regressions pass. Independent review found no blocker.

Renewed six-negative matrix retains 12 open observations; demand matrix retains two
C diagnostic-order disagreements and six separately approved C wildcard exceptions.
Their exit 1 results are retained, not recast as passing gates. Full checking census
is still historical at d224e79: 300 differences outside 385 old IO exclusions and
six upstream stack failures. Production runtime changed; no full-census renewal
or whole-tree equivalence claim. Coverage stays **424 linked / 878 unlinked**
(523 compiled candidates, 341 checker negatives, 13 interpreter profiles, one
checking-only module). Links do not establish full CPU parity.

Compact-pattern research remains unadopted; the original string-pattern 64-level
load refusal is still substantive open work. All other prior semantic/host/resource,
API and supplementary-lint gaps remain unwaived. Frozen inputs and disk floors
preserved; small receipts and reproduction instrumentation in Git, bulky local
evidence separately identified. No desktop/GPU, installs, merge or release.


### Competing-error decision and invalid-token diagnostics

At clean tested source `d8122bf54fb0879a3307ee1873f5ec607a824e40`, the order question remains
unresolved: **one direct Nat-first fixture, two primary O3 observations**. Independent
review found no port traversal defect; C ANF hoists its request failure before an
inline arithmetic failure. Six O0/O1/O2 observations reproduce the same case;
these are separate from six approved wildcard observations. All 78 ordering
executions match before/current, with 20 generated C/JS artifacts byte-identical.
Word O0 still has a failed build and two unrun executions. Primary demand remains
92 exact / 6 approved wildcard / 2 unresolved; required-parity runner exits 1.
[Precise proposed decision, not adopted](cpu-failure-order-renewal.md).

The independent metadata-only invalid-token correction closes five original
source-load diagnostic gaps. Nine cases pass 162 exact stderr/status observations
and 54 load-phase probes; no execution claim for rejected input. Public parser
errors, acceptance, terms and guards are unchanged. Existing status-bearing CLI
API caveats apply; external consumers were not audited. [Report](cpu-token-expectations.md).

Gate **1,175 / 0 / 49**, targeted strict Clippy passes. Coverage **429 linked / 873
unlinked**: 523 compiled candidates, 336 checker negatives, 13 interpreter profiles,
one checking-only module. Linked receipts are not universal parity. Full census
remains historical at d224e79 (300 differences outside 385 old IO exclusions,
six upstream stack failures); this subset does not redate or subtract from it.
Older host/foreign/resource/F32.read/erased-hole/API/supplementary-lint gaps remain.
Compact-pattern research stays unadopted. Frozen inputs, guards, budgets and disk
floors preserved. Small evidence committed; raw logs/binaries remain local.


### Declaration-boundary diagnostic context

At clean source `27204f16e87e57f4285610dc4cb0dab23b622988`, nine unchanged originals now match pinned
source-load diagnostics. Constructor-row invalid starts use the outer declaration
expectation; existing global declaration refusals retain Bend source excerpts.
Metadata only: public errors, cursor, acceptance, terms, demand and limits unchanged.
Independent source/evidence review completed. [Report](cpu-declaration-boundary.md).

Fourteen negatives pass 252 exact diagnostics and 84 load-phase probes. Three
positive controls pass 30 executable runs, six upstream interpreter results, six
strict evaluations and six strict checks, plus 18 accepted-phase probes. Six C/JS
artifacts are byte-identical before/current. No execution claim for rejected input;
unsafe-decorator control is parser-only. Full gate **1,178 / 0 / 49**; focused checks
74 passed and targeted strict Clippy passed. Historical census remains d224e79:
300 outside 385 old IO exclusions, six upstream stack failures; not rerun globally.

Coverage **438 linked / 864 unlinked**: 327 checker negatives, 523 compiled candidates,
13 interpreter profiles, one checking-only module. Links are not universal parity.
Two numeric lambda-arm diagnostic gaps were reproduced and remain unqualified.
The nat_first decision, compact-pattern guard gap and all prior oracle/host/resource/
F32.read/foreign/hole/API/supplementary-lint distinctions remain unwaived. No new
semantic exception, dependency/toolchain/pin change, raised limit or cancellation
claim. Disk floors and unique evidence preserved; bulk artifacts remain outside Git.


### Numeric lambda-arm diagnostics and full checking renewal

Clean tested source `59ac3fdbdd900be23e6516a3da84bafa31e7574a` fixes the two requested numeric-arm
diagnostics and three adjacent originals. Optional metadata reports the pinned
closing-brace expectation at the consumed colon; public errors/cursor, acceptance,
terms, sharing, demand and guards are unchanged. Independent review completed.
[Report](cpu-lambda-arm-diagnostics.md).

Fourteen negatives pass 252 exact diagnostics and 84 load probes. Four positive
controls pass 40 executable runs, eight interpreter/eval/check observations each,
24 accepted probes and eight unchanged C/JS artifacts. Shared compile-reference
control is parser preservation plus checking refusal, not an execution pass.
Gate **1,181 / 0 / 49**; focused final checks 77 passed; targeted strict Clippy passed.

The full **1,302** census is renewed: 366 both-accept, 318 exact rejections,
494 acceptance disagreements, 118 differing rejections and six oracle failures.
Every one of the historical 385 IO exclusions renewed exactly. **227 non-IO
differences remain: 109 acceptance / 118 rejection**, with six oracle failures
separate. All port statuses are unchanged. Seventy-three exact-diagnostic gains
are cumulative since d224e79, not all this turn. Unary-prefix diagnostic remains
unresolved. One upstream stack failure varied by phase; an initial supplementary
accepted sidecar has no retained streams and is phase-only. Five fully recorded
supplementary reruns all fail; none replaces the census outcome. Current
[table and evidence](cpu-lambda-arm-census.json) retain historical identities.

Coverage **443 linked / 859 unlinked**: 322 checker negatives, 523 compiled candidates,
13 interpreter profiles, one checking-only module. Census matches and family links
are not universal CPU parity. Nat-first order, compact-pattern guard and all prior
host/resource/F32.read/foreign/hole/API/supplementary-lint differences remain unwaived.
Frozen pins/dependencies/toolchains/limits, cancellation boundaries, disk floors and
unique evidence retained. No installs, desktop/GPU, main merge or release. Raw logs
and binary snapshots remain outside Git preservation.


## Acceptance-mode census priority checkpoint

Tested source `67b2035d525ea54b34c8d1ab4e1cf8a8f21e7b2f` changes tests/harness only.
[Classification and evidence](cpu-acceptance-priority.md) renew all 109 selected
strict acceptance disagreements exactly: 92 executable-check acceptances,
15 retained parser/flatten guards, two missing Window/Audio implementation surfaces.
No row is waived and no production acceptance fix is claimed. Last full census
remains 227 non-IO differences (109 acceptance + 118 rejection) and six separate
upstream oracle failures at 59ac3fd; the 109 subset is freshly renewed here.

The unchanged unsafe_many original returns 7 without calling twice. A live
nested reusable-function companion returns 5n. Both agree in 20 five-lane runs
plus four upstream direct-CLI observations. Four negative controls retain safe
signature, source-hole and false-proof boundaries; strict-mode diagnostic
exclusions remain distinct. Standard gate: 1,183 passed / 0 failed / 49 ignored.

Coverage links: 444 linked / 858 unlinked (522 compiled candidates, 322 checker
negatives, 13 interpreter profiles, one checking-only module). Links and mode
classification do not establish full CPU parity. Nat-first remains pending,
compact-pattern research unadopted; host/foreign/F32.read/hole/resource/oracle
and supplementary-lint limitations remain. No production/API/dependency/toolchain,
guard, budget or cancellation changes. No desktop/GPU, installs, merge or release.
Raw logs, generated artifacts and binary snapshots remain outside Git preservation.


## Nat-pattern measurement and erased-binder checkpoint

Final tested source `0aef62717e9587a21b7266c55e659ecdd6e65712`: standard gate
**1,187 passed / 0 failed / 49 ignored**. Production matches source
`a1f8f4d528f741d2db1859689321e83f2d42e3fe`, used for the Nat/diagnostic/census
matrices. [Report and reproducible evidence](cpu-nat-pattern-depth.md).

The two unchanged Nat originals remain load-blocked. Their upstream cores have
loaded depths 164/160 and normalized depths 164/159; no graph-preserving counter
repair can close the existing 128-depth boundary. No Nat fix, guard exemption or
policy change was adopted. Compact lowering/normalization remains an architectural
design task. Eight boundary controls have 16 complete matching graph comparisons;
eight pattern 64 and two literal 128 load refusals are retained. Six near-boundary
programs pass 60 five-lane runs; originals only have upstream C/JS success, with
port generated execution unrun. Controls do not close originals.

The independent erased-binder diagnostic is corrected with public errors, terms,
cursor and guard semantics unchanged. Nine cases pass 162 exact diagnostics and
54 load probes. A separate positive control exposes a pre-existing port-C emitter
failure for `+n = {3 : U32}; (n + n : U32)`: retained-owner-slot refusal, no artifact.
Other available lanes return 6. The positive matrix has 38 successful executions,
one failed generation and two unrun C executions; required-lanes runner exits 1.
This retained ownership case is the next bounded substantive CPU defect.

Full 1,302 checking census at A1: 366 both-accept, 319 exact rejections, 495 acceptance
disagreements, 117 differing rejections and 5 oracle failures. Every port status and
all 385 historical IO observations are unchanged. Current non-IO residual 227 is
**110 acceptance +117 rejection**. Prior 109 acceptance classifications remain;
`dead_param_fork` newly accepts in the unstable upstream oracle (five followups:
4 accept, 1 stack failure). The only port diagnostic delta is the now-exact unary
prefix original. Do not infer reduced total residuals or stable oracle parity.

Coverage **445 linked / 857 unlinked**: 522 compiled candidates, 321 checker negatives,
13 interpreter profiles, 1 checking-only module. Links are not universal parity.
Nat-first remains pending; compact-pattern research unadopted. All prior host,
foreign, F32.read, hole, resource, API and supplementary-lint caveats remain. Frozen
inputs, budgets and cancellation boundaries preserved. Independent source and
measurement review completed. No installs, desktop/GPU, main merge or release.
Compact evidence is committed; unique raw logs/graphs/binaries remain local and
are not preserved by Git publication.


## Retained legacy C owners

Clean tested source `208832a8282a7e1688aa725341cdf9ea0b5447c0` repairs the retained
owner invariant at the legacy-to-flat result boundary. Inline scalars now enter a
persistent writable frame slot before partial reuse, captures or fork transfers;
owner validation, use counts, evaluation order, cleanup and limits are unchanged.
One existing compiler charge and frame slot is added per legacy boundary.
Independent source and evidence reviews completed. [Report](cpu-retained-owner.md).

The unchanged reusable-U32 original executes as 6 in port/upstream C and JS. Pure
port `run` normalizes; separate IO companions exercise the native runtime. The
374-command matrix records 126 successful and 20 intentionally failing executions,
4 pure-normalization observations, 84 generations/builds, 112 checking/phase commands
and 28 exact negative CLI observations. All repeats are deterministic. All 18
before/after owner-generation refusals close; 52 overlapping non-C observations are
unchanged. Pure F32/string residuals and pure-overflow depth refusal remain distinct
from executable results. New successful C tests inspect allocation/frame cleanup;
existing worker-failure/cancellation controls pass. Gate **1,190 / 0 / 49**; targeted
59-test regression run and strict test Clippy pass.

Renewed upstream-O3 demand remains 92 exact / 6 approved wildcard / 2 unresolved
Nat-first observations, runner exit 1. Port-C demand builds retain their recorded
strict/no-explicit-optimization profile. Nat-first remains pending and Nat/string
compact-lowering research unadopted. Full census is not renewed: last A1 measured
227 differences outside 385 historical IO members (110 acceptance / 117 rejection),
plus 5 oracle failures. This is not literally an all-non-IO classification: one
additional IO-symptom candidate remains in the residual, unwaived. Unstable
upstream acceptance and historical failure receipts remain separate.

Inventory linkage remains 445 / 1,302, with 857 unlinked entries by applicability;
new authored controls do not manufacture upstream coverage. Prior host/foreign,
F32.read, hole, resource, oracle, API and supplementary-lint gaps remain. Next concrete
independent family: filled-definition diagnostics for live use of unfilled laws
in assert_fill_scope, axiom_use and imported/forward-reference companions; renew
before correcting metadata, preserve refusal. Frozen dependencies/toolchains/pin,
budgets and cancellation boundaries remain. No installs, desktop/GPU, merge/release.
Compact evidence is in Git; bulky unique raw evidence/binaries remain local.


## Live unfilled-law diagnostics

Clean tested source `d85cab01eae230d423f718220d945c4cd0a25d62` adds optional
source/context diagnostics after the existing live-law refusal; error identity,
acceptance, erased demand, term sharing and budgets stay unchanged. The six pinned
originals and five authored controls now match exact checking stderr. Two unused
open-law inventory diagnostics remain unresolved. Public invoke consumers must
inspect status-bearing output; external consumers were not audited.
[Report and commands](cpu-unfilled-law-diagnostics.md).

Negative matrix: 198 exact streams plus 66 separate phase probes. Positive matrix:
128 commands including 42 runtime executions and 16 pure-normalization observations;
strict IO exclusions stay distinct. Gate **1,193 / 0 / 49**, targeted regressions and
Clippy passed. Source and evidence reviews precede checkpoint publication.

Full 1,302 checking census: 366 both accept, 325 exact reject, 111 different reject,
496 acceptance disagreements, four oracle failures. All port statuses/phases are
unchanged and exactly six port stderr streams improve. All 385 historical IO
members renew exactly; outside that set 222 differences remain (111 acceptance /
111 rejection). Upstream borrow_fork_hold shifts from historical stack failure to
acceptance in this measurement, while supplemental probes remain unstable; it is
not a port fix. The historical six-failure cohort and dead_param_fork instability
remain separate. [Measured reconciliation](cpu-unfilled-law-census.json).

Coverage links: 451 applicable pinned originals / 1,302; 851 unlinked (522 compiled
candidates, 315 checker negatives, 13 interpreter-only, one checking-only). Linked
receipts are not universal parity. Nat-first stays pending/unwaived, compact Nat/
string lowering unadopted, and all earlier host/foreign/resource/F32.read/hole/API/
supplementary-lint gaps remain. Next demonstrated independent diagnostic boundary:
axiom_warn/open_claim final TODO counts, without weakening incomplete-proof refusal.
Frozen dependencies/toolchains/pins/budgets/cancellation and disk floors remain.
Compact evidence is in Git; bulky unique raw logs and binaries remain local.


## Final open-law TODO summaries

Implementation/differential source `ca333c470dd227291ad45ab1b846a633fef730eb` preserves the existing
final inventory rejection and adds private nonzero presentation metadata. Global
pinned summaries deliberately have no Bend source span. Error message/equality,
zero-inventory acceptance, erased/source-hole refusal, evaluation and budgets remain
unchanged. CLI invoke returns existing status-bearing output; external caller audit
remains outstanding. [Report and commands](cpu-final-todo-diagnostics.md).

Two unchanged originals plus five authored summaries pass 126 exact stream commands
and 42 additional check-phase probes. Positive/applicability matrices add 76 and 24
commands: pure normalization, native IO and generated C/JS remain distinct. Empty
modules check successfully; their emitters refuse missing Base before any missing-
main boundary, with runtime unrun. Gate **1,198 / 0 / 49**, focused 338/0/1 and targeted
Clippy pass. The successful full gate is at test-only `6507da388f35a88767d309b14878835bda0cdcec`,
with identical production source. Initial ca333c4 gate failed on a stale host-line
snapshot; six exact snapshots were renewed while preserving historical files and
all assertions. Earlier three source-hole TODO differences remain unwaived.

Full census: 366 both accept / 327 exact reject / 493 acceptance disagreements / 109 different rejections /
7 host-stack failures. All port statuses/phases unchanged. Two summary gaps close;
156 additional raw streams only move their host formatter source line and remain
raw disagreements. Historical 385 IO members renew exactly; residual 217 = 108 acceptance
+ 109 rejection. The drop 222→217 includes three unstable upstream acceptance→stack
transitions, not five port fixes. Supplemental probes remain mixed and do not replace
raw counts. [Measured census](cpu-final-todo-census.json).

Coverage 453/1,302 applicable originals; 849 unlinked = 522 compiled candidates + 313 checker
negatives + 13 interpreter-only + 1 checking-only. Historical receipts remain tied to their
commits. Nat-first stays pending, compact lowering unadopted, existing foreign/host/
resource/F32.read/hole/API and supplementary-lint gaps remain. Next independent
family: self-call refusal diagnostics for cop_circular_witness_000, descent_field_order
and partial_self_call_stuck, preserving all descent guards. Inputs/toolchains/limits
and disk floors remain frozen. Compact evidence is in Git; bulk raw logs/binaries
remain local. No desktop/GPU, installation, main merge or release.


## Self-call refusal diagnostic checkpoint

Tested `b8a07c233ab3726d9152f13d465130ef1672bb20` attaches bounded source diagnostics only after the existing structural self-call refusal. Rejection, descent field/column ordering, sharing, demand, internal errors and limits are unchanged. Three unchanged negative originals plus four authored cases pass 126 exact streams + 42 phase probes; one unchanged positive and two authored controls pass 78 observations including 24 C-O3/JS executions. Native pure observations are normalization. [Report and exact commands](cpu-self-call-diagnostics.md).

Full gate **1,201 / 0 / 49**, focused unit 1 pass and integration 5 pass, targeted Clippy pass. All port statuses/phases and all but 28 stderr streams remain unchanged in the 1,302-fixture census. Current counts: 366 both accept / 354 exact reject / 494 acceptance disagreements / 82 different rejections / 6 host-stack failures. Of 28 changed diagnostics, 27 are exact; halt/non_structural_descent context-type printing remains non-exact. Historical 385 IO members renew exactly; residual **191** outside that cohort (109 acceptance disagreements and 82 different rejections). Oracle transitions remain separate from port fixes; older instabilities stay unwaived. [Current reconciliation](cpu-self-call-census.json).

Original linkage 457/1,302; 845 unlinked by applicability, not 845 missing runtime tests. Nat-first pending, compact lowering unadopted, external invoke audit outstanding; earlier hole/resource/host/foreign/F32.read/lint gaps remain. Next independent gap: excluded-constructor context printing, with bare family-head diagnostics as an independent alternative, preserving rejection. Frozen dependencies/toolchains/references/limits and disk floors unchanged. Compact evidence is in Git; raw streams/binaries remain local. No install, desktop/GPU, main merge or release.


## Refined datatype context formatting

Tested `9a260404fbb5819870e48f14a78bb80b7afc9181` corrects the shared refined-ADT Display spelling and precedence to match pinned term_show. Public refined-type text intentionally changes; semantic types, source origins, checking, termination, evaluation and limits do not. Five diagnostics pass 90 exact streams + 30 phase probes; two positives add 52 observations including 16 actual C-O3/JS executions. All four port C/JS artifacts are byte-identical to the baseline. Sixteen programmatic printer cases establish text/identity only. [Report and commands](cpu-refined-type-diagnostics.md).

Full gate **1,205 / 0 / 49**, focused **10 / 0 / 0**, targeted strict Clippy pass. In the full 1,302-fixture census, all port statuses/phases and 1,301 streams stay unchanged; halt/non_structural_descent becomes exact. Current totals: 366 both accept / 355 exact reject / 494 acceptance disagreements / 81 different rejections / 6 host/oracle failures. Historical 385 IO members renew exactly; residual **190** outside that set (109 acceptance, 81 rejection). Oracle transitions remain separate from the one port fix. [Reconciliation](cpu-refined-type-census.json).

Original linkage 458/1,302; 844 unlinked by applicability. Nat-first pending, compact lowering unadopted, external invoke audit outstanding; older hole/resource/host/foreign/F32.read/lint differences remain. Next independent diagnostic gap: bare parameterized family head, preserving rejection. Nearby data-family IO exclusions are distinct. Pins/dependencies/toolchains/limits and disk floors unchanged. Compact evidence is in Git; raw streams/binaries remain local. No install, desktop/GPU, merge or release.


## Bare parameterized family-head diagnostics

Tested `c7519a0d1a5efeb872ccc6b1ab011e1b85f04b2e` attaches diagnostic metadata to the existing bare-family rejection. Internal KernelError message/Display/equality and semantic checking/evaluation/termination stay unchanged; private diagnostic metadata (also visible through derived Debug) and public CLI stderr now expose pinned Bend context/source. Existing status-bearing Ok failures still require exit_code handling; external invoke audit remains outstanding. [Report and commands](cpu-family-head-diagnostics.md).

Eight negatives pass 144 exact streams and 48 phase probes. Two positive controls add 52 observations including 16 actual C-O3/JS executions; four port artifacts remain byte-identical to baseline. Focused 291 / 0 / 1 and full standard gate 1208 / 0 / 49 pass, with independent source/evidence review. Incomplete exploratory baseline and mistaken initial unit expectation remain separate failed attempts. No weakening of tests, guards, demand or budgets.

Census 1,302: 366 both accept / 356 exact reject / 494 acceptance disagreements / 80 different rejections / 6 host/oracle failures. All port statuses/phases unchanged; only family_head_bare stderr improves. Historical 385 IO members renew exactly. Residual **189** outside that cohort (109 acceptance, 80 rejection). Upstream transition observations stay separate from the port fix. [Reconciliation](cpu-family-head-census.json).

Original linkage 459 / 1,302; 843 unlinked by applicability; links are not completed parity. Nat-first pending, compact lowering unadopted, external audit and all earlier semantic/oracle/resource/lint gaps retained. Next independent gap: forward_data_table source/unknown-datatype diagnostics; duplicate-constructor load diagnostics are adjacent unresolved work. Frozen inputs/limits and disk floors retained; compact evidence committed, bulk evidence local. No installs, desktop/GPU, merge or release.


## Unknown datatype and constructor provenance

Tested `5013562f98204d5e4694486d0a3f77f366345b7d` attaches message-only metadata to the existing unknown-datatype inference rejection. Internal KernelError message/Display/equality and checking/evaluation/termination remain unchanged. Crate-private metadata, derived Debug and public CLI stderr can differ; existing status-bearing Ok failures still require exit_code handling, with external invoke audit outstanding. Other ADT helper callers and arity/exclusion errors are unchanged. [Report and commands](cpu-forward-data-diagnostics.md).

Eight negatives pass 144 exact streams and 48 phase probes; two positives add 52 observations including 16 actual C-O3/JS executions. Four port artifacts are byte-identical to baseline. Corrected focused checks **10 / 0 / 0**, strict Clippy and full standard gate **1,212 / 0 / 49** pass after independent review. An authored recursive control initially omitted a U32 annotation; both upstream and port rejected it. Its failed emission and focused run remain retained failures, not current passes. No change to guards, budgets, demand or cancellation.

Census 1,302: 366 both accept / 357 exact reject / 495 acceptance disagreements / 79 different rejections / 5 host/oracle failures. All port statuses/phases unchanged; only forward_data_table stderr improves. Historical 385 IO members renew exactly. Residual **189** outside that cohort (110 acceptance, 79 rejection). Oracle changes stay separate from the port correction. [Reconciliation](cpu-forward-data-census.json).

Original linkage 460 / 1,302; 842 unlinked by applicability. Full parity is incomplete. Nat-first pending, compact lowering unadopted, external audit and all prior semantic/oracle/resource/lint gaps retained. Next independent batch: four duplicate-constructor load diagnostics, preserving rejection. Frozen inputs/limits and disk floors retained; compact evidence committed, bulk evidence local. No installs, desktop/GPU, merge or release.


## Duplicate-constructor load diagnostics

Tested `4d6459c0b1a81499fd106c352ae4ce8696db8281` attaches optional metadata at the unchanged duplicate-constructor load refusal. Pinned raw post-name characters and original source windows now match. Public ParseError fields/message/Display and plain load Err remain unchanged; formatted CLI failures use existing Ok with exit_code1. External invoke audit remains outstanding. No acceptance/evaluation/termination/sharing/guard/budget change. [Report and exact commands](cpu-duplicate-constructor-diagnostics.md).

Twelve negative roots plus two adjacent originals pass **252 exact diagnostic commands + 84 separate load-phase probes**. Two positive roots add 52 commands including 16 actual C-O3/JS executions; four generated port artifacts remain byte-identical to baseline. Full gate **1,216 / 0 / 49**, focused **6 / 0 / 0**, strict Clippy pass. Both interrupted fixture-preservation gate attempts and the initial invalid positive control remain incomplete/failed receipts, not passes. Independent review covers the correction and evidence.

Census 1,302: 366 both accept / 363 exact reject / 494 acceptance disagreements / 73 different rejections / 6 host/oracle failures. All port statuses/phases and 1,296 streams unchanged; six duplicate diagnostics become exact. Historical 385 IO members renew exactly. Residual **182** outside that cohort (109 acceptance, 73 rejection). The additional residual decrease is upstream borrow_fork_hold returning to a stack failure, not a seventh port fix. [Reconciliation](cpu-duplicate-constructor-census.json).

Original linkage 466/1,302; 836 unlinked by applicability. Full CPU parity is incomplete. Nat-first pending, compact lowering unadopted; external audit and prior semantic/oracle/resource/lint gaps retained. Next independent family: four duplicate-declaration load diagnostics. Frozen inputs/limits and disk floors remain. Compact evidence is committed; bulk evidence stays local. No installs, desktop/GPU, merge or release.


## Duplicate-declaration load diagnostics

Tested `f0ff543052518d156537ac3380764b342d9e5dd2` attaches private raw-name/source metadata at the unchanged duplicate-declaration and same-target filled-definition refusals. Parser ordering, public ParseError fields/Display and plain loader Err are unchanged. Formatted CLI failures use existing Ok with exit_code1; external caller auditing remains outstanding. Qualification differences and earlier-parenthesis errors remain untouched. [Report and exact commands](cpu-duplicate-declaration-diagnostics.md).

Thirteen negatives plus an adjacent original pass **252 exact diagnostic commands + 84 separate load-phase probes**. Two positives add 52 commands including 16 actual C-O3/JS executions; four port artifacts remain byte-identical to baseline. Full gate **1,220 / 0 / 49**, corrected focused 6/0/0 and strict Clippy pass. Initial focused Clippy exit101 failure and incomplete raw/text-mode phase runner are preserved separately, not counted as passes. No limits, tests or guards weakened.

Census 1,302: 366 both accept / 368 exact reject / 493 acceptance disagreements / 68 different rejections / 7 host/oracle failures. All port statuses/phases and 1,297 streams unchanged; five diagnostics become exact. Historical 385 IO members renew exactly. Residual **176** outside that cohort (108 acceptance, 68 rejection). The additional residual decrease is literal_rows_cubic becoming upstream-host blocked, not a sixth port fix. [Reconciliation](cpu-duplicate-declaration-census.json).

Original linkage 471/1,302; 831 unlinked by applicability. Another 74 authored observations preserve three disagreements outside inventory counts: missing-parenthesis diagnostic order and two namespace declaration acceptance gaps. Upstream pure/C-O3/JS produce 3 or 7; port refuses during load and remains unqualified. Next substantive task: pinned declaration-target selection while retaining imported-law completion. Nat-first pending, compact lowering unadopted, external audit and earlier semantic/oracle/resource/lint gaps remain. Frozen inputs/limits and disk floors retained; compact evidence committed, bulk local. No installs, desktop/GPU, merge or release.

## Imported declaration targets checkpoint

Reviewed production source `757b2b571d53c66716ba93f444f3c6fd6764f19c`; final test/census source `e0a09940c4d0d9317dc5a457af3f256105baf7f0`. Two unchanged authored alias/Base-shadow reproducers now accept and execute with outputs 3/7. Matrix of 396 commands: 368 matching observations, 4 upstream JS startup failures, 8 strict IO refusals and 16 foreign phase observations; no foreign execution claim. Final standard gate: 1,224 passed / 0 failed / 49 ignored. [Report and receipts](cpu-imported-declaration-targets.md).

All 1,302 port census outcomes/streams unchanged. Current residual outside the historical IO cohort: 178 = 110 acceptance + 68 rejection; 176→178 comes solely from two recovered upstream oracles, with 5 oracle failures separate. Historical 385 IO membership renews exactly; 386 symptoms do not broaden it. Inventory coverage stays 471 linked / 831 unlinked; authored fixes do not add inventory links. Keep missing-parenthesis, nat_first, compact lowering, foreign/host/resource/F32.read/printer/hole and external invoke-caller gaps unwaived. Bulk evidence remains local.

## Missing alias-member fallback checkpoint

Tested source `4b868cfd9e5b9d5d0ab1c2d66b5fa7647e6684ed`: unchanged authored `D.local()` reproducer now returns 9, with imported precedence, raw fallback, law/template/constructor and negative controls. Matrix: 430 commands = 408 matches + 4 strict IO observations + 18 unknown-template phase-only observations; 82 successful executions and 36 pure normalizations remain distinct. Full gate: 1,228 passed / 0 failed / 49 ignored. [Report and evidence](cpu-alias-fallback.md).

All 1,302 port census streams/statuses/phases remain unchanged. Current residual outside the historical IO cohort: 177 = 109 acceptance + 68 rejection differences; 178→177 reflects upstream stack failure only. Six oracle failures remain separate; the 385-member historical IO cohort renews exactly. Coverage remains 471 linked / 831 unlinked. Unknown-template text, older oracle/resource/hole/foreign/F32.read gaps, nat_first, compact lowering and external invoke audit remain unwaived. Bulk evidence remains local.

## Unknown-template load diagnostics checkpoint

Comparison source `eb65f84a7a194c1a6d8aae39563705a14fe41842` adds optional metadata at the unchanged tilde refusal. Renewed full-gate source `e9638be533aed465820168ced57b6f27f54e82d9` only renews the current golden and adds evidence. Public ParseError fields/Display, template resolution, evaluation and guards remain unchanged. Newly formatted invoke failures require `exit_code() == 1` handling; external auditing remains outstanding. [Report](cpu-unknown-template.md).

Four unchanged upstream originals plus authored controls pass 464 matching commands: 252 exact negatives, 56 phase probes, 48 C/JS executions and 24 pure normalizations. Twelve port artifacts remain byte-identical. Full gate **1232 / 0 / 49**; source and evidence independently reviewed.

The renewed **1,302-fixture checking census** records 366 both accepted, 372 exact rejections, 494 acceptance disagreements, 64 different rejections and six unchanged upstream stack failures. All port/upstream acceptance statuses and failure phases remain unchanged; all upstream streams remain unchanged. Seven port stderr streams change: the four original tilde diagnostics become exact, while `base/list_fold`, `gfx/app_linear` and `gfx/app_play` retain their strict-mode rejection and acceptance disagreement. Formatting those three refusals does not establish executable parity or admit a new exclusion.

The historical 385-member IO cohort renews byte-exactly without widening. The measured residual outside it decreases **177 → 173 = 109 acceptance + 64 rejection differences**, entirely through the four demonstrated diagnostic fixes. Six oracle failures stay separate (`check/bystander_hypothesis_cost`, `check/deep_hyp_rewrite`, `page/deep_list_walk`, `reg/borrow_fork_hold`, `reg/dead_param_fork`, `rfc/share_loop`); no oracle recovery or regression is claimed. [Reconciliation](cpu-unknown-template-census.json) and [four canonical status delta rows](cpu-unknown-template-census-delta.csv) preserve the current observation identities; the three additional stream-only changes remain recorded in the reconciliation. Historical triage categories are observational: the 173 split into 109 acceptance, 35 diagnostic candidates and 29 other rejection differences; do not call every rejection difference diagnostic-only.

Coverage: 475 linked / 827 unlinked remains applicability-led. Nat-first pending, compact lowering unadopted, external audit and prior host/resource/foreign/printer/hole/F32.read/lint gaps remain. Compact receipts committed; bulk evidence local.

## Untyped compile-time binder diagnostics checkpoint

Tested source `792aaf0abbb5f3c20a046ab39ecc66967350c2b1` attaches optional colon/source metadata at the unchanged telescope refusal. Public ParseError fields/Display, acceptance/evaluation, sharing, guards and budgets remain unchanged. New formatted invoke failures require exit_code1 handling; external audit remains outstanding. [Report](cpu-untyped-binder.md).

434 commands: 408 matching, eight strict-IO exclusions and 18 retained wrong_type observations. Nine colon diagnostics close; wrong_type remains unresolved. This includes 216 exact negative comparisons, 48 phase probes, 52 actual executions and 16 pure normalizations; twelve emitted port artifacts remain byte-identical. Full gate **1236 / 0 / 49**.

The renewed **1,302-fixture checking census** measures 366 both accepted, 373 exact rejections, 494 acceptance disagreements, 63 different rejections and six unchanged upstream stack failures. Every port/upstream status and failure phase is unchanged; every upstream stream is unchanged. The sole port stream change is `comptime/err_untyped`, now an exact rejection. The historical 385-member IO cohort renews byte-exactly, without widening. Residual differences outside that cohort decrease **173 → 172 = 109 acceptance + 63 rejection differences**. This is one measured original-fixture diagnostic improvement, not nine census improvements: the other eight corrected cases are authored controls. Historical triage labels divide the 172 into 109 acceptance disagreements, 34 diagnostic candidates and 29 other rejection differences; those labels do not prove every rejection difference is diagnostic-only. The six oracle failures remain `check/bystander_hypothesis_cost`, `check/deep_hyp_rewrite`, `page/deep_list_walk`, `reg/borrow_fork_hold`, `reg/dead_param_fork` and `rfc/share_loop`. No upstream variation or recovery is counted as port progress. [Reconciliation](cpu-untyped-binder-census.json) and [canonical delta](cpu-untyped-binder-census-delta.csv) preserve exact source and hash identities.

Coverage 478 linked / 824 unlinked remains applicability-led. Nat-first pending, compact lowering unadopted, external invoke audit and all prior unwaived disagreements remain. Compact evidence committed; bulk evidence local.

## Compile-time application type diagnostics checkpoint

Tested source `54bc100bafda627fadd0443592be0e6200235838` preserves weak application-edge and compile-time substitution origins, including actual quantity spans. Correct rejection/evaluation, term identity/sharing, public plain error contracts and guards/budgets stay unchanged. Newly formatted invoke failures still require exit_code1 handling; external audit remains outstanding. [Report](cpu-comptime-type.md).

442 commands: 434 matching and eight strict-IO exclusions. Nine bounded diagnostic differences close; 216 exact negative comparisons, 48 phase probes, 60 actual executions and 20 pure normalizations; fourteen emitted port artifacts stay byte-identical. The adjacent Base callback coordinate mismatch remains unqualified. Full gate **1240 / 0 / 49**. Earlier partial focused failures remain historical failures.

The complete [checking census](cpu-comptime-type-census.json) renews all **1,302** originals at this tested source: **366 both accept, 377 exact rejections, 494 acceptance disagreements, 59 different rejections and six oracle failures**. Both-accept means checking status only. All **385 historical IO members are exactly unchanged**, permitting the same subset accounting; outside that subset the measured residual is **168 = 109 acceptance disagreements + 59 rejection differences**, down from 172. The remaining historical triage labels are 109 acceptance/rejection mismatches, 29 other rejection differences and 30 diagnostic-only candidates; these labels do not waive observations or establish semantic parity.

Only four port stderr streams change: `check/error_window_filled_param_use`, `check/error_window_param_use`, `comptime/err_op_ns` and `comptime/wrong_type` become exact rejections. Port status/phase and upstream status/phase/streams are unchanged throughout; the six oracle failures are unchanged. There is no oracle-variation adjustment in this census. The two error-window gains are checking-only evidence, not additional executable qualifications. The fresh missing-IO symptom count is 386; it does not expand the historical 385-member subset. Earlier census files keep their original identities.

Coverage 481 linked / 821 unlinked remains applicability-led. Nat-first pending, compact lowering unadopted, external invoke audit and other unwaived disagreements remain. Compact evidence committed; bulk evidence local.

## Compile-time reference rebinding checkpoint

Tested source `d0176c3a104688fbe21bb79d9e05330bdc9ce08f` carries existing per-use source metadata into reference-pattern refusals while preserving pointer identity, public errors, rejection order and resource guards. [Report](cpu-comptime-rebind.md).

346 commands: 318 matching, eight strict IO, two retained normalization and 18 lambda diagnostic observations. Eight exact reference-pattern roots; 144 exact negative comparisons, 32 phase probes, 52 executions and 16 normalizations. Twelve emitted port artifacts byte-identical. Full gate **1244 / 0 / 49**. The local pure normalization remains `Nat.add(11n, 1n)` versus upstream `12n`; generated C/JS and explicit native IO execute correctly. No waiver or normalizer correction.

The complete [checking census](cpu-comptime-rebind-census.json) renews all **1,302** originals: **366 both accept, 378 exact rejections, 494 acceptance disagreements, 58 different rejections and six oracle failures**. Both-accept establishes checking status only. All **385 historical IO members renew exactly**; outside that same subset the measured residual is **167 = 109 acceptance disagreements + 58 rejection differences**, down from 168. Historical triage labels for those residuals remain 109 acceptance/rejection mismatches, 29 diagnostic-only candidates and 29 other rejection differences; the labels do not waive observations or establish semantic parity.

Only `comptime/err_rebind` changes, and only its port stderr: it now exactly matches upstream. Every acceptance status, failure phase and upstream stream is unchanged. The six upstream host-stack failures are unchanged; there is no oracle-variation adjustment in this census. The fresh missing-IO symptom count remains 386 and does not enlarge the historical 385-member subset. The authored local normalizer discrepancy is outside this inventory census and remains separately unresolved. Historical receipts retain their original source identities.

Coverage 482 linked / 820 unlinked remains applicability-led. Base coordinates, nat_first, unadopted compact lowering, external invoke auditing and historical disagreements remain. Compact evidence committed; bulk evidence local.

## Closed partial-application normalization checkpoint

Tested source `cd7cabbda37192a7d401d1b3b1a3b9caf37c9f7a` retries named spines exposed by computed heads, preserving existing argument processing and charging the tail retry. The unchanged authored template/let original now has the complete `12n` constructor graph instead of a retained `Nat.add(11n, 1n)` application. An adjacent true proof now accepts; false proofs, holes, nondecreasing recursion, lazy fields, sharing and resource/cancellation guards remain controlled. [Report and exact receipts](cpu-closed-partial-normalization.md).

Full gate **1252 / 0 / 49**. Matrix 298 commands, 110 deterministic repeated pairs, 74 executions and 44 normalization observations, with 12 exact generation refusals whose executions remain unrun. Independent source and evidence reviews found no blocker. No printer workaround, limit increase or public signature change. No mid-normalization cancellation claim.

All 1,302 census outcomes/streams/phases remain unchanged: 366 both accept, 378 exact rejections, 494 acceptance disagreements, 58 different rejections and six oracle-host failures. Historical 385-member IO cohort renews exactly; residual **167 = 109 acceptance + 58 rejection differences** outside that cohort. The authored fix does not create an inventory-wide gain. Linkage remains 482 / 820; Base coordinates, nat_first, compact lowering, external invoke auditing and all previous unwaived discrepancies remain. Compact receipts committed; bulk evidence local.

## Contextual reflexivity rejection checkpoint

Tested source `366c7487fa795685227c72fcdb58a1af4ab3f5df` attaches optional type/context/proof-source metadata only after the existing non-equation reflexivity refusal. Public kernel error message/Display/equality, checking/evaluation/sharing and live safety budgets remain unchanged; derived Debug can change with diagnostic metadata. Tracked CLI failures use the documented Ok output with exit_code1; external invoke auditing remains outstanding. [Report and commands](cpu-contextual-proof-diagnostics.md).

Full gate **1256 / 0 / 49**; focused **51 / 0 / 0**. Matrix 340 commands: 216 exact negative comparisons, 48 separate phase probes, 26 executions and eight normalizations. Six positive port artifacts remain byte-identical; invalid-source execution is unrun. Nine bounded diagnostic gains and three unchanged negative controls, with both original sources preserved.

The 1,302-fixture census changes only the target originals' stderr, now exact upstream: 366 both accept, 380 exact rejections, 494 acceptance disagreements, 56 different rejections and six unchanged oracle-host failures. Historical 385 IO membership renews exactly; residual **165 = 109 acceptance + 56 rejection differences** outside it. No acceptance/runtime gain or oracle adjustment is inferred. Receipt linkage 484 / 818 is not whole-parity acceptance. Base coordinates, nat_first, compact lowering, external audit and other disagreements remain unwaived. Cache relocation preserved exact historical bytes; compact evidence committed, bulk evidence local.

## Diagnostic meet-precedence checkpoint

Tested source `9d13eba51d51b2de7f5ab415770c355f86fc0e58` changes only diagnostic quantitative-meet grouping to pinned precedence, preserving ordinary Term Display, internal error text, graphs, evaluation, acceptance and budgets. [Report and exact commands](cpu-meet-diagnostic-precedence.md). Full gate **1260 / 0 / 49**; focused receipts contain 112 successful test invocations including repeated controls. Eight bounded diagnostic gains and two retained negative controls; both originals unchanged. Seventeen printer cases separate diagnostic spelling from public legacy Display.

Matrix 272 commands, 16 generated executions, 8 pure normalizations, 180 exact negative comparisons and 40 separate phase probes. Four positive port artifacts match baseline bytes. No native IO coverage claimed; negative execution remains unrun.

Full census measures **366 both accept / 381 exact rejections / 494 acceptance disagreements / 55 different rejections / 6 oracle-host failures**; residual outside the exactly renewed historical IO cohort is **164**, with oracle failures retained separately. Detailed changes are in the reconciliation receipt; no acceptance gain or oracle waiver inferred. Receipt linkage 486 / 816 is not completed parity. Base coordinates, nat_first, unadopted compact lowering and external invoke audit remain outstanding. Public error status boundary is unchanged; diagnostic Debug text may differ. Bulk evidence remains local.

## Lambda rejection diagnostic checkpoint

Tested source `286741ec7c8bce954ef25de6e6ed5420acff44de` attaches optional context/source metadata only after the existing non-function-goal lambda refusal. Plain kernel error message/Display/equality, proof rejection, evaluation, sharing and budgets remain unchanged. No function-extensionality rule is introduced; the existing positive definitional eta control still passes. [Report and commands](cpu-function-proof-diagnostics.md). Full gate **1265 / 0 / 49**; final focused runs19+3 passed. Initial gate SIGABRT and synthetic-state attempts remain recorded separately; the failure-only helper restores the baseline recursive frame size without raising limits.

Matrix 320 commands: 24 generated executions, 12 pure normalizations, 198 exact negatives and 44 phase probes. Six port artifacts remain baseline-byte-identical. Nine bounded diagnostic gains and two unchanged refusals; three upstream originals preserved. No native IO claim; invalid-source execution remains unrun.

Census **366 both accept / 382 exact rejections / 494 acceptance disagreements / 54 different rejections / 6 oracle-host failures**; residual outside exactly renewed historical IO cohort **163**, oracle failures separate. Receipt linkage 489 / 813 is not full parity. New sourced CLI failures use existing Ok(CliOutput) with exit_code1 instead of prior host Err; derived Debug may change. External invoke callers remain unaudited. Base coordinates, nat_first, compact lowering and other disagreements remain outstanding. Compact receipts committed; bulk evidence local.

## Reserved-name loader diagnostic checkpoint

Tested source `de64abf0a7f4c844d2de1b0898af852f1af93d8d` matches nine original name-slot keyword diagnostics without changing rejection, cursor, public ParseError, terms or resource limits. [Report and exact commands](cpu-reserved-name-diagnostics.md). Gate **1270 passed / 0 failed / 49 ignored**; focused 69 passed and targeted strict Clippy passed. Matrix 522 commands; 249 deterministic repeated pairs; 32 generated C/JS executions; 16 pure normalizations; 342 exact negative stream comparisons; 76 negative phase probes; 8 baseline-byte-identical port artifacts. Separate 30-control observations retain lambda-keyword, term-keyword and earlier-depth differences; no new exception.

Renewed checking census: 366 both accept, 391 exact rejections, 494 acceptance disagreements, 45 different rejections, 6 oracle/host failures. Outside exactly renewed 385 historical IO members: **154** residuals, 109 acceptance disagreements, 45 different rejections; oracle failures separate. Linkage **499 linked / 803 unlinked** includes partial/blocked evidence and is not universal parity. Existing acceptance criteria remain. External invoke audit, Base coordinates, nat_first, compact lowering and supplementary lint failures remain outstanding. Source/compact receipts are preserved in Git; bulk raw evidence remains local.

## One-name lambda-binder diagnostic checkpoint

Tested `169934b2c9ceabfb27040d31442e6bbc838ebbd9` matches the existing non-name lambda refusal without changing acceptance, public ParseError, cursor or guards. [Report and commands](cpu-lambda-binder-diagnostics.md). Gate **1276 passed / 0 failed / 49 ignored**; focused 70 passed (three renewed after stronger lint-compliant assertions). Matrix 456 commands plus 58 adjacent observations retained losslessly. Uppercase original normalizes; C/JS emission correctly refuses missing Base and execution remains unrun. Template negative covers declaration loading, not helper specialization replay.

Census: 366 both accept, 392 exact rejection, 495 acceptance disagreement, 44 different rejection, 5 current oracle failures. Historical 385 IO cohort renewed exactly; current 154 residuals = 110 acceptance + 44 rejection. One port stderr fix; borrow_fork_hold oracle variation reintroduces one measurable unchanged port refusal. Neither a regression nor waiver. Linkage 500 linked / 802 unlinked is not full parity. External invoke audit/Base coordinates/nat_first/compact lowering/supplementary lints remain open. Disposable bytecode cleanup and directly packed new Git objects preserve source/evidence and unchanged floors; bulk raw evidence remains local.

## Import-header diagnostic checkpoint

Tested `ec91a1634d410781623961607b9cf85aab2e5f87` corrects four pinned load diagnostics without changing rejection,
public ParseError or safety contracts. [Report and commands](cpu-import-header-diagnostics.md).
Gate **1282 passed / 0 failed / 49 ignored**. The 364-command matrix, 48 adjacent observations and
24 direct original observations are retained. Three valid controls have 24 generated
C/JS executions; native pure normalization stays separate. Six adjacent differences
remain unwaived.

Renewed census: 366 both accept, 396 exact rejection, 494 acceptance disagreement,
40 different rejection, 6 oracle failures. Historical 385 IO cohort exactly renewed;
current 149 residuals = 109 acceptance + 40 rejection. Four port stderr fixes; the
additional count reduction is borrow_fork_hold upstream stack instability, not
improvement. Coverage 504 linked / 798 unlinked is linkage, not universal parity.
Prior semantic/API/resource/oracle limits remain open.

User-authorized verified archival recovered 1,287,000,064 workspace bytes while
preserving source and unique evidence; historical binary paths require explicit
restoration. All resource floors remain unchanged. Raw archives and pending
Library artifacts remain local.

## Missing-import diagnostic checkpoint

Tested `4d62ceac607c4050c1d53ded536feb837118bf89` corrects the five missing-import diagnostics, preserving public
ParseError and rejection/guard/evaluation behavior. [Report](cpu-missing-import-diagnostics.md).
Gate **1291 passed / 0 failed / 49 ignored**. The reviewed 452 observations retain 220 deterministic
within-lane pairs, 16 generated C/JS positive executions and eight separate pure
normalization observations. Four port artifacts match baseline bytes. Original
missing-file fixtures remain execution-unrun. Nine adjacent fallback cases remain
non-exact, including an upstream read-directory host failure.

Renewed 1,302 census: 366 both accept, 401 exact reject, 494 acceptance differences,
35 different rejections, six oracle failures. All 385 historical IO observations
renewed exactly; 144 residuals = 109 acceptance + 35 rejection. Only five port
stderr streams changed; no port status/phase changed. Upstream dead_param_fork
host-failure phase varied, with no classification improvement. No new exception.
Coverage 509 linked / 793 unlinked is linkage, not universal parity. All prior semantic,
API, resource and oracle caveats remain. The 21 cleanup restoration records retain
their hashes; no new cleanup or Library upload attempted.

## Malformed-literal diagnostic checkpoint

Tested `cfc326bcff94c1f7820e968c2abc6ca55270d643` corrects two original diagnostics: unknown escape and word
overflow, preserving public errors/guards/semantics. Hex-suffix and backtick lexical
originals remain non-exact; grammar context and first-error ordering require
further integration. [Report](cpu-malformed-literal-diagnostics.md). Gate **1298 passed / 0 failed / 49 ignored**.
Reviewed evidence: 598 observations / 290 within-lane pairs, 24 C/JS executions, 12 pure
normalizations, six unchanged artifacts; 46 public-error probes cover 23 cases twice.
Ten retained lexical/Unicode controls remain unqualified.

Renewed census: 366 both accept / 403 exact reject / 495 acceptance differences / 33 different
rejections / 5 oracle failures. Exact 385 historical IO renewal; 143 residuals = 110 + 33.
Only two port stderr fixes; no port status/phase changes. Upstream dead_param_fork became
accepted after a host-stack failure, adding one measured acceptance disagreement.
Linkage 513 linked / 789 unlinked includes two retained differences; it is not parity.

All 21 cleanup/restore records are unchanged. One historical successful focused raw
log was accidentally overwritten by a misrooted copied runner; published hash,
subsequent focused/full gate and failure receipts survive. The loss and exclusive
creation/basename protections are documented. No Library retry or new exception.

## Contextual lexical diagnostic checkpoint

Tested `c5edea8a75022c1654fbf568629422cf73bee1d0` closes the retained original diagnostics in `parse/hex_no_digits`
and `parse/unclosed_char_code`. Public lexical errors and rejection remain unchanged;
optional grammar context is bounded and stops before expansion/host work.
[Report](cpu-contextual-lexical-diagnostics.md). Gate **1304 passed / 0 failed / 49 ignored**.

Evidence: 734 observations / 358 deterministic pairs, 24 C/JS executions,
12 pure normalizations, six unchanged artifacts, 104 public-error observations.
Both originals were already linked; renewed exact qualification adds no inventory
entries. Unsupported prefixes and four Unicode controls remain unqualified.

Renewed census: 405 exact rejections / 366 both accepted / 494 acceptance
disagreements / 31 different rejections / 6 oracle failures. Exact 385 historical
IO renewal leaves 140 residuals = 109 + 31. Only two port stderr improvements;
three upstream-only stack/acceptance variations account for the additional net
count decrease. No semantic exception or resource-limit change was adopted.

Source/receipts are pushed separately from local raw evidence. Prior cleanup and
backup hashes remain intact; the old lost focused log remains unrecovered.
Library connection gate and prior API/semantic decisions remain unchanged.

## Uppercase Unicode scalar checkpoint

Tested `eedb452e718b10ce8b22a6e5fb42962706aa557b` adds pinned uppercase Unicode scalar escapes while preserving
unsupported forms, lowercase behavior and all guards. [Report](cpu-uppercase-unicode.md).
Gate **1310 passed / 0 failed / 49 ignored**.

The bounded matrix has 514 observations / 235 deterministic pairs, not 514 parity
passes. Thirteen pure controls newly check/normalize; the IO control now executes
natively but remains proof-excluded. Of 16 port C/JS executions against 16 oracle
executions, 14 match and two retain the existing C string-printer mismatch.
Non-scalar/lowercase acceptance and malformed-escape diagnostic differences stay
unqualified. No new semantic exception was adopted.

No census rerun: the last measured 140 non-IO residuals and six oracle failures
remain historical at c5edea8, not inferred current totals. Linkage is 514 / 1302,
including failures; 788 remain unlinked by their existing applicability category.
Next demonstrated independent gap: C escape printer round-trip spelling.
Prior API, scalar, resource and oracle caveats, cleanup records and lost-log
limitation remain explicit. No desktop activity or Library retry occurred.

## Executable C escape-printer checkpoint

Production `3d809d30e94e86e7a2b7eea772045c22dbb956d0` corrects braced hex spelling for controls, DEL and
non-scalar constructor values, matching the pinned C printer. Final tested source
`d36956122f5b2f27ae429c440ccef3028e13af23` adds only test-probe Windows stream setup; no desktop execution.
[Report](cpu-c-escape-printer.md). Final gate **1313 passed / 0 failed / 49 ignored**.

The 495-observation matrix has 167 deterministic pairs. All 64 corresponding
port C/JS observations match: 56 successes and eight retained JS non-scalar
refusals. Eleven scalar readbacks preserve full values; three earlier C scalar
readback failures are fixed. Final-source follow-up renews 51 outcome/artifact
observations, separately from the production-source matrix.

Four non-scalar literal readbacks, the 95-character input-depth aggregate and
all 15 native pure constructor-text differences remain unqualified. Strict IO
checking exclusion remains distinct from actual raw-IO execution. No new exception.
No census rerun: 140 non-IO residuals and six oracle failures remain historical.
Linkage is 516 / 1302, including failures; 786 unlinked:
256 checker rejection, 517 interpreter/compiled candidates, 12 interpreter-only
and one no-main module. These categories are not all missing runtime tests.

Source and compact receipts are in Git; raw evidence is locally backed up.
Prior cleanup records, old log-loss limitation, blocked Library upload and all
pending semantic/API/resource/oracle decisions remain explicit. Next substantive
gap is non-scalar C-output literal readback; its scalar policy needs investigation
against pinned Base/lexer contracts before adopting a new interpretation.

## U32-backed literal readback checkpoint

Tested `087a646a33cac73307fd4adf3135fc3186553b53` fixes canonical non-scalar literal acceptance using U32 data,
matching pinned Base/lexer contracts. Malformed syntax and all host/resource
boundaries remain enforced; no new semantic decision or exception.
[Report](cpu-nonscalar-literal.md). Gate **1317 passed / 0 failed / 49 ignored**.

The 44-control matrix records 890 observations / 210 deterministic pairs, not
890 parity passes. All 120 corresponding C/JS executions match: 78 successes,
42 retained JS scalar refusals. All 23 canonical C-output readbacks accept and
preserve full values. Forty comparable historical cases show 18 checking
acceptance fixes and 19 native-status improvements (18 pure, one actual IO).
Four later controls have no fabricated baseline.

Current subset: 28 both accepted, two exact rejections, ten differing rejections,
four acceptance differences (two strict-IO exclusions and two resource guards).
Eight extra upstream guard executions do not qualify unrun port executions.
Three upstream originals plus four authored controls separately renew seven
native pure outputs and 14 identical artifacts; that receipt is emission evidence.

No full census rerun; the historical 140 non-IO residuals and six oracle failures
remain historical, not inferred current totals. Linkage stays 516/1302, including
failures; 786 unlinked entries retain their applicability categories. Next
independent gap: malformed uppercase-escape diagnostics. Pending nat_first,
compact lowering, Base coordinates, external invoke audit, supplementary lints
and other semantic/oracle limits remain explicit. Baseline executable retention
and the older lost-log limitations are recorded. Prior backups remain intact;
no desktop activity or Library retry occurred.

## Malformed uppercase escape checkpoint

Tested `a28efb73ec5d78cf8f0125854799c19c51c9163a` qualifies the existing escape-diagnostic boundary
without changing decoding, acceptance, runtime or guards. [Report](cpu-uppercase-escape-diagnostics.md).
Gate **1319 passed / 0 failed / 49 ignored**.

The 39 authored negatives improve from five exact/34 differing diagnostics to
28 exact/11 differing: 23 improvements, with no previously exact regression.
A byte-identical pinned unknown-escape original also remains exact. The matrix
records 605 observations/104 repeat pairs, not 605 parity passes. Six accepted
controls have 24 exact corresponding C/JS runs (16 success/eight JS refusal),
five unchanged native pure outputs, one unchanged native IO output and 12
unchanged artifacts. Strict IO exclusion remains.

Whole-census counts are historical and unchanged as measurements; linkage stays
516/1302, including failures. Eleven authored diagnostic differences remain open.
The next substantive gap is lowercase plus/nine-digit scalar escape acceptance:
eight renewed checking/normalization observations confirm disagreement; no
executable qualification or correction is claimed. Pending nat_first, compact
lowering, Base coordinates, external invoke audit, supplementary lints and older
resource/oracle gaps remain unwaived. Public invoke result-shape changes in both
directions are explicit. Prior cleanup records/backups remain intact, historical
log loss unrecovered, no Library retry or desktop activity.

## Lowercase Unicode escape grammar checkpoint

Tested `b46f9570a2174b6f475b4f58836d960534a967a8` rejects leading-plus and excess-digit lowercase scalar escapes
according to the pinned grammar, preserving canonical U32 codes and all safety
limits. [Report](cpu-lowercase-escape-grammar.md). Fresh standard gate:
**1322 passed / 0 failed / 49 ignored**. The earlier user-interrupted
402/0/2 receipt remains incomplete and unchanged.

The 34 authored negative controls now all reject: 19 exact / 15 differing
diagnostics, closing 14 acceptance gaps and three diagnostic gaps. The three
overlapping uppercase controls are the same closures, not additional gains.
676 matrix observations contain 106 repeat pairs; nine valid controls have
36 corresponding C/JS matches (28 successes/eight required scalar refusals),
eight unchanged native pure outputs, one unchanged native IO output and 18
unchanged artifacts. Pure baseline equality is not full upstream graph equality.

No full census renewal: historical counts and 516/1302 coverage linkage remain
historical, including failed receipts. All pending semantic/resource/oracle/API
issues remain unwaived. First-error and invoke result-shape caveats are explicit.
Source bundle and 150 evidence files verified on resumption; unique evidence and
cleanup records remain intact. Work is local only, pending publication approval.
Next independent candidate: existing lowercase malformed-escape diagnostics.

## Lowercase malformed-escape diagnostic checkpoint

Tested `d79486efef4cb6baf6d94515c9ee764b2fedeb77` adds private per-escape provenance while preserving all raw
public errors, rejection and value semantics. [Report](cpu-lowercase-escape-diagnostics.md).
Gate **1324 passed / 0 failed / 49 ignored**. The 30-case authored
cohort improves from 3 exact/27 differing to 26 exact/4 differing: 23 diagnostic
closures, no acceptance change. Thirteen overlapping older grammar cases now
match; they are not extra closures. Historical measurements remain unchanged.

620 observations/98 repeat pairs; all 60 raw loader observations unchanged;
36 corresponding C/JS matches (28 successes/eight required JS refusals), eight
unchanged native pure outputs, one native IO output and 18 identical artifacts.
Pure baseline equality is not full upstream normalized-graph equality. Four
diagnostic residuals, strict IO exclusion and all older parity/API/oracle gaps
remain explicit. No whole-census renewal or safety/policy change.

The previous reviewed c99e74a checkpoint was independently consolidated onto
main in a separate verified fast-forward. This diagnostic work retains its
own tested identity. No branch deletion, force push, dependency change or release.
Next candidate: later-character closing-quote diagnostics with grammar/lexer
precedence preserved. Library delivery remains blocked without retry.

## Character closing-quote diagnostic checkpoint

Tested `6c26f207626582deb04be75885325cfc9389197d` preserves raw parser errors and rejection while reporting the pinned
first-code closing-quote failure before malformed later escapes.
[Report](cpu-character-quote-diagnostics.md). Gate **1327 passed / 0 failed /
49 ignored**. The 31-negative cohort measures 5 exact / 26 different before and 25 exact /
six different after: 20 diagnostic closures, no acceptance change. Six closures overlap
older cohorts and are not counted again. Two unchanged earlier-grammar probes match
outside the 23 mandatory exact flags; the compact receipt explicitly corrects their
conservative matrix labels while preserving every raw observation.

751 observations / 122 repeat pairs;62 unchanged raw loader outputs;56 corresponding C/JS
matches (48 successes/eight scalar refusals),13 unchanged pure native outputs, one native
IO output,28 identical artifacts. No full upstream normalized-graph or whole-census
renewal. Six current residuals and all earlier semantic/API/host/oracle gaps remain
unwaived. Dependency/toolchain/pin/budget/guard/cancellation policies unchanged.
Next bounded diagnostic candidate: empty Char `''` contextual closing-quote reporting;
first confirm upstream cursor behavior without changing lexer acceptance or raw errors.

## Raw apostrophe compatibility checkpoint

Tested `61cab3744713e4841f44efd8b8bd3853e1026206` repairs raw apostrophe tokenization while preserving
template replay and existing guards. [Report](cpu-raw-apostrophe.md). Standard gate
**1334 passed / 0 failed / 49 ignored**. Authored cohort: 12 accepted/17 refused;
ten former load refusals now execute across corresponding C/JS backends. 513 observations,
106 repeat pairs, 96 successful matching generated executions. Eleven complete raw/escaped
value-graph pairs and one native IO pair pass; these are not full upstream graph proofs.

Fifteen negative raw errors unchanged; two token/import error changes disclosed.
Three exact upstream rejection diagnostics and 14 retained differences; strict IO
proof exclusion remains. No whole-census renewal or original-linkage increment.
All earlier semantic/API/host/oracle/resource/lint gaps remain unwaived. Main's previous
restriction was explicitly resolved for the separate reviewed 878 checkpoint.
This work uses reviewed feature preservation and the approved non-force main workflow.
Next bounded gap: contextual empty-character and quote-count diagnostics.


## Contextual empty-character diagnostic checkpoint

Tested `15568847ae21f8e917024edc0f39181fe21396dc` adds failure-only original-source attribution while
preserving acceptance, raw parser errors, evaluation and guards. [Report](cpu-empty-character-context.md).
Gate **1338 passed / 0 failed / 49 ignored**. The 22-negative cohort improves
from 2 exact/20 different to 18 exact/4 different; five executable controls remain accepted.
432 observations, 74 repeat pairs, 20 exact backend comparisons across 40 executions;
44 raw errors unchanged and ten generated artifacts identical. Strict IO proof exclusion remains.

Fresh overlapping cohorts measure 26/5 character-quote and 8/9 raw-apostrophe
rejection diagnostics; overlaps are not additional fixes. No whole-census or original
linkage update. Four residual diagnostics and all prior semantic/API/oracle/resource
gaps remain unwaived. CRLF attribution errors were corrected before the clean-source
matrix/full gate; historical receipts remain explicit. The earlier denied main update was explicitly reapproved, completed and verified.
This newer checkpoint follows independent review and the authorized non-force workflow.

## Quote and grammar diagnostic checkpoint

Tested `b383255b9b584408cb61927e7cd4b34567d78f99` closes three retained diagnostic gaps without changing acceptance,
raw loader errors, evaluation or guards. [Report](cpu-quote-grammar-context.md).
Standard gate **1342 passed / 0 failed / 49 ignored**; matrix **600 observations**,
98 repeat pairs, 40 generated executions and 68 unchanged negative raw API outputs.
The original cohort is 21 exact/1 different; twelve adjacent controls are exact.
`four-quotes` remains unqualified pending safe isolated diagnostic recovery.
Historical census/linkage and unwaived semantic, oracle, resource and external API
gaps remain separate. No second parser pass, new exception or raised budget.

## Bounded four-quote diagnostic recovery

Tested `02155ff3c30c6bcab683abb1fd8d0e73422acb1d` closes the remaining original four-quotes diagnostic and two
adjacent contexts through isolated, capped failure-only recovery. Raw errors,
acceptance, runtime behavior and existing language guards remain unchanged.
[Report](cpu-four-quote-recovery.md). Full gate **1350 passed / 0 failed / 49 ignored**;
**712 matrix observations / 114 repeat pairs / 0 timeouts**, 40 C/JS executions
and 84 unchanged negative raw API results. Original cohort 22/0, preceding
grammar cohort 12/0; new context cohort 5/3. Argument/list incomplete-atom recovery
and depth refusal remain different, unwaived. Whole census was not renewed; all
previous semantic/oracle/API limits remain open.

## Terminal incomplete-character metadata

Tested `1a135e437d91c99072bbd880b9a6d43ceb984d2a` qualifies the unchanged list context through a terminal
metadata-only refusal. The original argument remains blocked by prior-function
lowering; the depth refusal is unwaived. [Report](cpu-incomplete-character.md).
Gate **1354 passed / 0 failed / 49 ignored**; **908 observations / 142 repeat pairs**,
40 generated C/JS executions and 112 unchanged negative raw API results. Seven
diagnostic closures span one prior context, four new controls and two older
cases; no acceptance change or whole-census gain is inferred.

The 31 historical rejection differences were renewed and all remain different.
The full 1,302-original census and 109-row acceptance classification remain
historical. [Reconciled gaps](cpu-incomplete-character-gap-inventory.json) separate
proof-mode preemptions, resource refusals, missing hosts and unqualified runtimes.
Next substantive batch is typed array fill/fold and polymorphic elements, with
fresh baseline native success but generated/runtime-neighbor qualification pending.
All prior semantic, resource, oracle, cancellation and external API limits remain.


## Typed array execution checkpoint (d1ba0e5)

The unchanged array_fill_fold, array_poly_elem, array_keep_cow and array_copy_pair
originals now match all five executable lanes. Four new boundary/ownership
controls also match; three checker negatives reject exactly. The depth32 port-C
resource diagnostic remains unwaived. Final matrix: 198 observations / 48 repeat
pairs / 0 timeouts; standard gate: 1357 passed / 0 failed / 49 ignored. No production
change, full census or rejection-subset renewal occurred. Historical 109 acceptance
and 31 rejection disagreements retain their original source identities. Linked
receipts: 520/1302, not completed parity. Argument recovery remains
blocked by its existing lowering barrier. Next substantive original family is
array_borrow_generic/array_borrow_generic_sum, unrun here. See
[typed array runtime report](cpu-array-runtime.md) for exact profiles, commands,
counts, limits and preservation.
