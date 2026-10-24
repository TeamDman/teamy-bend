# No-main modules and multi-scrutinee CPU qualification

Tested source **`2417a27107e4adb5d5546ace6cd4e854fc2a0a92`**, following checkpoint
`c5b5013fbf1d4fc86b1d98d620dfe10d21d3399d`, adds detailed qualification for
nine checking-only modules and seventeen interpreter-only originals. Target:
**`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`**. The originals are unchanged,
hash-verified copies with upstream Apache-2.0 attribution. No dependency,
toolchain, guard, budget, cancellation or evaluation-policy change was made.
Full CPU parity remains incomplete.

## Corrected standalone compilation boundary

Both implementations accept all nine modules. Default upstream invocation and
port `run` report `All terms check.` with empty stderr and status 0; there is no
main execution to qualify. Strict checking also succeeds, and the thirty
source-defined functions normalize exactly in structure and text.

Artifact applicability exposed a port defect: executable C/JS generation emitted
successful checking-only programs even though the pinned standalone emitter
refuses a missing main. The initial receipt preserves eighteen successful port
emissions and the upstream refusals. Both port executable emitters now reject a
missing entry before constructing output or assembling foreign source. Native
compilation inherits the C rejection. The CLI regression covers all three
targets and verifies `--force` cannot overwrite existing output after rejection.
Checking, named-definition normalization, native no-main reporting and the
separate strict data compiler's named-entry facility are unchanged.

The source distinction is explicit: pinned
[book_run](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/main.ts#L454)
reports successful checking when there is no entry; pinned
[show_main](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1203)
refuses standalone generation. Seven originals reach `no main to run`; two
without Base (`constructor_discriminate`, `gadt_refutation`) reach upstream's
`a build needs import Base` prerequisite first. No upstream file was modified.

This intentionally changes the behavior of public `compile_executable_c` and
`compile_executable_javascript`: a checked book with no main now returns Err
instead of an Ok checking-only program. Their signatures are unchanged. The two
old success regressions were corrected against the pinned emitter contract;
checking success is separately retained, not deleted or weakened. External
consumers expecting the old emitted stub remain unaudited. The prior CLI exit-code
and KernelError construction compatibility caveats remain.

Compilation **raw diagnostics remain different**: the port uses its existing
host-formatted CompileError boundary; the two no-Base originals additionally
have a different prerequisite diagnostic. Rejection and absence of artifacts
are qualified; exact compilation stderr parity is not. No exception waives it,
and no blanket Base requirement was added to the port's existing compiler.

## Checking and normalization evidence

| Original | Named definitions normalized |
| --- | ---: |
| page/compile_no_main.bend | 1 |
| page/motive_binder_name.bend | 2 |
| proof/ackermann_lower_bound.bend | 7 |
| proof/constructor_discriminate.bend | 4 |
| proof/function_congruence.bend | 1 |
| proof/gadt_refutation.bend | 7 |
| proof/map_fusion.bend | 3 |
| state/clean_typecheck_000.bend | 2 |
| state/clean_typecheck_001.bend | 3 |

The final no-main matrix records **306 observations**, twice per lane, using
30-second child deadlines:

- 36 exact default checking reports and 18 successful typed checks;
- 60 pinned named normalizations, 60 matching typed eval results, and 60 exact
  actual-core/text comparisons covering all thirty definitions;
- 72 verified upstream/port C/JS emission refusals with no output artifacts.

All observations are deterministic, with no timeout or abnormal exit. The
structural comparison helper previously refused the multiplicity values in
`map_fusion.fuse` and `clean_typecheck_001.Vec`. It now represents None/Lone/Many
as distinct values. It still rejects unqualified syntax and performs no
reduction, reordering or text repair. Regressions distinguish all three
multiplicities, including their occurrence in datatype parameters. Production
terms, serializers, normalization and comparison depth limits are unchanged.

Two authored false-proof controls are preserved separately. A wrong Even index
rejects with exact upstream stderr. Changing map fusion's composed increment
from 1 to 2 also rejects, but exposes an **unresolved rewrite type-mismatch
diagnostic**: upstream normalizes both expected/observed types and identifies
`fuse` line 52; the port retains an unnormalized expected type and host context.
Four direct phase probes confirm successful loading and rejection during checking;
this is not an acceptance
or evaluation mismatch. Its exact streams/statuses are retained, with no waiver.

## Multi-scrutinee interpretation

The coherent second batch is `flatten/multi_scrutinee_000` through `_016`:
seventeen function-valued mains with two/three pattern columns, fallback order,
reconstructed constructors and repeated binder names. All match actual pinned
interpreter output, strict normalized core, typed eval and native pure output.
Original `#|` expectations were checked against actual upstream execution.

All originals omit Base, so upstream generation first refuses that prerequisite.
The port's emitters refuse the unprintable function result. Separate authored
probes add Base and prefix local datatype/constructor names to avoid collisions;
all seventeen then reach the upstream function-printer refusal in C and JS,
twice (**68 refusals**). Initial probes that collided with Base's Nat and None
are preserved as harness-construction failures; they are not printer evidence.
No compiled execution is claimed for these functions.

An independent direct-core upstream adapter applies the original functions to
**100 concrete argument vectors**, twice (200 value observations). Rust tests
apply the same checked original definitions and compare exact values and
structure. These vectors cover all zero/successor or empty/present combinations
used by this family, including reconstructed fallback arguments. They do not
establish exhaustive equivalence for arbitrary inputs.

The reusable `compare-normalization.ps1 -Suite multi-scrutinee` selection adds
the seventeen originals; `-Suite all` now contains 66 cases (59 originals and
seven authored controls).

## Validation at the clean tested source

| Check | Result |
| --- | --- |
| Standard gate, including strict formatting/Clippy/build/provider | **876 passed, 0 failed, 49 ignored**; 98 result-bearing targets; exit 0 |
| Focused four-target candidate run | 57 passed before commit; same regressions included in the clean full gate |
| Full normalization suite | 66 cases; 522 successful observations, 192 verified build/printer refusals, six strict F32 exclusions; exact structure/text |
| Serial baseline | 50 exact executions across upstream C/JS and port native/C/JS, two repetitions |
| Pinned corpus checking-status audit | 1,302 rows; zero transitions from d986964 |
| Added negative controls | Eight comparisons plus four direct phase probes; both reject, one exact diagnostic and one unresolved diagnostic |

All 439 expected checker negatives still reject; other audit results remain
366 accepted / 497 rejected. Those 497 rejections are status evidence, not
qualified parity. No ignored hardware/profiler case ran. Existing negative/demand,
constructor-diagnostic and false-list/brace matrices keep their actual earlier
receipts; they were not relabeled as newly executed. Their regression tests
remain in the standard gate. No test is left running.

## Coverage and remaining work

| Category | Inventory | Detailed originals | Without detailed receipt |
| --- | ---: | ---: | ---: |
| Checker negative | 439 | 8 | 431 |
| Checking only | 12 | 11 | 1 |
| Interpreter only | 164 | 32 | 132 |
| Compiled candidate | 687 | 115 | 572 |
| Total | 1,302 | 166 | 1,136 |

Detailed coverage increases by 26 originals; it includes historical and unresolved
cells, not universal parity. Runtime/interpretation union: 147. The 59
interpretation rows, nine new checking rows and three runtime originals carry
this source receipt; ten checking and 85 runtime rows remain historical.
`io/far_types` is the sole checking-only module still without a detailed receipt,
and needs its separate foreign-mode applicability review.

The [acceptance criterion](cpu-parity-acceptance.md) is unchanged. Full API/lane
review, F32.read, competing-error ordering, whole-program resource limits, native
foreign support and unavailable Bun/host oracle lanes remain open. The added
compile and false-fusion diagnostic differences are explicit unmet cells.

Next bounded correction: structured rewrite type-mismatch diagnostics for the
preserved false-fusion control, retaining rejection semantics and fixed budgets.
The next independent interpreter family is the ten `nested_field_match` originals
(including `_004_go` and `_007_go`), followed by their nested-multi-scrutinee
neighbors. Neither work item requires GPU or desktop access. No newer upstream
revision has been adopted.

## Reproduction, resources and receipts

After activating the frozen environment, the recorded commands include:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test cpu_no_main --test kernel_normalization --test cli_compile_executable --test compiler_executable_c --quiet
cargo build --locked --all-features --bin teamy-bend --example compare_normalization
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <clean-pin> -OutputDirectory <fresh-directory> -Suite all -Repetitions 2 -RequireExactText
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <fresh-directory> -Suite baseline -Repetitions 2
python3 <private-evidence>/qualify.py --output <fresh-name> --snapshot final-snapshot
python3 <private-evidence>/audit.py
python3 <private-evidence>/negative-controls.py
python3 <private-evidence>/verify.py
```

The private cpu-no-main-interpreter evidence directory holds exact argv, source/
binary/adapter hashes, stream bytes, statuses, timings and cache receipts.
Scripts with fixed output paths require fresh destinations before rerunning.
Public ledgers contain sanitized hashes/identifiers only; raw local diagnostics
and private paths are not repository artifacts.

Rust 1.96.0, nightly-2026-05-24 formatting, Node 24.19.0, PS 7.5.4 and Clang
19.1.7 remain frozen. Four host build jobs, debug=0 and disabled incremental
cache are recorded host build controls; they are not Bend optimization or budget
changes. Baseline upstream C uses the pinned O3 / -lpthread -lm profile with
`--threads 1 --gpu off`; port C keeps strict C11 warnings and -pthread -lm -ldl.
No C runtime is built for a refused no-main/function artifact.

All Cargo build preflights met the 10-GiB reserve. Full-gate minimum headroom was
**9.53 GiB**, above the unchanged 2-GiB workspace / 1-GiB tmp
stop floors. Final cleanup reclaimed only inactive rebuildable test executables,
restoring **10.15 GiB**. Protected binaries, source, original evidence and
historical receipts remain intact. No install, push, network/security change,
desktop or GPU work occurred.

| Evidence | SHA256 |
| --- | --- |
| no-main-final/receipt.json | 4485599629f1c9d1ad9b374b42c5dae1d90312e9c7a66231d48c5f92cdac4c93 |
| normalization/receipt.json | 297ba5579847e81bcc21d60e5cac2ed8e79d6bd25e0b2e4b049ed86f5f5aead1 |
| adjacent-qualified/receipt.json | ce699b9f984f3f970ff82ba96b2b91a99eddbc644d33b39a3d687173e7babf89 |
| audit/receipt.json | 357c5be86c50b218bd674c002acd1c26c3caae15aaea9848db29f1d45fccd648 |
| baseline/receipt.json | 5a9869574f8cede70d3fd781c51fe699e0634cbc43e1672a70aafd634c1aa4b3 |
| full-gate.log | 8b7ab0d34dc16a53f5ff6c4bb88b891efba24c0852a94599c707e505c681a84a |
| full-gate.json | fcf569ef0ad4e1da322e95b651d2afd4a633bb072ed804cfe064942285e88c9b |
| negative-controls/receipt.json | 7a55c00f6536fcf59061e201bf536d9f5de7be7ce67152ee87da53923ee9ddee |
| negative-controls/phase-receipt.json | 089648ba2cda22a1dd27e347ff8b88dd35d3fea7777bb6c5b7434111919a8fa2 |

Cargo.lock remains SHA256
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
Upstream remains clean at the target pin and teamy-docs remains clean at
`50e89d6897ce2a65167ef65b17c5fa83fe849158`. The later documentation checkpoint
records this clean-source gate without claiming a second execution.
