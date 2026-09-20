# Bend Rust rewrite and Poche formalization

**Plan status:** Active
**Primary implementation root:** `teamy-bend` repository
**Last updated:** 2026-09-20
**Intent audit:** Passed 2026-09-19 against the initiating user request

## How to update this plan

- `[ ]` Not started; `[~]` In progress; `[x]` Complete; `[!]` Blocked.
- Update each task heading and its completion evidence together. A phase is
  complete only when all its work is complete.
- Preserve the requirements ledger. Do not convert a first supported subset
  into a claim that the entire requested rewrite is complete.
- Parallel tracks: kernel agent owns `src/kernel`; parser agent owns
  `src/syntax`; Poche agent owns the target project's Bend integration;
  root owns CLI, repository setup, validation and publication.

## Authoritative guidance ledger

| ID | User requirement | Coverage |
| --- | --- | --- |
| U1 | Find local `bend` or `bend2` under the named repository root. | 1.1 |
| U2 | Create a new repo named `teamy-bend` using `gh`. | 1.2, 5.1 |
| U3 | Make the new repo public. | 1.2, 5.1 |
| U4 | Use MPL-2.0 if compatible with Bend's licence. | 1.3 |
| U5 | Use `teamy-rust-cli` as the starting template. | 1.4 |
| U6 | Rewrite Bend in Rust. | 2.1–3.3, full compatibility remains open |
| U7 | Then use the rewrite to formalize the user's Poche4 repo. | 4.1–4.3 |
| U8 | Locate Poche4 at the approximate older games-repository path. | 1.1 |
| U9 | Set an active goal for this work. | Goal tool, done |

## Intent audit evidence

- Extraction: reread the original compound request; recorded naming, gh,
  visibility, conditional licence, template, rewrite, order, approximate paths
  and goal separately in U1–U9.
- Traceability: each requirement maps to a task and observable acceptance.
  U6 includes runtime/compiler work beyond the first proof-checking slice.
- Adversarial omission: retained the licence condition, uncertainty in repo
  names, and the requirement to use our implementation in Poche. Existing
  Poche formalization is foundation, not grounds to omit the integration.
- Known source limitation: none.

## Verified foundation and references

- Bend reference: `bendlang/bend`, revision
  `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, Bend 2.0.5. This is the dependent,
  affine proof language, not Bend 1/HVM. Source remains read-only.
- Relevant upstream files: `bend2/bend.ts` (kernel and parser), `bend2/comp.ts`
  (compiler/runtime), `bend2/main.ts`, `bend2/base.bend`, and `tests/`.
- Poche target: `TeamDman/Poche`, discovered directory name `poche-4`, branch
  `spacetimedb`, initial revision `97558e0`. Existing independent Rust, Alloy,
  NuSMV, Prolog and Weavy models provide conformance oracles and rule IDs.
- Machine-dependent paths stay outside published source. `<bend-reference>`,
  `<template-root>` and `<poche-root>` denote the user's existing checkouts;
  `<poche-root>` is the corrected discovery for the approximate user path.
  These placeholders preserve purpose without publishing drive layout.
- Reference upstream warns of kernel/Lean differences. Testing this Rust port
  does not establish metatheoretic soundness of either checker.

## Decisions and acceptance consequences

| Question | Decision | Acceptance |
| --- | --- | --- |
| Licensing | MPL new code; Apache translated upstream files with notices | Inspect staged SPDX headers and full licences |
| Scaffold | Reviewed tracked working files only | No untracked binaries/captures copied |
| First execution target | Native Rust proof checker and pure evaluator | Works without a TypeScript interpreter |
| Incomplete/unsupported inputs | Fail closed | Negative tests for holes, open laws, invalid proof, unsupported syntax |
| Poche correspondence | Independent Bend rule kernels, then phase model | Exhaustive declared finite input comparison and negative controls |
| Poche batch interface | Nat rows in JSON; one parse/check per invocation | `batch FILE --entry NAME --args-json PATH` returns `results` |
| Broader parity | Still required by U6 | Explicit compatibility inventory; no completion on subset alone |

## 1. Repository foundation

### [x] 1.1 Identify the existing reference and Poche checkout

Completion: confirmed the remote/revision/licence and clean state of both.
Poche is a 31-crate Rust workspace with established formalization rules in
`CONTRIBUTING.md`. Its existing micro model is two seats, two suits, three ranks,
six cards, schedule 1/2/1; published existing evidence reports 431,800 states.
No existing checkout was fetched, pulled or rewritten.

### [x] 1.2 Create the named public remote using gh

Completion: `gh repo create TeamDman/teamy-bend --public` succeeded at
https://github.com/TeamDman/teamy-bend. Local main branch is initialized with
the user's public pseudonym and GitHub noreply identity. First implementation
commit `2e540f9` was pushed to public `main`.

### [x] 1.3 Establish compatible licensing and provenance

Completion: upstream Apache-2.0 text preserved in `licenses/Apache-2.0.txt`;
template MPL-2.0 text retained as `LICENSE`; attribution in `NOTICE`.
Authoritative references: [Mozilla FAQ Q13](https://www.mozilla.org/en-US/MPL/2.0/FAQ/#q13-may-i-combine-mpl-licensed-code-and-bsd-licensed-code-in-the-same-executable-program-what-about-apache)
and [Apache licence section 4](https://www.apache.org/licenses/LICENSE-2.0).
Translated files keep Apache licensing; new files use MPL.

### [x] 1.4 Adapt and validate the Rust CLI template

Completion notes: identity/env names, CLI commands, profiler default and README
adapted. Logging, structured output, cancellation, Windows resources and CLI
fuzzing retained. The latest `check-all.ps1` passes 100 tests, formatting,
Clippy and all-feature compilation; two opt-in local profilers are ignored.
The template/library slice increases the gate to 124 passing tests and adds
native `base`, `base NAME` and `base --types` source browsing.
Help/version and native command checks passed.

Work: replace identity, CLI commands, env names, examples and profiler defaults;
retain logging/output/cancellation/build-resource and fuzz infrastructure.
The template initializer was inspected: it traverses untracked downloads and
captures. Used its tracked working-tree source selection directly to avoid
copying unrelated artifacts. Reference source and local changes remain intact.
Validation: `cargo run -- --help`, `cargo run -- --version`, `./check-all.ps1`.
Completion: command surface describes the new project and quality gate passes.

## 2. Proof language

### [~] 2.1 Port the trusted core and pure evaluator

Completion notes: dependent affine core, structural descent, kinds, equality
rewrite, normalization and strict complete-book checking implemented. 18 kernel
regressions include false proofs, erased evidence, resource use, invalid rewrite,
hidden holes and resource limits. Full parity remains open; large terms are
bounded explicitly. `examples/induction.bend` passes both this and upstream
checkers and proves Nat right-zero by induction.

Work: represent terms/books, reduction, definitional equality, bidirectional
checking, kinds/quantities, affine use, ADT validity, structural termination,
law filling and equality elimination. Unsupported features reject explicitly.
Validation: focused kernel tests, including false reflexivity, invalid rewrite,
erased evidence escaping, affine duplication and non-descending recursion.
Completion: faithful supported behavior with honest feature inventory; no
unsafe or incomplete proof can produce a successful check report.

### [~] 2.2 Port parsing and module loading

Completion notes: source declarations, nested patterns, quantities, literals,
operators, do/array sugar and local imports implemented; 18 parser tests pass.
Empty datatypes, reusable parallel lets and implicit array-write rebinding are
covered. Foreign bodies, GPU calls and hub packages remain open.
Exact supported surface/limits: `docs/compatibility.md`.

Closed compile-time template specialization now passes 13 focused tests:
capture rejection, erased argument checking, inlined lambda resource use,
declaration/import scope, alpha-canonical structural cache keys and bounded
instance growth. Typed local matcher lambdas now lower with their own binders.
No kernel acceptance rule changed. A combined recursive/shadowing/duplication
fixture agrees with independent expected values in the normalizer, lazy native
runtime, JavaScript and compiled C. Both upstream and this port reject
unannotated constructor arguments exposed by macro beta-reduction; annotated
or typed-parameter forms work. Unused template bodies are parsed, while
their type/resource checks run when instantiated, matching upstream.

Work: parse Bend syntax and declarations, namespace imports and desugaring
against upstream sources. Keep diagnostics useful on native Windows.
Validation: parser tests and real `.bend` fixtures covering accepted and
rejected inputs, nested binders, patterns, imports and malformed files.
Completion: supported upstream programs load into the Rust kernel; unsupported
constructs fail without silently changing semantics.

### [~] 2.3 Establish differential compatibility

Completion notes: `examples/audit_upstream.rs` ran all 1,302 reference fixtures
in isolated native processes. First milestone acceptance: 302 positive cases
accepted, 551 positive cases rejected, 449 negative cases rejected; zero crashes
and zero accepted negative cases. This is checking acceptance only; exact error
messages, values, runtime behavior and performance remain unverified by this
audit. Raw path-containing output is ignored under `target/`.

Next-slice audit: 344 positives checked, 509 positives rejected, all 449
negatives rejected, zero accepted negatives and zero abnormal exits. The
newly exposed `run/fuel_loops.bend` debug stack overflow was reduced to a
Nat128 body and repaired with a bounded 16 MiB CLI worker stack. Nat128 now
returns the existing nesting error; nearby Nat120 passes. Proof limits did
not change. The complete audit was rerun after the repair.

Template/library audit: 349 positives checked, 504 positives rejected, all
449 negatives rejected, zero accepted negatives and zero abnormal exits across
all 1,302 fixtures. The five new positives are three nullary-ADT display
fixtures and `stuck/fold` / `stuck/map`. Upstream comptime positives still
require IO, GPU offload or unsafe execution; dedicated pure template tests
provide that supported feature's execution evidence. The audit's compiled
source fingerprints match the implementation being published.

Array audit: 360 positives checked, 493 positives rejected and all 449 negatives
rejected across all 1,302 fixtures. There are zero accepted negatives, crashes
or newly rejected positives. Eleven further positives cover array statements,
array literals, chained expressions and literal readback. The complete quality
gate passes 131 tests with two opt-in local profilers ignored. All 14 finite
Array behavior examples also match the upstream normalizer independently.

Work: use upstream `#|` fixture expectations in a local test harness. Do not run
upstream cluster scripts. There are 1,302 fixtures; enumerate and classify all
instead of claiming parity from a handful of examples.
Validation: run parse/check/proof/halt/eval categories; investigate divergence.
Completion: every advertised feature has fixture evidence and all remaining
differences are named, with work retained in this plan.

## 3. Compiler and runtime parity

### [~] 3.1 Port the compiler and execution support

Completion notes: native Rust JavaScript generator and `compile` CLI implemented.
Ten Node execution tests and source-name escaping test pass. Closures, patterns,
parallel let scope and recursive arithmetic run independently of TypeScript.
`compile examples/induction.bend` produces a program returning Nat 5. A baseline
portable C11 backend now passes 11 real MSVC compile/run tests against the
normalizer and JavaScript; `compile --target c` selects it. Effects, foreign
bodies, upstream optimization strategy and GPU runtime remain open.

The complete Poche `absorbed_trace` also compiles and runs through C with its
default limits, yielding exactly the JavaScript constructor result. Both the
full witness and focused closure/allocation-failure fixtures pass MSVC
AddressSanitizer checks. Generated C is a portable baseline, not upstream
optimization parity.

Work: map `comp.ts` IR, lowering, code generation and runtime functions to Rust;
cover upstream C and JS behavior before GPU runtime integration.
Validation: upstream compiler/runtime/effect fixtures against actual outputs.
Completion: the Rust tool can compile and run the advertised target set.

### [~] 3.2 Finish language/library and target coverage

Completion notes: 367 selected pure source declarations are bundled
with Apache attribution (322 definition forms including 18 templates, 25 laws,
20 datatypes). Template instances enter the checked book at their call sites;
source and checker-event counts differ. Six Base
regressions and four numeric regressions cover witnesses, Boolean proofs,
equality transport, lists, division, Word/U32 helpers and false claims. The
Word full-adder formula has an exhaustive checked Boolean law. A targeted
audit accepts 23 more of 133 previously rejected positive fixtures without
crashes. Full Base and target coverage remain required.

The next pure library slice is implemented: Patricia Map, Char/String operations,
and List templates including map, filter, folds and stable sorting. Ten new
collection tests cover boundaries, duplicate preservation, lookup/update/delete
lifecycles and affine values. All 20 Base/numeric/collection tests pass.
Upstream's seeded Base can skip ordinary source-order validation; this port
checks its bundled pure Base. Rewrite forward helper cycles into equivalent
structural definitions rather than allowing unchecked live forward laws.

The pure Array operations now check and execute: size/new/get/swap/set/clone,
to-list and map. Five focused tests cover 14 finite behavior cases and six
negative controls, including illegal affine duplication and false equality.
A wrapped-update/mapping fixture also agrees across the normalizer, native
runtime, JavaScript and C. Existing array-write parser coverage now uses the
real implementation instead of a temporary function stub.

Remaining library work includes window-backed App helpers, effects and foreign
implementation contracts. Representation or
syntax support alone does not establish an operation's implementation.

All 37 execution-only numeric contracts are now implemented in a separate
sealed registry, with bounded native IO evaluation and executable JavaScript.
Strict proof normalization keeps them opaque. Ten ordinary F32 helpers,
Nat/U32 decimal readers and checked addition commutativity complete the missing
ordinary numeric helper inventory identified in this slice. Native surface
printing, large native Nat representation and executable C remain unfinished;
see [numeric execution](numeric-execution.md).

Bool.show, Maybe.show, the nine Set definitions and sequential List.for_each
are implemented. Channels have a separate sealed executable-only opaque Chan
contract, four JavaScript foreign operations and four ordinary IO.fork/join
helpers. Image/Event and four ordinary Image helpers are implemented; executable
Base adds App and finite more/fold/play helpers. A declaration-name inventory
still finds six ordinary definitions, 24 foreign functions and five opaque laws
absent from the union of pure and executable Base after the playback slice.
Completing that inventory alone does not establish target, runtime or language
parity.

Work: base library, templates, effects, packaging and supported CPU/GPU targets;
record unavailable hardware and platform-specific validations accurately.
Validation: compatibility matrix with observed results, not assumed parity.
Completion: U6 has a documented complete scope and verified coverage; any
remaining unsupported areas keep the broad rewrite goal active.

### [~] 3.3 Implement effects and foreign execution contracts

Reference reconnaissance is complete in [the effects design](effects-design.md).
The native console implementation now has a separate executable-check result,
retained foreign source/origin metadata, exact IO continuations and requests,
ordinary IO helpers and three native console effects. `run` preserves raw
output and Halt exit codes. The full gate passes 168 tests including three
compile-fail API boundaries. Eight positive upstream fixtures match actual
upstream JavaScript stdout/stderr/status; two refusals match stdout/status and
absence of unintended effects, with different diagnostic wording.
Typed executable lowering now retains code-generation types, quantities,
constructor owners/fields, local tags, matches, lets and erasure. Its first
consumer is a distinct executable JavaScript emitter; the strict pure compiler
and proof-check result remain separate. `compile --executable` supports native
JavaScript representations, synchronous foreign imports, curried callbacks and
the console driver. Foreign source is embedded without running it at compile
time. Undefined results now suspend in the cooperative JavaScript scheduler;
saved continuations, spawned tasks and time hooks are supported. Promises and
descriptor readiness remain explicit failures. The C interface and effect driver,
native scheduling, channels, handles and the other 28 Base
foreign effects remain required.
Upstream foreign return contracts can contain false equality payloads; they
are runtime assumptions and must never mint strict proof evidence. This is
required remaining rewrite work, not optional replacement scope.

The direct ordinary U32.show port exhausted the native arena even for 42.
It now uses checked decimal doubling over bits, preserving upstream show helper
truncation/accumulator behavior. Boundary/sample tests cover u32::MAX, every
power-of-two boundary, decimal carries and 48 deterministic sample words.
No host intrinsic or proof-checking exception implements that conversion.

Validation: actual stdout/stderr/status and effect ordering against upstream
fixtures, request-versus-constructor rejection, forged-origin tests, marshalling
and callback behavior, plus unchanged strict proof and Poche protocol tests.
Completion: the full documented effect/interface inventory executes through
Rust implementations and Rust-generated backends with observed compatibility.

## 4. Poche formalization using this rewrite

### [x] 4.1 Add executable independent Bend rule kernels

Completion notes: independent Poche model has seven scalar entry points for six
rule groups, two universal definitional laws and five closed equality examples.
Both upstream and our native checker accept it. Model Blake3:
`cf1791c29ac0ac56e48eb5044b9f4a05ec19ec3d0bddf1a6b4dad51c389b1034`.

Work: bid bounds, follow suit, two-card winner, score/payment, final-round
decision and winner mask in `<poche-root>/models/bend/`; add checked laws.
Validation: our Rust CLI checks and evaluates the actual Bend files, no host
callback implements the rules. False-law fixtures must fail.
Completion: documented rule IDs, source provenance and checked laws.

### [x] 4.2 Compare rules with the existing independent implementation

Completion notes: Poche `compare rust bend --scope kernels-v1` passes 15,503
finite rows, seven checked equality terms and two negative controls (false
equality plus a well-typed incorrect scoring implementation). Repeated with
the repaired checker. Four harness tests and xtask checks pass. Receipt contains
model/rulebook/oracle hashes and portable backend identity. Poche edits remain
local; concurrent unrelated UI work was left untouched.

Work: `<poche-root>/crates/poche-conformance/src/bend.rs`, command wiring and
scope documentation. Executable path is configurable, never machine-specific.
Validation: exhaustive named finite domains, checked-case counts and
discriminating negative controls for each kernel.
Completion: actual production/oracle results agree with the independent Bend
program over stated domains; mismatches show inputs and outputs.

### [x] 4.3 Extend the independent model to bounded state transitions

Design handoff: `docs/poche-state-model-design.md` specifies exact public-getter
adapters, typed constructor protocol, state/action schema, deterministic replay,
431,800-state/549,896-edge reference and negative controls. The independent
micro model now checks, and its first-round trace produces the expected
scores. The persistent typed constructor protocol and Rust micro adapter pass
22 state inspections, 21 transitions, all 300 chance partitions and eight
invalid-state/action controls. A release sample also passed 1,000 distributed
states and 1,000 edges, including both observation projections. This is not
exhaustive evidence. The measured 38.487-second sample projects roughly five
hours for the full graph with the substitution-based evaluator.

A separate native call-by-need runtime now passes differential and independent
adversarial tests for checked data calls. Complete program checking and per-call
dependent argument type checks remain mandatory; the proof normalizer stays
independent. The same sample passes in 10.520 seconds, with backend evaluation
about 6.3 times faster (roughly 44 minutes projected for the complete graph).
Profiling found runtime computation, rather than JSON or argument validation,
dominates the remaining work, so the existing codec is retained. Run the
complete graph using the published `6802c1e` clean release executable.
Poche's compiled 21-transition witness also succeeds, and well-typed wrong-leader/wrong-scoring
source mutants produce the precise expected mismatches. Scalar or sample
success does not mark this task complete.

Completion: the complete run finished successfully using the clean `6802c1e`
release after fresh scalar and trajectory preflights. Every one of the
431,800 reachable states, 549,896 labeled transitions and 863,600 observations
matched; all 300 chance partitions and eight rejection controls passed.
The 981,720 requests took 1,658.848 seconds. The receipt records model BLAKE3
`3798ecaad4f3bff64d9648452c0f7f2090bde1294a5beb8c505d8bce4fedf226`
and transcript BLAKE3
`a273a13c52d7e36f956c30b0675cfefeb60ace620dece023b0beb361b0bbed39`.
Its source snapshot includes only rule-ID comments added after the earlier
samples; normalized semantic source is unchanged. A dedicated Poche coverage
document maps all 61 normative rules with explicit bounds and unmodeled cases.
This completes the bounded transition comparison specified here; full-deck,
network, UI and money-transfer correctness and an inductive invariant proof
remain separate work and are not established by enumeration.

Work: phases, exact card partition, immutable bids, trick progression, winner
leads, score/pot separation, absorbing finish and decreasing progress. Compare
the existing shared 1/2 prefix and then the full micro schedule where supported.
Validation: finite state/transition comparisons with exact model bounds and
counterexamples; preserve normative rules and conventional authority.
Completion: reviewed scope/evidence accurately separates kernel laws, bounded
state checking and unproved full-deck/network/UI behavior.

### [x] 4.4 Prove symbolic observation privacy

Seven separate laws in Poche's `models/bend/privacy.bend` quantify over arbitrary
well-typed observations and states. For either viewer, equal-size changes to
the opponent's hidden hand cannot affect the observation when own/public fields
are fixed. Bidding/Playing corollaries also allow arbitrary hidden stock and
captured identities; Scoring has no hand-count premise. A well-typed wrong-viewer
mutation fails the unchanged privacy proof. This is symbolic equality checking,
not enumeration or a claim about protocol/trace/full-deck confidentiality.

The proof imports the existing micro model without changing its bytes. It
requires the new module-scoped Nat sugar resolution; the earlier `f12096c`
release cannot check the separate imported proof. The original exhaustive
model receipt remains intact and attributable to its retained executable.

## 5. Validation and publication

### [x] 5.1 Publish the first verified implementation and evidence

Completion notes: public remote exists; first milestone review excludes local
paths, raw diagnostics, build outputs and reference downloads. Full quality gate
and final upstream audit passed. First implementation commit `2e540f9` is on
public `main` at https://github.com/TeamDman/teamy-bend. Both source and Poche
remotes were verified public; Poche changes remain locally reviewable. This
completes initial publication, not the broad rewrite/formalization goal.

Work: README, compatibility status, docs and licence headers; inspect all staged
content and commit identity; commit and push the requested public repository.
Do not publish local discovery output, captures, private paths or Poche changes
incidentally. Poche changes can remain locally reviewable unless its workflow
or the user authorizes their publication.
Validation: quality gate, help/version, real Poche conformance command,
publication scan, remote visibility and clean commit verification.
Completion: public repository contains the verified implementation and honest
limitations; Poche integration is runnable and evidence is reproducible.

### [x] 5.2 Publish typed execution and portable C generation

Completion: commit `6802c1ec78686c8804d873394aad580bcb71854d` is verified on
public `main`. The full quality gate passes 100 tests with two opt-in local
profilers ignored. The full 1,302-fixture audit has zero crashes or accepted
negative fixtures. A clean release binary was copied before further source
edits and reports revision `6802c1e` with a clean worktree. Poche receipts
fingerprint the actual executable, the served model snapshot, and the compiled
Rust rules/explorer/adapter sources. The complete micro comparison will use
that fixed release while broader library work continues independently.

### [x] 5.3 Publish templates, checked collections and Poche evidence

Completion: `92d20eebcac1df2e98c7af992cda1c76a0ad1fe1` is verified on
the public `main` branch. A clean release snapshot identifies revision
`92d20ee`. Its Poche scalar gate passed all 15,503 comparisons, seven checked
equalities and two negative controls; its trajectory gate passed 22 states,
21 transitions, all 300 chance partitions and eight rejection controls.
The exhaustive graph receipt remains tied to the separately retained
`6802c1e` binary, without attributing that run to the newer release.

Subsequent progress: pure Array operations were published in `abf373e` with
131 passing tests, 14 upstream value comparisons and unchanged negative-proof
acceptance. Native console execution now uses separate loader/checker products
for external contracts, preserving the strict proof path. Generated effect
drivers and the remaining foreign inventory are the next implementation work.

The template/library publication slice passes the complete quality gate
(124 tests and two ignored local profilers), plus the subsequently added
deferred-template regression (13 template tests pass). Independent review
found no new defect in specialization keys, closure checks, declaration order,
budgets or affine collection behavior. `base` source/name/type output and
invalid selectors passed native CLI smoke checks. All 1,302 upstream fixtures
were audited against a fixed executable while development continued separately.

### [x] 5.4 Publish native console execution

Completion: `f12096c709bd225aabaadbf07b09ae46c184c16f` is verified on
public `main`. A retained release executable reports revision `f12096c` and a
clean worktree. Poche regressions against the frozen audited build passed all
15,503 scalar comparisons, seven equalities, two scalar negative controls,
22 trajectory states, 21 transitions, all 300 chance partitions and eight
invalid-state/action controls. The older exhaustive receipt remains unchanged.
Both gates also pass against the clean `f12096c` release; its 67-request
trajectory run completed in 333 ms. The release executable BLAKE3 is
`ca0811356fdb54db30154766c067d107c16071882a35121fb40bfbc96bfdb074`.

The console slice adds `run`, distinct executable contracts, private IO requests,
three sealed console handlers and checked decimal U32 text. Its standard gate
passes 168 tests with two optional local profilers ignored. Ten real upstream
IO comparisons agree on output/status within their stated diagnostic scope;
17 independent provenance, source-order and Unicode-boundary probes pass.
The full 1,302-fixture strict audit retains 360 accepted positives, 493 rejected
positives and 449 rejected negatives, with no abnormal exits. The source change
after the audit was removal of an extra trailing blank line from pure Base.

Native evaluation remains lazy, so a discarded invalid Char differs from the
upstream JavaScript runtime's eager scalar validation. Arbitrary foreign code,
generated JavaScript/C effect drivers, remaining Base effects and full numeric,
GPU, hub and CLI behavior remain open. This is a publication milestone, not
completion of U6 or the broad goal.

### [x] 5.5 Publish executable numerics, import scope and typed lowering

Completion: `9da68e5af881d9f473e77353d7811eedcb013412` is verified on
the public `main` branch. The retained executable reports that revision and a
clean worktree. Poche proof and adapter changes remain locally reviewable.

The complete quality gate passes 190 tests, including four compile-fail API
examples, with two optional profilers ignored. Ten focused typed-lowering tests
and an independent erasure/dependency/identity review pass. The imported Nat
literal defect was reproduced from the unchanged Poche model and reduced to
literal-versus-explicit-constructor module fixtures; three regressions cover
scope, operators, Base identity and distinct datatype rejection.

An intermediate frozen-build audit covers all 1,302 reference fixtures with
360 accepted positives, 493 rejected positives, 449 rejected negatives and zero
abnormal exits. Native numeric comparisons include 30 exact generated boundary
cases, two unchanged upstream programs and one recorded signaling-NaN target
difference. The longer upstream float comparison program still exhausts the
native arena; the existing limits remain unchanged.

The clean `9da68e5af881d9f473e77353d7811eedcb013412` release passes
the final full strict audit with the same 360/493/449 acceptance counts and
zero crashes or accepted negatives. It also passes all seven symbolic Poche
privacy laws and the wrong-viewer control, all 15,503 scalar comparisons and
the micro trajectory (22 states, 21 transitions, 300 partitions and eight
controls; 67 requests in 339 ms). Its executable BLAKE3 is
`2dc33071205ebcde7d1165bd1e8286350be8656140501dc3680441895e23bf20`.
Original Poche model bytes and all 13 compiled conformance fingerprints remain
unchanged. The earlier exhaustive receipt retains its original attribution.

### [x] 5.6 Publish synchronous executable JavaScript

Completion: `06f08086c61bcc53a479d9b08b10ece17db113a9` is verified on
public `main`. The retained executable reports that revision and a clean
worktree; all 71 compiled source fingerprints match the audited candidate.
The clean release also passes the 34-program upstream JavaScript comparison.

Implementation: typed emission and the generated IO driver are connected to
`compile --executable`. Separate checked products preserve the strict proof
boundary. Ordinary Nat decimal text supplies the upstream native-value fixture;
a sealed native U32.add optimization removes the observed float-example arena
failure without changing proof reduction or budgets.

The complete quality gate passes 236 tests, including four compile-fail API
examples, with two optional profilers ignored. Strict library/test Clippy and
independent reviews of code generation, FFI isolation, erasure, native
optimization and publication provenance pass. The relative-import defect was
reduced to operator qualification of parent/dot-directory names and has four
parser regressions. NaN payload loss was reduced to V8 array-spread behavior
in argument accumulation; both signs and quiet/signaling inputs have regressions.

Fixed-release-candidate comparisons pass 34 upstream programs (31 exact
stdout/stderr/status matches and three expected failures with distinct wording)
and a separate 34-case numeric matrix, whose three upstream programs overlap
the first set. Native execution passes 33 numeric cases exactly and preserves
the documented signaling-NaN target difference. The original longer float
program now prints 1999985 under unchanged native limits.

The fixed candidate also passes the complete 1,302-fixture strict audit:
360 accepted positives, 493 rejected positives and 449 rejected negatives,
with zero abnormal exits or accepted negatives. Acceptance is unchanged from
`9da68e5`; all compiled source fingerprints still match the audited snapshot.
The retained clean release passes all seven Poche symbolic privacy laws and
the well-typed wrong-viewer control, all 15,503 scalar comparisons with seven
equalities and two controls, and the micro trajectory: 22 states, 21 transitions,
44 observations, all 300 chance partitions and eight controls. Its 67 requests
complete in 312 ms. The executable BLAKE3 is
`2cc22a65de57f922f112de728bbf3dd15a7ca0c6d1f54c38103c1da7292de30a`.
The original models, privacy proof/runner and all 13 compiled conformance-source
fingerprints remain unchanged. Poche changes remain local; the exhaustive graph
receipt retains its original `6802c1e` attribution.

Full scheduling, executable C, remaining library/CLI and GPU behavior still
keep the goal active. The next implementation work remains in 3.2 and 3.3;
completion of this publication milestone does not complete U6.

### [x] 5.7 Complete numeric execution and ordinary readers

Implementation is complete: all 37 upstream numeric primitive contracts preserve
exact quantities and the `Maybe<&2, F32>` read result. The 19 remaining scalar
math operations and F32 show/read execute in native Rust and JavaScript. Ten
ordinary F32 helpers, pure Nat/U32 readers and checked addition commutativity
complete this numeric helper slice. The Nat reader retains its full 48-bit
contract without first expanding its enormous unary bound for small inputs;
materialized large native values still return explicit resource errors.

The complete quality gate passes 254 tests, including four compile-fail API
examples, with two optional profilers ignored. Strict library/test Clippy and
independent implementation, proof-boundary and publication reviews pass.
The frozen candidate passes 186 math cases on each target against its own
oracle, with exact non-NaN bits and NaN classification. Fourteen unchanged
upstream numeric IO programs match stdout/stderr/status on both targets.
Native text comparisons cover 20,012 formatting and 14,862 parsing inputs;
JavaScript covers 20,000 formatting and 177,624 grammar/whitespace inputs.
Eighteen ordinary reader cases match upstream, including full 48-bit Nat bounds.
Target-specific parsing and NaN differences remain documented.

The frozen candidate's complete strict audit covers 1,302 fixtures: 361 accepted
positives, 492 rejected positives and 449 rejected negatives, with no accepted
negatives, crashes or newly rejected positives. The additional accepted fixture
is `proof/word_add_comm.bend`. All compiled source fingerprints remain unchanged
after the audit.

Completion: `02aab0fae923828328d7b873a959ad84c3de3910` is verified on
public `main`. The retained release reports that revision and a clean worktree;
all 72 compiled source fingerprints match the audited candidate. Its executable
BLAKE3 is `f539c5602723ede9e363285181da09548e9d952de8041f6d1817bd5c410bdff0`.
It passes all seven Poche privacy laws and the wrong-viewer control, all 15,503
scalar comparisons with seven equalities and two controls, and the micro
trajectory: 22 states, 21 transitions, 44 observations, 300 chance partitions
and eight controls. Its 67 trajectory requests complete in 364 ms. All 13
compiled conformance fingerprints and original model/privacy source bytes are
unchanged; Poche edits remain local. The exhaustive graph receipt retains its
original `6802c1e` attribution.
Native surface printing, scheduler, executable C and remaining U6 scope stay open.

### [x] 5.8 Add cooperative JavaScript tasks and timers

Implementation is complete: the reference FIFO continuation scheduler supports
undefined suspension, saved continuation resumption, tasks that outlive main,
terminal Halt and deadlock reporting. Sealed IO.spawn, IO.sleep and IO.now retain
their exact source signatures. The portable Node timer wait preserves the
synchronous generated-program interface; promises, descriptor readiness and
native Rust scheduling remain unsupported. Queue, live-task and transition
budgets fail explicitly. Strict proof results and private request identity
remain separate.

Validation: 209 deterministic traces match the actual upstream scheduler with
an explicit timer-only host adapter. This comparison exposed a missing host
wait for zero/overdue deadlines; the corrected adapter preserves that scheduling
point. The five upstream spawn/sleep/clock fixtures match stdout/stderr/status.
All 34 prior executable fixtures still pass: 31 exact matches and three expected
failures with distinct diagnostics. Nine new scheduler tests and independent
origin, erasure, request, lifetime and resource-limit reviews pass. The complete
quality gate passes 266 tests, including four compile-fail API examples, with
two optional profilers ignored. Strict library/test Clippy also passes.

The frozen candidate passes the complete 1,302-fixture strict audit with unchanged
361 accepted positives, 492 rejected positives and 449 rejected negatives;
there are no accepted negatives, crashes or changed compiled source fingerprints.
The task example compiles and prints main completion before its delayed child.

Completion: the published implementation is
`23f42e38b17e03ed7972cad4eb9a8359a84a66b7`. Its retained release reports
that revision and a clean worktree; all 72 compiled fingerprints match the
audited candidate. Executable BLAKE3 is
`6c4a4e5c06415eaf6de8d2fb31cb6b2c55ffca3e3c982cdac8c3d555758a0051`.
It passes all seven Poche privacy laws and the wrong-viewer control, 15,503
scalar comparisons with seven equalities and two controls, and the micro
trajectory: 22 states, 21 transitions, 44 observations, 300 chance partitions
and eight controls. The 67 trajectory requests complete in 304 ms. All 13
compiled conformance fingerprints and original model/privacy bytes are unchanged.
Poche changes remain local; exhaustive evidence remains attributed to `6802c1e`.
Channels and ordinary fork/join helpers remain the following scheduler work.

### [x] 5.9 Add checked Set helpers and executable channels

Implementation is complete: eleven ordinary Bool/Maybe formatting and Set
definitions, sequential List.for_each, an exact loader-sealed executable-only
Chan contract, four JavaScript channel operations and four ordinary fork/join
helpers. All 21 added Base declarations match the fixed reference source.
Channels preserve FIFO buffers, zero-capacity rendezvous, affine payloads,
close wakeups and raw foreign rows. Private cleanup retains host-visible rows.
The reference's null sender/receiver-marker collision is preserved and tested.
Chan never becomes a strict proof certificate; native channels remain unfinished.

The real chan_pipe fixture exposed rejection of structurally smaller let aliases.
Descent now follows original let RHS aliases and annotations, with existing
step/nesting bounds and no general function reduction. Ten reduced cases and
seven independent edge cases agree with upstream, including rejection controls.
The complete 1,302-fixture strict audit accepts 362 positives and rejects 491
positives and all 449 negatives, with no crashes, newly rejected positives or
changed compiled source fingerprints. The newly accepted positive is
proof/rewrite_type_family.bend.

The complete quality gate passes 299 tests, including five compile-fail API
examples, with two optional profilers ignored. Strict library/test Clippy and
independent boundary, driver, descent and publication reviews pass. All 509
deterministic channel traces and 209 scheduler traces match upstream, including
raw row state. The 34 previous executable comparisons still pass (31 exact and
three expected diagnostic differences), as do five timer fixtures.
Nine unchanged channel/fork/join fixtures agree with upstream under both exact
virtual waits and injected oversleep: 18 comparisons, with only expected deadlock
diagnostic wording different. An original real-clock mismatch was retained and
reproduced in upstream: multiple overdue timers wake in registration order.
Both runtimes produce either observed order under the same shared clock adapter.

Pure-helper evidence includes 28 runtime values, 25 checked equalities/projections
and seven negative controls. Nineteen complete comparison programs agree with
upstream. Two unchanged long IO programs still reach the existing nesting limit;
all 67 print expressions match in nine shorter chunks. This does not count as
whole-program equivalence. No resource budget was raised.

Completion: `f27f7ba6c4531d12c4bb33fcf2414238903593b3` is verified on
public main. Its retained release reports that revision and a clean worktree;
all 72 compiled source fingerprints match the audited candidate. Executable
BLAKE3 is `0e535bb8516581f31fd16788f52879abe7056992af9c1ad29d83b5ef03413b3a`.
It passes all seven Poche privacy laws and the wrong-viewer control, 15,503 scalar
comparisons with seven equalities and two controls, and the micro trajectory:
22 states, 21 transitions, 44 observations, 300 chance partitions and eight
controls. Its 67 requests complete in 301 ms. All 13 compiled conformance
fingerprints and 17 source fingerprints are unchanged from the timer release.
Poche source changes remain local; the exhaustive graph is still attributed
solely to `6802c1e`. Remaining U6 target/runtime/CLI work keeps the goal active.

### [x] 5.10 Add Image/Event values and finite App playback

Implementation is complete: exact Image/Event datatypes and Image.sink,
drop.join/free/drop in strict Base; exact App and more/fold/play in executable
Base. These ten declarations introduce no kernel, compiler or runtime rule.
Finite playback preserves affine state and closed template specialization,
processes frames sequentially and stops at None/Halt without invoking view.

Five pure tests cover 20 checked equalities, 22 runtime values and seven rejection
controls, including constructor order, reusable kinds, erased/affine ownership
and descent. Eight native/JavaScript App tests cover all Event fields, callbacks,
empty input, early stopping, unused view and captured/duplicated affine values.
An explicitly invoked expensive view still reaches unchanged resource limits.
The asymmetric Image/Event result matches its literal expected tree in native
evaluation and agrees across normalization, generated JavaScript and compiled C.
The public playback example prints Processed events: 3 on both executable targets.
The full quality gate passes 313 tests with two optional profilers ignored;
strict library/test Clippy and independent source/boundary reviews pass.
Six JavaScript comparisons against independently generated upstream programs
match stdout/stderr/status. Native execution matches four of five applicable
programs; the large Image observation fails as detailed below. The arbitrary
foreign callback observer is excluded only from native execution.

An exact Image build/fold extraction from the upstream window fixture exposes
a native compatibility gap: depth zero through two passes, but depth three
(64 leaves) onward exhausts the thunk arena. Shape-only counting passes depth
four but fails at five; following one path and forcing Image.free pass through
depth six. All 21 reduction probes have upstream output oracles. Cumulative
intermediate allocation is the likely cause; this is an inference awaiting
runtime instrumentation. The depth-six JavaScript workload passes, while the
native failure remains recorded without smaller-workload substitution or raised
budgets. Completing the library names does not resolve this runtime work.

The frozen candidate's complete 1,302-fixture strict audit has unchanged
362 accepted positives, 491 rejected positives and 449 rejected negatives,
with no accepted negatives, abnormal exits or changed source fingerprints.
Completion: `39f3dc5731ffa86c7017f179d11511fdf5fca964` is verified on
public main. Its retained release reports the clean revision and all 72 compiled
source fingerprints match the audited candidate. Executable BLAKE3 is
`1a413d37286879e272957003f4c004c3f5e107460a9d7baf764bda92e6786507`.
It passes all seven Poche privacy laws and the typed wrong-viewer control, 15,503
scalar comparisons with seven equalities and two controls, and the micro
trajectory: 22 states, 21 transitions, 44 observations, 300 chance partitions
and eight controls. The 67 requests complete in 300 ms. All 13 compiled
conformance fingerprints and 17 source fingerprints remain unchanged from the
channel release. Poche edits remain local; the exhaustive graph retains its
original `6802c1e` attribution. Native Image limits, window-dependent App helpers,
opaque handles, remaining platform effects and all other unfinished U6 work
remain required.

### [~] 5.11 Reduce native word allocation and complete the Image workload

The previous native numeric result allocated 66 thunks. A depth-six shape count
uses 4,095 additions, requiring 270,270 result thunks alone in an append-only
arena limited to 131,072. Even interning Boolean bits and WNil would leave
135,135 result thunks. Ordinary arithmetic acceleration alone could not remove
that lower bound.

Compact words are implemented: execution-only U32/F32 literals and numeric
results retain raw bits, decode directly for numeric/console operations, and
expose bounded lazy Word views for pattern matching. Exact float payloads,
sealed Base provenance, user lookalikes and strict proof behavior are preserved.
Only complete closed literal trees pack automatically; arbitrary constructor
fields stay lazy. Checked ordinary multiplication and one-bit shift join the
addition optimization; recursive shln remains ordinary. Optimizations demand
outer wrappers in source order and fall back to checked bodies for ordinary
fields. Regression tests also fixed an older eager-addition bug.

The complete quality gate passes 332 tests, including five compile-fail API
examples, with two optional profilers ignored. Strict library/test Clippy and
independent representation/boundary reviews pass. Six private representation
tests and eight integration tests cover conversion, wrapping, float payloads,
partial application, origins, unused fields, request rejection and unchanged
cancellation/limits. Actual upstream normalization confirms the lazy arithmetic
controls. All 186 float-oracle cases and 14 unchanged upstream numeric programs
still match their respective native and JavaScript expectations.

The frozen 256-case integer comparison matches 253 native outputs against
actual upstream JavaScript and independent modulo arithmetic. Three reconstructed
multiplications exhaust the arena in both this candidate and the previous
retained release. These are compatibility failures, not matching successes.
Image probes improve from 15/21 to 19/21 with no regressions: exact sums pass
through depth five, including 14,560 at depth three and 1,031,680 at five;
shape-only counting passes through five. Exact fold and shape count at six
still fail, with required outputs 8,386,560 and 4,096. All six full JavaScript
programs match upstream; native still passes four of five applicable programs,
with the original full Image failure retained. No limits or workloads changed.
The complete 1,302-fixture strict audit remains 362 accepted positives, 491
rejected positives and 449 rejected negatives, with no crashes or changed
compiled source fingerprints.

Compact-word implementation: `a7ddfcc79c509890b119b76daf41f54b9147895a`.
Its retained release reports that revision with a clean worktree; all 73 compiled
source fingerprints match the audited candidate. Executable BLAKE3 is
`5e2acc14b82e2335fc07fcb180dc9452d2dd373d7c8b7b4f215311eabf057f90`.
It passes all seven Poche privacy laws and the typed wrong-viewer control, 15,503
scalar comparisons with seven equalities and two controls, and the micro
trajectory: 22 states, 21 transitions, 44 observations, 300 chance partitions
and eight controls. The 67 requests complete in 482 ms. All 13 compiled
conformance fingerprints and 17 source fingerprints are unchanged from the
Image/App release. Poche changes remain local, and exhaustive graph evidence
remains attributed solely to `6802c1e`.

Next: establish explicit roots before adding nonmoving reclamation of both
arenas. Collect only at committed force-loop safe points, never inside allocate.
Roots must cover current/continuation IDs, Update blackholes, all numeric argument
vectors and saved tails, globals, closures/environments, pending IO requests and
Halt fields. Scoped caller roots must retain siblings and temporary packed
constructor views across nested decoding/materialization calls; those view
fields are not reachable from the original packed value. Trace metadata only,
without forcing terms or running effects. Use stable IDs, vacant slots and free
lists, bound marker storage and charge collection work to the existing budget.
Keep each arena capped at 131,072 slots and reject a genuinely full live graph.

Reclamation changes cumulative allocation limits into live-value limits and
needs explicit documentation. Test collection at every safe point against
collection-disabled evaluation, shared closures/lets, numeric continuations,
packed views, IO order, cancellation, request rejection and live-state exhaustion.
Measure retained globals/environments before promising depth-six success.
Rerun the unchanged Image workload and the three integer refusals, then the
full gate, strict audit and retained-release Poche regressions. This plan item
and the broad goal remain open until the full workload succeeds.

## Completion and risks

The goal is complete only when U1–U9 are delivered and no required rewrite or
formalization work remains. A scaffold or supported language subset is progress.
Proof soundness risk is controlled by preserving erasure/resource/descent rules,
rejecting unsupported constructs and testing false proofs. Model correspondence
risk is controlled by independent conformance and negative controls. Reference
contamination is controlled by read-only checkouts and tracked template selection.
Hardware-dependent coverage must be marked unverified when unavailable.
