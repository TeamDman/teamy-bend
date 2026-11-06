# CPU rewrite progress and remaining work — 2026-11-06

This report concerns the pinned Bend 2.0.5 reference
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. The port is an experimental
supported subset, not a drop-in replacement. It does not estimate a completion
percentage: fixture links and tests have no common weighted denominator for
language features, runtimes, host interfaces and the separate Poche goal.
The former 55–65% estimate was withdrawn.

## Implemented and verified boundaries

The port has parsing, proof checking and pure normalization; executable checking
is deliberately separate from strict proof checking. Native CPU execution and
generated C/JavaScript implement selected pure and effectful programs, including
console IO, tasks, timers, channels, environment, files and TCP/UDP. Generated C
also has a bounded CPU worker pool. These statements describe capabilities,
not complete compatibility. [Compatibility](compatibility.md) and the
[implementation plan](implementation-plan.md) give the contracts and limits.

The preserved nested-array checkpoint `ea4cc7d` covers two unchanged upstream
originals, owner-transfer and split/rejoin controls, and affine/Data negatives.
Its focused array suite passed 12 tests and strict targeted Clippy passed. The
fresh final-source matrix completed 132 observations, with zero timeouts. The
statuses and both streams of 120 path-independent observations, plus all 24
generated-artifact hashes, match the preserved baseline. Fifty successful
executions, 18 checker-negative observations and ten runtime refusals remain
distinct. At that checkpoint the port-C array-depth diagnostic was
one unwaived discrepancy; matching refusals are not proof of compatibility.
The two originals returned 135 and 145896, respectively, in all five executable
lanes twice. These controls do not prove general memory safety, physical sharing
or destruction timing. See the [preserved checkpoint](https://github.com/TeamDman/teamy-bend/blob/ea4cc7d2fd9e4e2f028cf3602499da3a93a214da/docs/cpu-pause-20261026.md).

The standard gate at `ea4cc7d` passed formatting, strict Clippy, all-feature
build and the real Node TCP/UDP provider checks, then hit its unchanged
1,200-second outer deadline. The completed test groups contained 941 passed,
zero failed and 48 ignored; another group was interrupted, and remaining tests
were unrun in that attempt. It is not a passing full gate. The source-bound
matrix used binary SHA-256 `93ab25d9ece882309e3485ce5e74acfba19ec91c3245a8867a8f5d41f36789b1`;
the raw matrix receipt SHA-256 was `8722a92794b0cd330b50e52bd267eef98b3184a4bc1f6e4da0fe01fad935ee52`.
A separate complete test run on the same clean source and test feature set
passed **1,366 / 0 / 49 ignored**: all 197 Cargo integration targets in two
bounded batches, plus explicit library, binary and documentation tests. These
batches supplement the successful pre-test gate stages; they do not turn the
interrupted single-command gate into a pass. Results, hashes, selection and
limits are recorded in [the validation receipt](cpu-resumption-20261106-validation.json).

The later NaN-payload change was separately validated at `c3a775c` with the
complete test set in bounded batches: **1,371 passed / 0 failed / 49 ignored**.
Its standard gate timed out at 1,200 seconds after 936 completed passes, zero
failures and 48 ignores; the interrupted group and subsequent groups were not
completed by that run. See [NaN validation](cpu-float-nan-overflow.md). The
subsequent string-normalization investigation changed only research fixtures,
a standalone probe and reports, not production code.

The subsequent [core String and Char rendering change](cpu-core-string-printing.md)
addresses a native pure-output mismatch while preserving the normalized value
graphs. Alongside iterative substitution and explicit release, its final-source
complete offline test set passed in bounded batches: **1,382 passed / 0 failed /
49 ignored**, covering all 200 integration targets plus library, binary and
documentation tests. This is separate from an uninterrupted standard gate. The
string-pattern depth issue remains unresolved.
The final-source standard gate passed formatting, strict production Clippy,
build and Node provider checks, then timed out at 1,200 seconds with 952
passed, zero failed and 48 ignored in completed groups; its active group was
interrupted and subsequent groups were unrun.

The subsequent [iterative nullary fallback change](cpu-pattern-stack.md)
replaces recursive fallback calls with explicit frames and protects suspended
partial arms on covered failure/unwind paths. It retains the existing guards;
the original `"hi"` source still fails at the 64-level pattern guard. Other
compiler/kernel recursion and ordinary public term destruction remain open.
See its report for final-source validation and precise limits.

The subsequent [field-bearing branch change](cpu-field-stack.md) schedules
positive and fallback compiler calls with explicit frames and cleans up suspended
arms on covered failures. The original `"hi"` witness still refuses at the
64-level compilation guard. Pattern/body cloning and destruction, other
compiler recursion, public term ownership and kernel recursion remain blockers
to safely retiring the guard; the kernel's 128-level boundary is separate.

The [stack-safety path map](cpu-stack-path-map.md) distinguishes the work
needed to retire the compiler guard from the separate input/kernel boundary.
A reduced automatic-destruction model handles deep shared cleanup, but adding
`Drop` to the public `Term` enum breaks existing owned-field matches; no
production ownership change or guard removal was claimed at that point.

The subsequent [private pattern-tree change](cpu-pattern-tree-stack.md)
converts parser pattern traversal and cloning to explicit work and makes private
pattern destruction automatic and iterative. The unchanged `"hi"` witness
still refuses at 64 levels; public term destruction and other recursive surface
and kernel paths remain open.

The subsequent [private body traversal and cleanup change](cpu-body-stack.md)
makes body cloning, substitution and reusable-binding traversal iterative, and
cleans private body-owned graphs automatically. The original `"hi"` witness
still refuses at 64 levels; public `Rc<Term>` last-owner drop and other
recursive compiler and kernel paths remained open at that point.

The subsequent [automatic core-term cleanup](cpu-term-ownership.md) adds an
iterative `Term` destructor, preserving `Rc`/`Weak` sharing while intentionally
changing Rust owned-field destructuring. It handles ordinary last-owner,
early-error and unwind cleanup in focused small-stack tests. Work-list
allocation failure aborts the process rather than yielding a Bend resource
error; dedicated isolated tests inject failure both normally and during unwind.
The original `"hi"` source still fails at the 64-level pattern guard. Remaining
compiler recursion and the separate input and kernel limits are still open. Its
complete bounded test set passed 1,417 tests, with zero failures and 49
ignored. The standard uninterrupted gate was not rerun; the prior timeout
remains incomplete.

The subsequent [match-continuation change](cpu-match-continuations.md) removes
Rust recursion from variable-only columns and skipped binders. It preserves the
64-level compiler guard, so the unchanged `"hi"` witness still refuses there;
nested-body recursion and the input/kernel boundaries remain. Its focused and
before/after evidence is recorded separately from the prior complete test set.

The subsequent [unified body/match driver](cpu-unified-body-match.md) replaces
the compiler's nested body-to-match calls with explicit continuations and moves
parsed bodies into those frames. The original `"hi"` still reaches the same
64-level compilation guard. Retiring that guard requires a shared work and
allocation policy covering body mapping, pattern copying and term substitution;
the distinct input and kernel limits remain afterward. The complete bounded
test set passed 1,420 tests, with zero failures and 49 ignored; the uninterrupted
standard gate was not rerun and the prior timeout remains incomplete.

The subsequent [iterative qualification and presentation change](cpu-iterative-presentation.md)
removes host recursion from operator qualification and core-term Display and
compact Debug. A no-guard experiment moved the original `"hi"` witness to the
kernel input guard and ran `nat65`, but exposed a 2 MiB checker stack overflow
on an existing 122-level Nat pattern. The port's 64-level compiler guard remains
because of that measured host-safety blocker; it is not an upstream language
limit. The report records growth measurements and the remaining kernel work.

## Inventory by applicable mode

The inventory was regenerated from the clean pinned reference: all 1,302 source
and expectation rows reproduce the published inventory SHA-256
`04378ce0661f27c819cbe99860b7f11539b634e40308a6a8d14f86e85c038fb0`.
This is a source census, not an execution run. Categories reflect the pinned
upstream gate's candidate selection, not guaranteed printability or successful
execution. An interpreter-only case can still contain executable concepts; a
candidate lane can be blocked by foreign companions or host requirements.

| Gate classification | Fixtures | Mixed detailed receipt links | No such link |
| --- | ---: | ---: | ---: |
| Expected checker rejection | 439 | 183 | 256 |
| Checking without main | 12 | 11 | 1 |
| Interpretation only | 164 | 152 | 12 |
| Interpretation and compiled candidates | 687 | 178 | 509 |
| Total | 1,302 | 524 | 778 |

The 687 compiled candidates include 687 JS and 671 C candidates by the gate's
source-level selection rules. The 524 links are the union of the 36 evidence
files listed in [the gap inventory](cpu-incomplete-character-gap-inventory.json)
and the three subsequent array evidence files, joined by fixture name and exact
source hash. A link can describe checking, normalization, a passing execution,
a refusal or an incomplete result. It is not a pass count and its absence does
not establish an implementation defect. Rows can have multiple links. The
checkpoint's two original array fixtures are additional to this historical
524; their new matrix is summarized above and is not silently added to the
older linked ledger.

The source scan also flags 11 filesystem, 14 network, 11 clock/timer, four
environment and 20 foreign-companion fixtures, plus one window and one audio
fixture and 13 offload-review fixtures. These counts overlap and are textual
hints only; they do not establish reachability, successful execution or full
API coverage.

The separate historical [runtime ledger](cpu-runtime-evidence.csv) has 112
original rows at their recorded source revisions. It records 110 native, 109 C
and 111 JS port `pass` labels, alongside explicit foreign, safety-limit or
inapplicable results; upstream C/JS have their own unavailable and failed
lanes. These labels have individual receipt scopes and do not cover the full
corpus or establish current-head runtime parity. The 206-row interpretation
ledger and checking/other evidence overlap these rows and cannot be added to
obtain execution coverage. Platform/API references in
[cpu-applicability.json](cpu-applicability.json) are source hints, not proof
of reachability or lane qualification.

The last whole checking comparison remains historical at `c5edea8`: 366 both
accepted, 405 exact rejections, 494 acceptance disagreements, 31 different
rejections and six upstream stack failures. Outside the fixed 385-row IO cohort,
109 acceptance and 31 rejection disagreements remained. These are not a
current residual count or runtime results; strict-proof and executable
assumptions differ. See [the gap inventory](cpu-incomplete-character-gap-inventory.json).

## Remaining work, ordered by dependency

1. **Correctness of known CPU differences.** The depth-32 generated-C
   diagnostic was subsequently corrected without widening allocation limits;
   see [array class-limit validation](cpu-array-class-limit.md). Investigate
   remaining array/resource boundaries. Resolve argument-recovery lowering,
   Base source locations, source holes/TODO behavior and diagnostic provenance. `nat_first`
   has a documented pinned C versus JS error-order conflict; a narrow exception
   remains a decision, not an assumed approval. The Linux x86-64 glibc native
   NaN-payload overflow mismatch was corrected; see [NaN payload validation](cpu-float-nan-overflow.md).
   F32.read still has documented backend-specific differences, including those
   payload spellings and the earlier three cases, so no uniform rule is inferred.
2. **Representation and normalization.** Compact Nat and string-pattern
   lowering needs typed continuation construction and canonical normalization,
   preserving open terms, demand, error order, resource accounting and existing
   guards. The [prototype](cpu-compact-patterns.md) is research, not support for
   the original source. A [reduced normalization investigation](cpu-string-normalization-boundary.md)
   separates the port's pattern limit, kernel normalization boundary and data
   output limit; the original pattern is still refused and no repair is claimed.
   A [bounded shared-DAG prototype](cpu-string-dag-summary-attempt.md) adds
   path-depth, occurrence-budget and first-error adversaries; it does not make
   the expanded string matcher acceptable or implement production lowering.
   General large native Nat computation and cancellation
   boundaries remain incomplete or unmeasured.
   The user has directed that the depth thresholds be treated as temporary
   implementation constraints. Removing them safely requires iterative
   traversal and stack-safe cleanup across every consumer while retaining
   meaningful work and memory safeguards; changing one guard alone is unsafe.
   [Iterative substitution and explicit release](cpu-stack-safe-substitution.md)
   provide initial stack-safe paths; they do not yet remove any end-to-end
   depth guard or make ordinary `Rc<Term>` destruction stack-safe.
   [Automatic partial-result and memo cleanup](cpu-stack-auto-cleanup.md)
   extends stack-safe cleanup to selected kernel failure and unwind paths;
   other term owners and the recursive compiler/kernel paths remain open.
   Its complete final-source offline test set passed in bounded batches:
   **1,389 passed / 0 failed / 49 ignored**, covering 200 integration targets
   plus library, binary and documentation tests. This is not a standard-gate pass.
   Its final-source standard gate timed out at 1,200 seconds after 959 passed,
   zero failed and 48 ignored in completed groups; the active group was
   interrupted and later groups were unrun.
3. **Runtime and library breadth.** Systematically measure the unlinked corpus
   in each applicable lane, and reproduce failures before repair. Exercise
   concurrency under multiple worker profiles, cancellation, ownership and
   resource exhaustion. Arbitrary native foreign callbacks remain unsupported;
   host/oracle limitations, window/audio effects, and whole-library behavior
   need their own qualification. Base is mode-dependent. A lexical recount finds
   367 top-level type/law/def forms in strict Base and another 95 in its execution-only overlay, versus
   467 in upstream Base. Names overlap and some forms have different contracts;
   these counts are not a missing-feature count or proof of equivalence.
   A top-level name comparison finds 15 upstream names absent from both
   source files: six window-dependent `App` helpers, five `Window` names,
   and four `Audio` names. This is a lexical inventory, not an API signature
   or implementation audit. Intrinsics and foreign interfaces need separate
   review. Upstream gate candidate counts do not include all API uses.
4. **Interfaces beyond the corpus.** Audit external library consumers and CLI
   compatibility, including callers that must inspect `CliOutput.exit_code()`
   even when `invoke` returns `Ok`. Audit emitters and callbacks, imports,
   packaging and host behavior; local repository callers are not an external
   compatibility survey. Hub fetch/publish remains unfinished.
5. **Hardware and integration.** Fresh Windows/macOS host validation, CUDA/GPU
   execution and sanitizer qualification, other GPU targets, and broader Poche
   formalization/integration are separate from this Linux CPU run. Existing
   hardware receipts retain their exact historical revisions; ignored tests are
   not passes. Supplementary all-target lint failures also remain unqualified.

The size of each item is uncertain because uncovered cases can share a root
cause or reveal new contracts; no defensible time or completion percentage can
be derived from the current census. Preserve the pinned upstream, toolchains,
dependencies, safety budgets and negative controls while pursuing these items.
