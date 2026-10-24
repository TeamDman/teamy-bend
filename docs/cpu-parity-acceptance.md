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
