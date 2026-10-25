# String literal pattern depth investigation

The original `parse/string_pattern_sugar.bend` remains unresolved. Production
source is unchanged. This checkpoint preserves a rejected-for-production
experiment, bounded neighboring regressions, and the precise next blocker.
The clean-source gate and bounded comparisons are complete; their receipts follow below.

The pinned original matches `"hi"` and returns `True{}` upstream. The port refuses
loading at its 64-level pattern-compilation guard. Literal expansion is
`SCon` / `Chr` / `U32` / 32 `WCon` decisions per character. Compiler frames for the
first character retain unfinished arms and fallbacks while processing the next.
The existing guard is a local recursion/pending-work bound, not a global count
of all simultaneously retained states; a single-character pattern can have 69.

The unaccepted [fragment experiment](cpu-string-pattern-depth-experiment.patch)
retires a completed field's compiler working states before lowering the next
field. It retains the numerical guards, uses an Rc-identity continuation, and
keeps immutable output. Review identified a nested deferred-error ordering risk;
the archived version disables nested fragmentation. It is **not applied** and
has not passed comprehensive graph/provenance/error-order or full-gate review.
It must not be treated as a production correction.

That candidate loads the unchanged original, but check/eval/run and both emitters
then refuse at the separate **128-level kernel input guard**. Complete alpha-tree
comparison with the pinned parser yields `is_hi`: **depth 138 / 610 nodes** (root depth 0);
`main`: **depth 37 / 139 nodes**. Comparison includes every node, constructor, child order and
bound-variable index; it excludes binder spelling, source metadata and Rc sharing.
The graph probe fails on any unsupported node tag. The complete canonical `is_hi`
stream SHA-256 is `5d4ecab6cb8eeab4ad1c637303f25c6fab361d1002cb92eec1339d5b85023c69`.

Graph-preserving iteration or sharing cannot shorten a root-to-leaf path. Neither
skipping shared nodes in validation nor changing the threshold is an acceptable
repair. The next substantive task needs a reviewed design for compact lowering
that changes the intermediate graph while preserving checking, normalization,
binder/provenance, evaluation and safety behavior. No such design or resource
policy change is implemented here. Independent review recommends archiving the
experiment and retaining the refusal until that work is justified.

The seven positive controls cover empty/ASCII/supplementary-character matches,
mismatches, and longer-prefix rejection. They do not qualify the two-character
literal. Negative controls retain the 65-Nat and 65-column compilation refusals
and rejection of a String pattern on a Nat. The wide-column control also lacks
cases upstream: that is a phase difference, not matching diagnostics.

Reproduction uses `scripts/string-pattern-depth/compare.py`, the standalone
`phase-probe.rs`, `graph-probe.rs`, and `upstream-depth.mjs`. Rust probes link to
the exact matching built library; no dependency is installed. Host debug=0 and
incremental=0 settings control cache size, not Bend semantics. C uses the pinned
upstream O3 profile, CPU threads=1/GPU=off; port C additionally uses strict warnings.

The raw instrumentation receipt has an incorrect `base` field. Its correct base
is `6c2ba98c964247fc424a1d57ef2772dbfe2a16eb`; raw bytes remain preserved with this
explicit correction. Dirty experiment builds are never attributed to a clean
commit. Raw binaries/logs remain outside Git; compact source, patches and unique
failure receipts are preserved in the reviewed checkpoint.

## Clean-source validation

Tested source: `a6a62afea10d57d6142855a3ceaf2d71f9e14c26`.
The final receipt commit updates documentation and installs the corrected
comparison harness; production and regression-test trees are unchanged.
[Exact observations and content-addressed streams](cpu-string-pattern-depth-evidence.json),
[experiment/provenance evidence](cpu-string-pattern-depth-experiment.json), and
[validation manifest](cpu-string-pattern-depth-validation.json) are checked in.

| Check | Result |
| --- | --- |
| Standard gate | 1,168 passed / 0 failed / 49 ignored |
| New literal/negative/guard regression tests | 3 passed, included in gate |
| Seven companions, five executable lanes, two repetitions | 70 exact executions |
| Strict/executable/upstream phase probes | 66 observations |
| Raw checking CLI observations | 22 observations |
| Entire bounded matrix | 278 observations, zero timeouts |
| Missing generated-artifact lanes | 12 explicitly unrun records |

The positive five-lane comparison is native Rust, port C/JS and upstream C/JS.
Pure interpreter output and port eval's JSON envelope are separate observations.
Every repeated observation had identical status/stdout/stderr. Wrong-type pattern
rejection matches upstream's checking phase and exact raw diagnostic. Nat65 is
accepted upstream and refused at port load; columns65 rejects upstream at checking
for missing cases and at port loading for depth. These boundary controls preserve
safety behavior, not compatibility exceptions.

The original still has upstream check/interpreter/C O3/JS success (`True{}\n`,
exit 0, no stderr). Port check/eval/run and both emitters exit 1 at loading;
port C/JS execution is unrun because no artifacts exist. In the **unaccepted
candidate only**, loading succeeds but check/eval/run and both emitters instead
exit 1 at the 128 input guard. No original runtime correction is claimed.
The candidate's paired32-Nat observation succeeds, but is not enough to accept
the compiler transformation or qualify the original.

Retained failed attempts: the first gate was stopped by its task-owned disk
monitor at the 10 GiB floor, with unit testing incomplete; the initial matrix
refused the same floor before any observations. The next matrix stopped after
110 observations because its harness incorrectly expected raw stdout from eval.
The corrected harness validates the exact JSON envelope instead. No comparison
was weakened or production stream changed. Its SHA identifies the exact saved
script used for the complete matrix; the final script has identical bytes.
The preliminary targeted Clippy run found two test assertions, corrected without
suppression; the renewed targeted check and standard production Clippy pass.

The failed gate exposed combined build-cache growth. Before restarting, inactive
Rust incremental caches were removed and current dependency build artifacts were
relocated to temporary storage, preserving all artifact bytes/hashes and the
original Cargo pathname via a symlink. Headroom became 11.96 GiB workspace / 5.13
GiB temporary storage. Both disk floors remained active; source and unique
historical evidence were untouched. The cache relocation is a local build setting,
not a repository/toolchain/dependency or Bend runtime change.

The 1,302-row full census was not rerun because production source is byte-identical
to the last measured source. This task renewed the affected original and ten
companions only. Historical census remains at `d224e79d33b4a0ab26f701b57f448056b63b790a`:
**300 differences beyond 385 historical IO exclusions, plus six upstream stack
failures**. The original acceptance disagreement remains within that count.
Ledger coverage remains **420 linked / 882 unlinked**: 527 interpreter-and-compiled
candidates, 341 checker negatives, 13 interpreter-profile cases and one
checking-only module. No failed original was promoted to qualified coverage.
F32.read, failure ordering, erased-hole/TODO, printer/oracle, host/foreign,
resource and external-API gaps remain unwaived. Historical supplementary
all-target Clippy failures and the incomplete 688-vector exploration remain open.
There is no mid-normalization cancellation claim.

## Reproduction

```bash
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test cpu_string_pattern_depth
cargo clippy -p teamy-bend --locked --all-features --test cpu_string_pattern_depth -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Link `phase-probe.rs` to the matching default+extended-observability Rust library:
`rustc --edition=2024 -C debuginfo=0 scripts/string-pattern-depth/phase-probe.rs
--extern teamy_bend=<matching.rlib> -L dependency=target/debug/deps -o <phase-probe>`.
Supply a JSON build receipt containing `tested_source`, `binary_sha256` and
`phase_probe_sha256`; `compare.py` requires these to match the clean checkout and
supplied artifacts, and requires a clean pinned reference.

```bash
python3 scripts/string-pattern-depth/compare.py \
  --reference <pinned-checkout> --binary <tested-binary> \
  --phase-probe <matching-phase-probe> --build-receipt <build-receipt.json> \
  --output <new-evidence-directory>
node scripts/string-pattern-depth/upstream-depth.mjs \
  <pinned-checkout>/bend2/bend.ts \
  tests/fixtures/cpu-string-pattern-depth/original/string_pattern_sugar.bend
```

The port graph observation requires the archived experiment in a separate
checkout at its recorded base; **do not apply it to production as an accepted
fix**. Link `graph-probe.rs` using the same rustc pattern, then pass the original
fixture. Compare the complete canonical streams, not only node counts.
The experiment remains incompletely validated despite that exact alpha-tree match.

Pinned source basis: [literal Word construction](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1132),
[string expansion](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1939),
[constructor pattern conversion](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1690),
and [definition lowering](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2506).
No dependency, upstream, toolchain, guard, budget, cancellation or demand-policy
change was made. No desktop/GPU, installation, merge or release was performed.
Raw captures and binaries remain outside Git in the task's `cpu-string-pattern-depth`
evidence directory; Git preserves the compact evidence and unaccepted source patch.

The [diagnostic-only instrumentation patch](cpu-string-pattern-depth-instrumentation.patch)
reproduces the retained working-state measurements at the recorded baseline.
Its test records successful execution of a probe even when loading is refused;
that test status is never counted as a passed parity fixture.
