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
the user's public pseudonym and GitHub noreply identity. First push pending.

### [x] 1.3 Establish compatible licensing and provenance

Completion: upstream Apache-2.0 text preserved in `licenses/Apache-2.0.txt`;
template MPL-2.0 text retained as `LICENSE`; attribution in `NOTICE`.
Authoritative references: [Mozilla FAQ Q13](https://www.mozilla.org/en-US/MPL/2.0/FAQ/#q13-may-i-combine-mpl-licensed-code-and-bsd-licensed-code-in-the-same-executable-program-what-about-apache)
and [Apache licence section 4](https://www.apache.org/licenses/LICENSE-2.0).
Translated files keep Apache licensing; new files use MPL.

### [x] 1.4 Adapt and validate the Rust CLI template

Completion notes: identity/env names, CLI commands, profiler default and README
adapted. Logging, structured output, cancellation, Windows resources and CLI
fuzzing retained. `check-all.ps1`, help/version and native command tests passed.

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
operators, do/array sugar and local imports implemented; 14 parser tests pass.
Templates, foreign bodies, GPU calls, hub packages and implicit array-write
rebinding remain open. Exact supported surface/limits: `docs/compatibility.md`.

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
`compile examples/induction.bend` produces a program returning Nat 5. C, effects,
upstream optimization strategy and GPU runtime remain open.

Work: map `comp.ts` IR, lowering, code generation and runtime functions to Rust;
cover upstream C and JS behavior before GPU runtime integration.
Validation: upstream compiler/runtime/effect fixtures against actual outputs.
Completion: the Rust tool can compile and run the advertised target set.

### [~] 3.2 Finish language/library and target coverage

Completion notes: 114 selected pure upstream Base declarations are bundled with
Apache attribution. Six Base regressions cover dependent witnesses, Boolean
proofs, open equality transport, polymorphic lists, division and false claims.
Full Base and target coverage remain required.

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

### [ ] 4.3 Extend the independent model to state transitions

Design handoff: `docs/poche-state-model-design.md` specifies exact public-getter
adapters, typed constructor protocol, state/action schema, deterministic replay,
431,800-state/549,896-edge reference and negative controls. Implementation has
not started; scalar success does not mark this task complete.

Work: phases, exact card partition, immutable bids, trick progression, winner
leads, score/pot separation, absorbing finish and decreasing progress. Compare
the existing shared 1/2 prefix and then the full micro schedule where supported.
Validation: finite state/transition comparisons with exact model bounds and
counterexamples; preserve normative rules and conventional authority.
Completion: reviewed scope/evidence accurately separates kernel laws, bounded
state checking and unproved full-deck/network/UI behavior.

## 5. Validation and publication

### [~] 5.1 Publish the reviewed implementation and evidence

Completion notes: public remote exists; first milestone review excludes local
paths, raw diagnostics, build outputs and reference downloads. Full quality gate
and final upstream audit passed. First commit/push pending. Both source and
Poche remotes were verified public; Poche itself is not being published here.

Work: README, compatibility status, docs and licence headers; inspect all staged
content and commit identity; commit and push the requested public repository.
Do not publish local discovery output, captures, private paths or Poche changes
incidentally. Poche changes can remain locally reviewable unless its workflow
or the user authorizes their publication.
Validation: quality gate, help/version, real Poche conformance command,
publication scan, remote visibility and clean commit verification.
Completion: public repository contains the verified implementation and honest
limitations; Poche integration is runnable and evidence is reproducible.

## Completion and risks

The goal is complete only when U1–U9 are delivered and no required rewrite or
formalization work remains. A scaffold or supported language subset is progress.
Proof soundness risk is controlled by preserving erasure/resource/descent rules,
rejecting unsupported constructs and testing false proofs. Model correspondence
risk is controlled by independent conformance and negative controls. Reference
contamination is controlled by read-only checkouts and tracked template selection.
Hardware-dependent coverage must be marked unverified when unavailable.
