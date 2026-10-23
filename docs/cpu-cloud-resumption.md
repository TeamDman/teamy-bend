# CPU cloud resumption

Updated 2026-10-23. Development was parked because other work took priority.
The resumed objective is still the Bend 2 rewrite, followed by the remaining
Poche formalization; the first new milestone is a reproducible Linux CPU
baseline. GPU execution and sanitizer runs are deferred to a local hardware
session. The first Linux milestone now passes; see the
[Linux CPU validation receipt](linux-cpu-validation.md) for source identity,
exact commands, counts and remaining boundaries.

## Preserved work and evidence boundary

Commit `b7eac20` preserves the previously unpublished candidate exactly as Git
content: resumable CUDA constructor-field extraction, pointer-reversal drops,
scope/argument cleanup, nested finite-layout conversions, tests and progress
notes. The remote documentation commit `969561f` is merged into this branch.
The original desktop checkout and private continuation notes remain separate.

The nested-layout candidate initially failed its existing CPU regression with
`invalid finite C layout discriminator`. Single-constructor finite layouts have
no tag word. The resumed implementation emits a discriminator switch only for
multiple constructors, matching the established synchronous converter. The
same CPU regression passes after this correction. The candidate's CUDA
behavior has not been revalidated during this resumption.

The recovered drop and nested-layout fixtures, with identical console wrappers,
match the pinned upstream JavaScript, Rust native runtime and generated
JavaScript byte-for-byte: `16\n` and `17\n`, empty stderr and exit 0. Their
original generated-C CPU regressions remain in
`tests/compiler_executable_c_gpu.rs`. This small comparison does not replace
the pending complete upstream-C execution on Linux.

Fresh Windows CPU validation on 2026-10-23 passed `./check-all.ps1`: formatting,
strict Clippy, all-feature build, the real Node socket-provider contract suite,
and 728 tests with 49 explicit ignores and zero failures. GPU hardware tests and
optional profilers were not enabled. The separately pinned
`nightly-2026-05-24` formatting check also passes. Linux qualification is recorded
separately in the linked receipt; these Windows results must not be relabeled
as Linux or GPU evidence.

Plan items 5.15.7.4.g/h retain their historical September validation; those
receipts do not qualify the later nested-layout changes. The existing strict
checker audit and exhaustive bounded Poche results retain their original
revision attribution. Do not rerun them or claim new coverage merely because
the work has moved hosts.

## Dedicated headless environment

Use this repository's published resumption branch/commit, with a separate
read-only checkout of `bendlang/bend` at
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599` (Bend 2.0.5).
Do not substitute current upstream main. Preserve upstream notices and the
MPL-2.0 / Apache-2.0 file boundaries recorded in `NOTICE`.

Also include `TeamDman/teamy-docs` in the environment as the reference for
established Teamy development conventions. Read its current `AGENTS.md`, CLI,
Rust and Facet guidance, and the relevant Teamy CLI implementations before
designing changes. Keep CLI/dependency migration separate from preservation.
Investigate direct Serde use and significant transitive consumers against the
owned Facet fork; do not mechanically replace dependency internals or weaken
compatibility tests. Prefer cloud/local verification; no GitHub CI workflow is
added by this handoff.

Prepare these tools and record their exact installed versions in the setup
receipt:

- Rust 1.96.0 and Clippy, pinned by `rust-toolchain.toml`; preserve `Cargo.lock`.
- `nightly-2026-05-24` with rustfmt for formatting. The gate accepts
  `-FormatterToolchain` so the environment need not track mutable nightly.
- Node 24.19.0, the desktop comparison version. Node executes the upstream
  TypeScript CLI directly; Bun is not needed for this CPU slice.
- PowerShell 7 for the existing quality gate.
- Clang 14 or newer, the POSIX C development headers, pthread, math and dynamic
  loader libraries. Pin the chosen package versions/image after Linux setup
  succeeds; no particular Linux Clang release has been qualified here yet.
- Git and the normal linker/build tools needed by Cargo dependencies.

Use a CPU environment without CUDA tooling. The selected fixtures need no
display, audio device, public listener, database or application secret. Keep
socket contract tests on local loopback. Permit the required Rust package and
toolchain hosts, GitHub source hosts, and chosen OS/Node/PowerShell package
repositories during setup; record the actual destinations rather than enabling
unrestricted network access by assumption. Git dependencies are revision-pinned
in `Cargo.toml`/`Cargo.lock`.

From this repository, after installing the pinned tools and dependencies:

```sh
cargo fetch --locked
export BEND_GPU=off
export CARGO_NET_OFFLINE=true
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

The gate builds all features, checks the native Node socket provider and tests
without Tracy transport. Hardware-dependent CUDA cases remain ignored; never
add `--ignored` or `--include-ignored` to this CPU gate. The Node provider has no
node-gyp or separate Node-header requirement.

## First Linux acceptance milestone

1. Record the port SHA, pinned upstream SHA, OS/compiler/tool versions and
   resource limits. Establish the portable gate on Linux, including the Unix
   file, clock, socket and Node-provider paths. Report unavailable or skipped
   tests explicitly. Repair actual portability failures without loosening proof
   checks, expected output or resource limits.
2. Reuse the recovered nested-layout/drop fixtures and the existing CPU suites
   for arrays, typed calls, tails, parallel tasks, callbacks and effect ordering.
   Add a small representative set of unchanged upstream fixtures. Existing
   Windows/upstream-JavaScript comparisons are already documented; this
   milestone adds execution of the complete untouched upstream POSIX C runtime.
3. Generate upstream C from the pinned source and compile it with Clang C11,
   pthread and math libraries. Generate the port's executable C for the same
   source, compile it with strict warnings, and compare stdout, stderr and exit
   status under bounded CPU execution. Run upstream generation from its
   `bend2` directory so relative effect companions resolve. Use separate
   temporary output directories and disable GPU execution for every child.
4. Record each fixture's source hash, commands, output/exit comparison, timeout
   and platform-specific differences. Compare pure native values structurally,
   or use the same console wrapper in both runtimes: native `run` deliberately
   prints canonical constructors for a pure result, unlike generated C/JS text.
   Matching rejection alone is not semantic parity. Preserve false-proof and
   resource-limit regressions.

Acceptance is a reproducible passing Linux gate plus a reviewed, bounded
upstream-C comparison receipt. It is not whole-engine parity, a speedup claim,
GPU qualification, or completion of the remaining Poche formalization.
Only after that baseline should the next CPU implementation gap be chosen from
observed failures. GPU demand backing, sizing/residency, remaining helper
suspension and Metal remain separate work in the implementation plan.
