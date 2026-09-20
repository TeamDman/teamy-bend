# Bend Rust rewrite and Poche formalization

**Plan status:** Active
**Primary implementation root:** `teamy-bend` repository
**Last updated:** 2026-09-20
**Intent audit:** Updated 2026-09-20 against the original request and available scope/GPU follow-ups

## How to update this plan

- `[ ]` Not started; `[~]` In progress; `[x]` Complete; `[!]` Blocked.
- Update each task heading and its completion evidence together. A phase is
  complete only when all its work is complete.
- Preserve the requirements ledger. Do not convert a first supported subset
  into a claim that the entire requested rewrite is complete.
- Historical parallel ownership is not a list of currently running agents.
  Native tasks, timers and channels are complete for this bounded native slice
  (5.12), as are environment and file effects (5.13). Native descriptor readiness
  and TCP/UDP now pass native and generated JavaScript validation on Windows
  (5.14); executable C is the next engine implementation (5.15). Keep existing Poche
  checks as regressions and defer model expansion.

## Goal wording and scope

The user resumed the goal with the following wording on 2026-09-20. It preserves
the original intent and makes engine priority and later GPU references explicit:

> Complete teamy-bend, the public Rust port of Bend2 created with gh from
> teamy-rust-cli, using MPL-2.0 where compatible with upstream licensing.
> Prioritize the checker, parser, compiler and CPU/GPU runtimes, validated against
> upstream behavior. Use the port to formalize Poche4's rules and state model with
> explicit proof boundaries and validation, preserving its existing application
> architecture. During the GPU phase, evaluate the Makepad and teamy-tts
> strategies recorded in this plan.

The user's concern is scope drift into rebuilding Poche or Bevy. Engine parity
remains the main unfinished work. Existing bounded Poche evidence is useful
validation, not a claim of whole-game/application correctness and not a silent
redefinition of the final requested formalization. Resolve any additional
formalization acceptance scope before declaring the overall goal complete.

The goal tool now reports active. When GPU work becomes the active phase, read
[GPU port references](gpu-port-references.md); exact local checkout locations are
in the ignored `.local/gpu-port-references.md` file at the repository root.

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
| U10 | Keep progress focused on the Bend2 engine; address concern about rebuilding Poche or Bevy. | Goal wording and scope; 5.12–5.15; existing Poche regression coverage |
| U11 | Makepad strategies identified by the user as Rik Arends's work may help the later Bend GPU port. | GPU port references; 3.4, evaluation deferred to GPU phase |
| U12 | The user reports those strategies improved teamy-tts; retain it as a second implementation reference. | GPU port references; user-reported provenance distinguished from inspected source |
| U13 | Persist both local reference paths and their purpose across compaction; assess current goal wording. | Ignored local reference note; portable GPU reference document; accepted wording above |

## Intent audit evidence

- Extraction: reread the original compound request; recorded naming, gh,
  visibility, conditional licence, template, rewrite, order, approximate paths
  and goal separately in U1–U9.
- Traceability: each requirement maps to a task and observable acceptance.
  U6 includes runtime/compiler work beyond the first proof-checking slice.
- Adversarial omission: retained the licence condition, uncertainty in repo
  names, and the requirement to use our implementation in Poche. Existing
  Poche formalization is foundation, not grounds to omit the integration.
- Follow-up extraction pass (2026-09-20): reread the available original request,
  scope/progress questions and GPU-reference message. Added U10-U13, preserving
  both named checkouts, the reported Makepad-to-teamy-tts relationship, the
  qualifiers "may be useful" and "when it's time", and the request for durable
  notes and a wording assessment.
- Follow-up traceability pass: U10 maps to the current engine milestone and
  Poche regression boundary; U11-U12 map to deferred GPU evaluation; U13 maps to
  the portable reference document, exact ignored local paths and proposed goal
  wording. U1-U9 remain unchanged.
- Follow-up adversarial omission pass: no immediate GPU implementation, mandated
  Makepad dependency, imported UI framework, guaranteed speedup or reduced
  formalization scope was inferred. The recommendation does not change the
  recorded goal. Local paths remain untracked.
- The subsequent goal continuation adopts the recommended wording and resumes
  implementation; U1-U13 and their existing acceptance boundaries remain intact.
- Known source limitation: intervening implementation conversations have been
  compacted; their existing ledger and completion evidence are retained. The
  original request and the current scope/GPU follow-ups are available directly.

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
now finds six ordinary definitions, seven foreign functions and two opaque laws
absent from the union of pure and executable Base after native network support.
The six file/environment contracts and sealed affine File type execute natively
and in generated JavaScript; see 5.13 and [native files](native-files.md).
The eleven network contracts and affine Socket/Listener types execute natively
and in generated JavaScript through its synchronous provider (5.14). Name coverage is not backend
parity.
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
the remaining opaque handles and Base effects remain required. Native tasks,
timers and channels now have their own implementation and evidence in 5.12;
environment and file effects follow in 5.13.
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

### [ ] 3.4 Evaluate Makepad and teamy-tts strategies during the GPU port

This is a deferred reference/decision task within the existing GPU scope, not
the next implementation milestone. Start with
[GPU port references](gpu-port-references.md) and its local checkout map.

Work: compare the original Bend2 offload/compiler/runtime requirements with the
reference approaches to device residency, transfer/synchronization boundaries,
buffer reuse, batching and workload-specific kernels. Decide which mechanisms
fit Bend's semantics and target hardware before introducing an implementation
dependency. Record relevant source revisions and provenance for any reuse.

Validation: preserve upstream outputs and ownership/effect behavior; measure
representative Bend workloads in release builds with hardware, cold/warm timing,
transfer and synchronization costs recorded. Any CPU/GPU numerical difference
must have a documented semantic justification. A reported TTS improvement is
not evidence of a Bend improvement; unavailable targets stay explicitly
unverified.

Completion: the GPU design records evaluated references, adopted or rejected
mechanisms with reasons, and actual compatibility/performance evidence. Existing
CPU/runtime work and Poche application architecture are not expanded by this
reference task.

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

### [x] 5.11 Reduce native word allocation and complete the Image workload

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

The compact-word quality gate passed 332 tests, including five compile-fail API
examples, with two optional profilers ignored. Strict library/test Clippy and
independent representation/boundary reviews pass. Six private representation
tests and eight integration tests cover conversion, wrapping, float payloads,
partial application, origins, unused fields, request rejection and unchanged
cancellation/limits. Actual upstream normalization confirms the lazy arithmetic
controls. All 186 float-oracle cases and 14 unchanged upstream numeric programs
still match their respective native and JavaScript expectations.

At that stage, the frozen integer comparison matched 253 of 256 native outputs
against actual upstream JavaScript and independent modulo arithmetic. Three
reconstructed multiplications exhausted the arena in both that candidate and
the previous retained release. Those were compatibility failures.
Image probes improved from 15/21 to 19/21 with no regressions: exact sums passed
through depth five, including 14,560 at depth three and 1,031,680 at five;
shape-only counting passed through five. Exact fold and shape count at six
still failed, with required outputs 8,386,560 and 4,096. All six full JavaScript
programs matched upstream; native passed four of five applicable programs,
with the original full Image failure retained. No limits or workloads changed.
The complete 1,302-fixture strict audit remained 362 accepted positives, 491
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

Nonmoving reclamation of both arenas is now implemented. Collection occurs only
at committed force-loop safe points, never inside allocate. Roots cover current
and continuation IDs, Update blackholes, numeric argument vectors and saved
tails, globals, closures/environments, pending IO requests and Halt fields.
Scoped caller roots retain siblings and temporary packed constructor views
across nested decoding/materialization calls. Marking traces metadata without
forcing terms or running effects. Stable IDs, vacant slots and free lists permit
reuse. Marker storage is bounded; traversal and sweeping consume the existing
step budget and check cancellation. An 8,192-slot reserve and allocation-debt
threshold avoid repeated futile collection.

Each arena remains capped at 131,072 retained slots; caller roots have a
separate 131,072-entry cap. This changes the arena constraint from cumulative
allocation to live retention. A full live graph still fails, and an individual
transition may run out of room before the next safe point. These semantics are
documented in the data protocol. No evaluation, continuation or output limit
was raised.

Seven private collector tests cover every graph edge, forced collection versus
disabled collection, slot reuse, full live arenas, scoped roots and 80 interruption
scenarios spanning marking and both sweeps. Six numeric/IO tests force collection
at every safe point, including partial arguments, decoder tails, ordinary
fallback, output order, Halt, cancellation and request rejection. Nine public
executable tests and three strict-data tests cover large discarded work, captured
closures, simultaneous lets, packed tails, effect boundaries and failed-call
isolation. A retained wide graph stops at the step budget; that result is kept
distinct from the private tests that fill both live arenas exactly. Two older
resource assertions now recognize the step/continuation limits reached after
their former allocation failure is removed.

The frozen collector candidate passes the full gate: 357 tests, including five
compile-fail examples, with two optional profilers ignored. Strict library/test
Clippy and independent root/sweep/boundary reviews pass. The complete strict
audit remains 362 accepted positives, 491 rejected positives and all 449 negative
fixtures rejected, with no crashes or changed compiled fingerprints.

All 21 unchanged Image probes now match actual upstream output, including the
depth-six sum 8,386,560 and shape count 4,096. All six full JavaScript programs
and all five applicable native programs match, including the original complete
Image observation. The 256 integer cases all match upstream and independent
modulo arithmetic, resolving the three earlier multiplication failures. All 186
float-oracle cases and 14 unchanged numeric programs also match. Strict custom
datatype folds at depths 12 and 13 now succeed, with five values confirmed by
actual upstream normalization. Complete 12,287-node materialization is checked
path by path; larger output still fails at the original node budget.

Reclamation implementation: `6356963d02e4c099075c4cb33edf3c8d52081ab4`.
Its retained release reports that revision with a clean worktree; all 74 compiled
source fingerprints match the audited candidate. Executable BLAKE3 is
`24f13a9421a18f51892e4402d0ddf1cc55462a3dc4082876f86c6d7ea676dd1c`.
It passes all seven Poche privacy laws and the typed wrong-viewer control, 15,503
scalar comparisons with seven equalities and two controls, and the micro
trajectory: 22 states, 21 transitions, 44 observations, 300 chance partitions
and eight controls. Its 67 requests complete in 319 ms. All 13 compiled
conformance fingerprints and 17 source fingerprints are unchanged from the
compact-word release. Poche changes remain local; exhaustive graph evidence
retains its original `6802c1e` attribution. The Image workload milestone is
complete; full Bend parity remains required and the broad goal stays active.

### [x] 5.12 Add native cooperative tasks, timers and channels

Native FIFO tasks, timers and sealed channels are implemented and validated on
Windows. Spawn continues its parent; sleep suspends even at zero milliseconds;
ready work precedes timers, and overdue timers preserve registration order.
Children outlive main. Any-task Halt preserves the full u32 API status. A driver
exit clears all tasks, timers, channel payloads and pending continuations.

The clock uses OS monotonic nanoseconds (Windows performance counter; Unix
CLOCK_MONOTONIC), retaining its origin across invocations. IO.now floors to
milliseconds as executable-only compact Nat, with the upstream native 48-bit
bound and lazy constructor views. Waits check cancellation within 100 ms without
charging idle time against the evaluation budget. Unix clock code remains
unverified on Unix hardware.

Chan.new/send/recv/close preserve FIFO buffering, zero-capacity rendezvous,
suspended senders/receivers, close wakeups and post-close buffer draining.
Generation-checked private handles prevent stale handles from addressing reused
slots; exhausted generations retire. Logical capacity accepts full U32 without
preallocation, while table entries, retained payloads and waiters have separate
aggregate bounds. Buffered payloads, pending send payloads and continuations are
traced directly by the Machine's collector. Wakes resume existing live tasks
without yielding the current task. Ordinary IO.fork/join are unchanged. See
[native channels](native-channels.md) for behavior, limits and target differences.

Native C has a distinct receiver sentinel, so erased proof/type payloads work in
both rendezvous directions. JavaScript's null collision differs for sender-first
unbuffered sends. Both backends retain their respective upstream behavior; this
is tested and documented rather than hidden by a cross-target equality claim.

Validation: the full quality gate passes 417 tests including five compile-fail
examples, with two optional local profilers ignored. Strict Clippy covers the
library and tests. The new slice adds nine channel-state tests, twelve checked
source tests and five forced-collection/cleanup tests. A final strengthened
cancellation assertion was separately rerun with all twelve source tests after
the full gate; formatting still passes. Existing clock/Nat/task tests remain.

The production Rust channel module matches verbatim upstream native C transition
functions across 509 scenarios and 45,307 operations, checking immediate results
and ordered wakes after every operation. The frozen release candidate passes
22 complete program cases: nine unchanged upstream fixtures, seven exact checked
test sources and six erased-payload probes. Nineteen match actual upstream
JavaScript stdout/stderr/status exactly, one deadlock matches status/stdout with
diagnostic wording differences, and two erased sender-first probes intentionally
follow native C semantics. Actual generated C confirms erased values lower to
zero distinct from TERM_HOLE. Complete generated C programs were retained but
not executed; the native transition oracle executes the extracted C functions.

All 256 integer comparisons and 21 original Image workloads pass. The full
1,302-fixture strict audit remains 362 accepted positives / 491 rejected positives
/ 449 rejected negatives, with zero accepted negatives, abnormal exits, changed
source fingerprints or new rejections. All 83 compiled-source fingerprints match
the frozen candidate. This strict audit does not measure whole-engine completeness.

Ignored evidence: target/audit-native-channels, target/native-channels-c-oracle,
target/native-channels-release-comparison, target/native-channels-integer-comparison
and target/image-native-channels. Retained clean releases and short Poche receipts
use target/verified-<commit>. The earlier task/timer release c293299 passed 391
tests and 13 actual-upstream program comparisons across two clock origins; its
separate receipts remain under the native-scheduler names. The prior exhaustive
Poche graph retains its original attribution and is not rerun for this slice.

Descriptor readiness, arbitrary native foreign callbacks, remaining platform
effects, executable C and the remaining CLI/kernel/GPU scope stay open. This
finishes the native task/timer/channel milestone, not the overall rewrite.

### [x] 5.13 Add native environment and file effects

Implementation: ac4a9e5. Its retained clean release matches all 91 compiled-source
fingerprints from the frozen, audited candidate. The following documentation-only
closure preserves those source fingerprints; retain its clean release and refresh
the short Poche receipts before pushing it.

IO.get_env and File.open/read/read_bytes/write/close are implemented in native
Rust and generated executable JavaScript. Exact signatures and loader origin
seal the affine File: Type contract; none of these host assumptions enters the
strict proof Base. Read/write return the live handle outside Result on failure.
Open accepts exactly r/w/a, byte reads return octets, text reads decode each
chunk independently, and close consumes its handle while ignoring close errors.

Valid native open/read/write park through an owned-data worker pool. Workers
never receive VM values or continuations; the VM packs results and resumes tasks.
Pending continuations are direct GC roots. Runnable tasks precede collected host
completions, which precede due timers. Environment lookup, invalid open arguments
and close complete synchronously. Exit removes queued work and closes owned
files. Already-running OS calls cannot be undone; their eventual replies and
resources are dropped without resuming the abandoned VM.

The process-wide pool grows to at most 64 workers, with 131,072 pending jobs,
64 MiB of retained byte buffers and an 8 MiB per-transfer ceiling. Temporary text
decoding and worker stacks are additional bounded allocations. This is not a
total-process memory cap. Windows errors use an explicit CRT-style mapping;
Unix native-byte/error handling is implemented but not runtime-validated here.
The native codec follows upstream C io_str/io_utf8 even for malformed bytes and
raw U32 character codes. Generated JavaScript retains synchronous host work,
TextDecoder behavior and the upstream raw-descriptor foreign interface. A default
Node host adapter provides file operations; supplied BEND_SYS remains authoritative.
See [native files](native-files.md) for target differences and resource limits.

Validation: ./check-all.ps1 passes 454 tests, including five compile-fail
examples, with two optional profilers ignored. Strict library/test Clippy passes.
The frozen candidate passes 18 whole-program cases through actual upstream
checking/compilation/JavaScript execution and the native executable: nine original
fixtures (temporary paths relocated where required) and nine exact checked test
programs. Thirteen have exact cross-target results; five explicitly differ in
Windows environment NUL handling, errno, malformed text or worker ordering.
An independent oracle compares the production Rust codec with verbatim upstream
C across 3,268 decoder cases and 3,017 full-U32 encoder cases: all 6,285 match.
An actual Windows CRT probe confirms the tested errno and zero-byte read behavior.
Complete generated C programs remain outside this validation.

All 256 integer comparisons and 21 original Image workloads pass. The strict
1,302-fixture audit remains 362 accepted positives / 491 rejected positives /
449 rejected negatives, with no accepted negatives, crashes, changed source
fingerprints or new rejections. This audit does not measure whole-engine
completeness. Candidate source fingerprinting covers 91 files. Ignored
evidence lives under target/audit-native-files, target/native-files-c-oracle,
target/native-files-release-comparison, target/packed-word-native-files and
target/image-native-files. Retained clean releases use target/verified-<commit>.

The clean ac4a9e5 release passes the unchanged Poche regression gates: seven
symbolic privacy theorems, three imported equalities and one well-typed rejected
mutant; 15,503 scalar cases with seven equalities and two negative controls; and
the 22-state/21-transition trajectory with 44 observations, all 300 chance
partitions, eight controls and 67 requests. All 17 source hashes, 13 compiled
conformance fingerprints, Poche HEAD and dirty paths are unchanged. Receipts are
retained in target/verified-ac4a9e5/poche-regression.json. This does not rerun or
reattribute the prior exhaustive graph evidence from 6802c1e.

This completes the bounded environment/file milestone. Descriptor readiness,
other platform effects, arbitrary native FFI, executable C and GPU remain
unfinished engine work. The overall goal stays active.

### [x] 5.14 Add descriptor readiness and TCP/UDP

Native implementation: 7887746. Exact executable contracts, native owned sockets and the
mixed timer/readiness scheduler. Network waits use the VM poller rather than file
workers; a lazily installed notification socket wakes that poller on file-job
completion. Generated JavaScript now has the corresponding effects, mixed
readiness scheduler and Rust Node-API provider. This bounded milestone is
validated on Windows; Unix implementations still need platform qualification.

Work: extend the existing native scheduler with bounded readiness registrations,
sealed affine Socket/Listener contracts and all eleven TCP/UDP/close effects.
Keep Poche model expansion deferred. Socket waits must not occupy file workers:
the upstream slow-peer fixture parks 70 senders while timers and file work remain
responsive, exceeding the 64-worker pool. Retain ownership of handles/resources
and root pending continuations; release registrations/resources on close, Halt
and cancellation.

Preserve upstream ordering: TCP.accept, TCP.recv and UDP.recv_from register and
park before their first syscall. Connect/send attempt immediately and park only
when necessary; UDP.poll returns immediately. Preserve strict numeric IPv4
addresses, ports at most 65535, rejected leading zeros/NUL, handles outside
Result, partial TCP sends, short receives/EOF, datagram truncation and sender
address/port. Use Winsock ownership/errors on Windows rather than treating
sockets as CRT file descriptors. Decide and document platform adaptation before
claiming support; apply owned Windows socket patterns where needed.

Reference entry points: upstream comp.ts io_sys_addr, io_wait_on, io_wait and
IO_READ dispatch; effs/tcp_*.c, udp_*.c, socket_close.c and listener_close.c;
tests/io/tcp_* and udp_*. In this port, start from runtime/executable.rs,
runtime/scheduler.rs, runtime/runtime_clock.rs and the file-job completion/GC
integration. Generated JavaScript uses a synchronous syscall/readiness
provider preserving the raw-descriptor and callback ABI. Keep supplied BEND_SYS
authoritative. Do not resume VM continuations from asynchronous host callbacks.

JavaScript implementation: maintain bounded descriptor waits alongside timers, poll
only after runnable work is drained, and preserve their common registration
order. Error/hangup readiness must resume the operation; EAGAIN reparks it.
Preserve upstream's distinction between explicit write parking and the initial
foreign `need.read` request, which does not park solely for `need.write`.
The selected default Node provider is a Rust Node-API addon in native/node-sys.
A minimal napi-rs prototype loads on the current Node host without a Node import
library, using maintained dynamic-symbol bindings. The provider owns sockets it
creates while exposing actual OS descriptor values to foreign code. Its portable
syscall interface uses Linux-style constants; a full-width poll_descriptors
extension preserves Windows SOCKET values, and legacy POSIX pollfd remains
available for compatible supplied providers and upstream comparison programs.
The generated loader keeps supplied BEND_SYS authoritative and can load an
explicit TEAMY_BEND_SYS_MODULE. The CLI packages a built sibling provider beside
generated programs without embedding machine paths or silently replacing a
different existing provider. Normal builds include both workspace crates.
Deterministic descriptor/timer traces and actual upstream JavaScript loopback
programs pass through this provider. Source entry points:
compiler/executable_io.js, executable_host.js, executable_network.js and
cli/compile/compile_cli.rs. Native-addon execution is validated on Windows with
Node 24; Unix execution remains unverified.

Native validation: ./check-all.ps1 passes 487 tests, including five compile-fail
examples, with two optional profilers ignored; strict library/test Clippy passes.
Nine public networking tests, ten host tests and seven private VM tests cover
the new path. Six private tests force GC. Actual OS backpressure is covered by
70 parked sends plus a worker canary and a separate 512 KiB send that completes
with exact bytes. A checked source fixture covers 70 receivers, real file work,
a timer and Halt cleanup. See [native networking](native-network.md) for the
test boundaries, including the distinction between initial parking and a
deterministically observed partial-send retry sequence.

The frozen candidate passes all eight CLI networking cases, 256 integer
comparisons, 21 Image workloads and 18 environment/file cases with their existing
target qualifications. The production IPv4 parser matches 3,090 upstream C
address cases. Six actual-loopback host groups match the adapted upstream C
effects; three additional scripted C send-transition groups and adapter cleanup
pass separately. This is not whole generated-C runtime equivalence.

All 97 compiled-source fingerprints remain unchanged during validation. The
strict 1,302-fixture audit remains 362 accepted positives / 491 rejected positives
/ 449 rejected negatives, with zero accepted negatives, abnormal exits or new
rejections. Ignored evidence: target/audit-native-network,
target/native-network-address-oracle, target/native-network-c-effects,
target/native-network-release-comparison, target/packed-word-native-network,
target/image-native-network and target/native-network-file-regression.

The retained clean 7887746 release matches all 97 candidate source fingerprints.
Its existing Poche regression gates pass: seven symbolic privacy theorems, three
imported equalities and a well-typed privacy mutant; 15,503 scalar comparisons,
seven equalities and two controls; a 22-state trajectory with 44 observations,
21 transitions, 300 chance partitions, eight controls and 67 requests. The exact
trajectory transcript, all 17 source hashes, 13 compiled conformance fingerprints,
Poche HEAD and dirty paths remain unchanged. Receipts are retained under
target/verified-7887746. This does not rerun or reattribute the prior exhaustive
graph from 6802c1e. No Poche source or application changes were made.

JavaScript validation: the workspace quality gate passes 513 tests, including
five compile-fail API examples, with two optional profilers ignored. Strict
workspace library/test Clippy and the direct Node provider test pass. New
integration coverage comprises eleven networking, nine readiness and five
ownership tests, plus the provider's descriptor-conversion unit test. These
cover real TCP/UDP peers, raw descriptor exchange, truncation and zero-byte
datagrams, mixed timers/duplicate waits/re-parking, frozen provider receivers,
managed raw-close aliases, provider changes, bounds and CLI packaging.

The frozen JavaScript candidate matches 17 actual upstream checked programs:
three deterministic syscall scripts, five real peer/refusal/raw-descriptor
groups and nine upstream networking fixtures with ephemeral-port adaptation.
Seven actual upstream scheduler traces match observable event ordering,
including the final timer-only wait after descriptor work drains. These do not
claim upstream Bun/FFI or Linux/macOS validation. Descriptor-only waits use
bounded 1,000 ms polls instead of upstream's indefinite -1. The stable managed
io_sys view intentionally differs in object identity from BEND_SYS; callers
bypassing that view also bypass invocation cleanup bookkeeping.

All 106 compiled-source fingerprints remain unchanged during the frozen audit.
The 1,302 strict fixtures retain 362 accepted positives / 491 rejected positives
/ 449 rejected negatives, with zero accepted negatives, abnormal exits or new
rejections. Ignored receipts: target/audit-js-network,
target/js-network-release-comparison, target/js-readiness-release-comparison
and target/js-network-release-provider.log. Previous native integer/Image/file
evidence remains attributed to its prior retained release, verified-a5c0c6c.
Clean release and Poche regression retention follow these candidate checks.

This completes the bounded native and JavaScript networking slice. Continue
with executable C (5.15) and keep Poche model expansion deferred.

Completion: supported network effects preserve results, readiness ordering,
ownership, cancellation and bounded resources on validated targets. Then advance
to executable C: its foreign registration/callback ABI, constructor IDs/aliases,
native representations, import deduplication and portable initialization remain
required. Pure C compilation does not establish that interface. Window/audio and
GPU remain later engine work; consult the saved GPU references when that phase
begins.

### [ ] 5.15 Implement executable C and its foreign/runtime ABI

The current compiler/c.rs path lowers a strict Book into the separate lazy
Value/Thunk constructor runtime in c_runtime.c and prints JSON. The CLI rejects
executable C. Extending its console dispatch would not provide the upstream
foreign ABI. Start a separate executable_c backend over the existing typed
ExecutableProgram; expose the target-neutral lowering currently named
lower_for_javascript instead of duplicating checking.

First coherent deliverable: native-value layout and constructor-ID assignment,
C foreign-source assembly, and the Effect/IoWork continuation/activation runtime.
Preserve the packed Term/Env heap interface used by actual upstream C imports,
including native words, float bits, characters, Nat, strings, tuples, Result,
Maybe, Bool, Unit, closures and arrays. Sharing pure front-end logic does not
make the existing Value-pointer C representation compatible with that ABI.

Share live-reference discovery and canonical import deduplication with the
JavaScript assembler. Select the first C import, include each canonical source
once in reference order, and scope constructor aliases around each import.
Validate constructor IDs, field/parameter arities and mangled-name collisions.
Reference: upstream comp.ts compile_reqs/compile_tables, the Term layout around
3332, native representations around 3766, and Effect/IoWork around 5171.

Implement request ownership before specializing bundled effects: the final
request field is the continuation, an effect returns a Term or IO_PARK, and
worker call functions must not access VM memory. Packing/resumption and read/time
parking belong on the event loop. Carry forward bounded buffers, registration
order, re-parking, cancellation and resource cleanup. Upstream's io_hand has a
56-bit payload; explicitly reject an unrepresentable host handle or provide a
qualified bridge instead of truncating it.

Plan Windows host adaptation explicitly: upstream assumes pthread, poll, pipe
and POSIX sockets. The Node addon establishes relevant syscall contracts but is
not a standalone C runtime dependency. Preserve effect registration: upstream
sources use constructor attributes to call io_eff. Qualify a supported C
toolchain or implement a reviewed explicit initializer path; stripping the
attribute without invoking those functions is not equivalent.

Validation: compile and run actual emitted C using the existing compiler_c.rs
toolchain harness. Compare native values, closures/arrays and effects against
upstream C, generated JavaScript and the Rust VM as applicable. Custom imports
must exercise constructor aliases, shared-file deduplication, initializer order,
callback-produced data, raw handles and readiness re-parking. Then carry the
channel, file and TCP/UDP suites across, including more than 64 waits, partial
sends and cleanup, with malformed-layout/registration negatives. Console output
is an early smoke test, not completion of this milestone. Poche model expansion
remains deferred; window/audio and GPU remain subsequent engine work.

## Completion and risks

The goal is complete only when U1–U13 are delivered and no required rewrite or
formalization work remains. A scaffold or supported language subset is progress.
Proof soundness risk is controlled by preserving erasure/resource/descent rules,
rejecting unsupported constructs and testing false proofs. Model correspondence
risk is controlled by independent conformance and negative controls. Reference
contamination is controlled by read-only checkouts and tracked template selection.
Hardware-dependent coverage must be marked unverified when unavailable.
