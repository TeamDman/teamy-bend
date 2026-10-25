# Constructor definitions and compile-time provenance

Tested source: `1e8c0c3fba2315ecfeecfd86574044b3feb39608`. The following evidence checkpoint is documentation-only.
Upstream remains `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`; all dependencies,
toolchains, evaluation policies and resource limits remain frozen. This closes
constructor-label gaps and an adjacent kind diagnostic; **CPU parity is incomplete**.

## Corrected diagnostics

Pinned [book_valid](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3666)
checks a datatype signature under its datatype name, then each constructor
parameter/field domain under the constructor name. The port previously filled
all such labels with the datatype. The correction annotates only an already
rejected structured diagnostic. It leaves `Lhs` unchanged because that structure
also controls recursion and demand; public `KernelError.message` retains its
historical datatype prefix.

Seven unchanged original negatives now match exactly at check/eval/run and all
three executable compile targets: `check/cop_field_formation`, `data_self_ref`,
`field_type_bound`, `nested_field_bound`, and `grade/reject_fn_field`,
`reject_negative`, `reject_plus_field`. They load successfully and reject during
checking. No generated program runs after rejection. Eleven adjacent controls
cover datatype signature labels, first-field order, a failing second constructor,
dependent context, imported names, reduced aliases and qualified template names.
Together these form **18 exact negative cases**, repeated at process boundaries.

A non-kind datatype result previously fell back to host diagnostics. The new
structured report uses the actual datatype name, parameter context and reduced
kind. When a zero-parameter datatype reference becomes its existing `Adt` result,
only weak external provenance is copied; allocation, reduction steps and strong
ownership are unchanged. Atomic `Adt` syntax also receives its ordinary parser
origin. Local and imported alias controls establish that the source window points
to the reduced alias body, as required by upstream, rather than its caller.

The complete census found **15 changed stderr streams**, all exact pinned-checker
matches: 13 constructor labels and two non-kind reports. Seven originals have the
repeated full boundary evidence above; the other eight are supplementary
checker-only observations, without new runtime or completed-family credit.

A positive custom datatype/template program retains `W{Z{}}` in complete graph
and text comparisons, native strict evaluation and pure C/JS execution.
Executable emission still requires Base; pure compilation is a separate API.

## Compile-time substitution remains open

The exact original `comptime/later_def.bend` and four adjacent reproducers are
preserved in `pending-cases.json`. All reject at the same checking phase and
status, but five source windows still differ. These are true diagnostic gaps,
not IO proof exclusions or accepted policy exceptions. For example:

```bend
import Base
def app(~f: Nat -> Nat, x: Nat) -> Nat:
  f(x)
def main() -> Nat:
  app(~later, 3n)
def later(n: Nat) -> Nat: n
```

Both implementations report undefined `later` in `app~0`. Upstream points to
`f(x)`; the port points to `app(~later, 3n)`. The other reproducers cover a bare
returned function, multiline parenthesized head, repeated uses and an imported
original. A dotted `Missing.func` argument correctly retains the caller's source
in both implementations and remains an exact control.

Pinned [parse_var](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1635)
turns a substituted bare `Var` into a `Ref` with the use span; an already qualified
`Ref` retains its existing span. The port represents the bare argument as a
shared `Ref` before substitution, and node-origin metadata has one location per
allocation. Independent review confirmed that making fresh reference leaves is
unsafe as a presentation-only fix: `is_compile_binding` recognizes arguments
using `Rc::ptr_eq`, and those markers protect macro application, rebinding and
pattern behavior. A call-head-only override would leave other uses wrong.

No such partial correction was applied. The next implementation step is private
per-occurrence provenance carried separately into parent edges and surface/root
occurrences while preserving shared core terms. This is a bounded architectural
implementation gap, not an established need for a new language-policy decision,
new dependency, higher limit or desktop execution.

## Validation and review

| Check | Outcome on the tested source |
| --- | --- |
| Standard gate | **1,126 passed / 0 failed / 49 ignored** |
| Format, production strict Clippy, all-feature build, Node provider | Pass |
| Focused integration-target strict Clippy | Pass |
| New regression tests | Three integration tests and one structural/budget unit, included in the full gate |
| New differential matrix | 441 process observations: exact controls and explicitly unresolved template windows |
| Prior runtime/diagnostic matrix | 246 observations, previous outcomes retained |
| Combined matrices | **687 observations**, zero timeouts; repeated groups deterministic |
| Phase probes | 23 loads succeed, then 23 checking refusals; no runtime claim |
| Inventory census | **1,302 acceptance/rejection outcomes unchanged**, zero timeouts |
| Current changed streams | 15 exact pinned-checker matches |
| Renewed preceding 489-case subset | **31 exact diagnostics / 385 strict IO exclusions / 73 unresolved** |
| Prior ten diagnostic/acceptance cells | Six constructor labels now exact; three pattern anchors and `unsafe_many` remain unresolved |
| Positive and hole/TODO controls | 14 positive observations; 18 retain three unwaived disagreements |

The 73 renewed differences comprise 64 upstream-accept/port-reject outcomes and
nine differing rejections: `later_def` and eight IO/foreign preemptions. All 489
upstream status/stdout/stderr observations are renewed unchanged. The four old
remaining cells are tracked separately; do not add overlapping ledgers as if
they were disjoint or infer universal parity from a linked receipt.

The runtime renewal still includes two repeated upstream C-generation failures
for forged IO and two Node direct-IO host failures. Failed generation leaves C
build/execution unrun. Strict IO proof rejection is distinct from executable
success. Historical imported-JS printer timeouts, dependent-pair/minimal-IO C
failures, F32.read, competing-error order, host/foreign/resource/API gaps and the
incomplete 688-vector exploration remain unwaived. Historical supplementary
all-target Clippy failures were not rerun and remain open; passing standard and
focused Clippy is not an all-target pass.

Independent review closed an alias-origin finding and strengthened exhaustion
coverage. The unit compares tracked/plain parsed structure, internal errors,
fuel/depth/freshness for all 23 negatives. It measures the work required to reach
the non-kind rejection and repeats with exactly that fuel, preserving the
original error while diagnostic rendering has no allowance. Existing aggregate
sharing, demand, guards and cancellation regressions remain in the full gate.
No new mid-normalization cancellation is claimed. Early development failures
(a test's incorrect budget-error wording and an incomplete reduced-origin trial)
are preserved separately, not counted as passing validation.

No public signature or term field changes. Repository callers still read
`CliOutput::exit_code()` and propagate it after emission/cleanup. Newly structured
kind failures can return `Ok(CliOutput)` with status 1; external consumers remain
unaudited and must not treat `Result::is_ok()` as command success. Earlier emitter
and `DefDecl::completion_parameters` compatibility caveats remain.

## Reproduction and storage

```sh
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo test --locked --features extended_observability --test cpu_definition_provenance
cargo test --locked --features extended_observability --lib definition_origins_preserve_structure_and_checker_budgets
cargo clippy --locked --features extended_observability --test cpu_definition_provenance -- -D warnings
node "$UPSTREAM/bend2/main.ts" "$FIXTURE"
node "$UPSTREAM/bend2/main.ts" "$FIXTURE" -o "$ARTIFACT"
"$BEND" check "$FIXTURE"
"$BEND" compile "$FIXTURE" --executable --target c --output "$ARTIFACT"
```

Paths are caller-selected. The committed manifests, unchanged originals,
expected diagnostics and complete positive graph/text make the bounded cases
reproducible. Each matrix child has a 30-second bound; the full gate has a
1,200-second outer bound. Successful upstream C controls use the pinned CLI's
`-std=c11 -O3`, linked with `-lpthread -lm`, and run `--threads 1 --gpu off`.
Port C adds `-Wall -Wextra -Werror -pedantic -pthread -lm -ldl`.
Pure matrix C uses O3; repository pure-C tests retain their existing default
strict compilation profile.

Frozen host: Rust/Cargo 1.96.0, formatter nightly-2026-05-24, Node 24.19.0,
Clang 19.1.7 and PowerShell 7.5.4. Host build overrides are
`CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0`
and `CARGO_BUILD_JOBS=4`, with `BEND_GPU=off CARGO_NET_OFFLINE=true`.
These Rust host settings are distinct from Bend runtime optimization.
Work/depth guards remain 2,000,000 / 128. Cargo.lock SHA-256 remains
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.

Disk preflight requires 11 GiB free and stops below 10 GiB, with a separate
1 GiB temporary floor. The gate started at 11.49 GiB and reached
10.45 GiB minimum. Rebuilt test binaries explain the transient
growth. Only inactive rebuildable caches were reclaimed after completion;
source, unique historical failures, snapshots and receipts remain intact.
Final headroom is 11.50 GiB. Compact fixtures and ledgers are in Git;
raw streams, generated artifacts and binary captures remain **outside Git
preservation**. Captured tested binary SHA-256:
`b575a8e9065868eb61915f3285d6e3264db2fd6dc2e1dfee30b40e2f697e514c`.

| Retained private receipt | SHA-256 |
| --- | --- |
| `full-gate.json` | `2642cb514c504a1c18512c547bba6cef6ed3cd6553ea2eb67853d176d91a7ce7` |
| `focused-clippy-current.json` | `806d9fe2c8670e4aca354d62367380d937201616c14ba222b42362a3dec471c6` |
| `snapshot.json` | `be1ffcb8e4a97ca6ba8420dc406bbfcbfac6c1ba93d73f7d464c56ed75b563e2` |
| `matrix/receipt.json` | `fb4efda1ca608c38ec60a813a37a3728a9f630361a841a65aaee0180f858e425` |
| `baseline-matrix/receipt.json` | `f21969c9caaf918bbc0bd30ec48ad73b1ecc02368c3cbd06c660157d1c00eed8` |
| `phase-controls.json` | `7a93d1c17dec0aa07b9c44cdae01f6e8358433074da8bc438d27b8938d282f42` |
| `audit/receipt.json` | `6df592685201e6fab70410f113fd9f560308db3c30436dc8a7d01c70b35e4b1e` |
| `audit-delta.json` | `3fb0adfdb3f79ab328eb60f664335c83fc2923da1b0732feb48d138f0e9b7f06` |
| `audit-upstream-delta.json` | `8ab87e042dda2db9c5b8a3d359198f775950544447cd692e56575706f4315b91` |
| `renewed-checking.json` | `1503a624f4631af6ac9b802d8bc4f202be1930c8a12e369fe11c4e1bbc7ad064` |
| `checking-renewal-summary.json` | `cd121d780268b11fd1b594b92ed3447928fc979c0f6161c9016cc095f0aca3cf` |
| `retained-controls.json` | `f95830c5866a7db63a60e2f9ef5bbe6494cccdabb28000e647c45723dddd4119` |
| `controls.json` | `3015bc1e2730962476b041443a8de967dc6268fe5b0603dc3b3d1e5e17833491` |
| `independent-review.json` | `c1fc2a8f742d70c54d668a69aceef6b2da2e08659b0c7dba1018acb80c8ff955` |
| `api-review.json` | `4a31fcb4986f855b7db987f8c9b517c28afac9426d1205fb454470c6614835a8` |
| `frozen-inputs.json` | `0f8b4a89f5c8ead6947601e0ceb78669e3ddc467c155704dcd9967289086ae22` |

## Remaining acceptance work

Detailed links cover **406 originals**; **896 remain unlinked**: 345 checker
negatives, one checking-only module, 13 interpreter-profile cases and 537 compiled
candidates. Runtime/interpretation links remain unchanged. Links include partial,
historical and blocked outcomes; they are not a completed-parity count. The
[acceptance criterion](cpu-parity-acceptance.md) remains unchanged.

Next: occurrence-aware template provenance for the five preserved reproducers,
then the three remaining pattern-anchor cells. The substantive applicability-first
candidate batch remains `parse/datatype_declaration_form`, `empty_datatype` and
`empty_param_group`. No proposed work receives qualification credit.
