# Bend Rust rewrite and Poche formalization

**Plan status:** Active
**Primary implementation root:** `teamy-bend` repository
**Last updated:** 2026-09-19
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
| U6 | Rewrite Bend in Rust. | 2.1–3.2, full compatibility remains open |
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

Completion notes: 290 selected pure upstream source declarations are bundled
with Apache attribution (254 definition forms including 14 templates, 18 laws,
18 datatypes). Template instances enter the checked book at their call sites;
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

Remaining library work includes the pure Array operations, complete numeric
behavior, effects and foreign implementation contracts. Representation or
syntax support alone does not establish an operation's implementation.

Work: base library, templates, effects, packaging and supported CPU/GPU targets;
record unavailable hardware and platform-specific validations accurately.
Validation: compatibility matrix with observed results, not assumed parity.
Completion: U6 has a documented complete scope and verified coverage; any
remaining unsupported areas keep the broad rewrite goal active.

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

## Completion and risks

The template/library publication slice passes the complete quality gate
(124 tests and two ignored local profilers), plus the subsequently added
deferred-template regression (13 template tests pass). Independent review
found no new defect in specialization keys, closure checks, declaration order,
budgets or affine collection behavior. `base` source/name/type output and
invalid selectors passed native CLI smoke checks. All 1,302 upstream fixtures
were audited against a fixed executable while development continued separately.

The goal is complete only when U1–U9 are delivered and no required rewrite or
formalization work remains. A scaffold or supported language subset is progress.
Proof soundness risk is controlled by preserving erasure/resource/descent rules,
rejecting unsupported constructs and testing false proofs. Model correspondence
risk is controlled by independent conformance and negative controls. Reference
contamination is controlled by read-only checkouts and tracked template selection.
Hardware-dependent coverage must be marked unverified when unavailable.
