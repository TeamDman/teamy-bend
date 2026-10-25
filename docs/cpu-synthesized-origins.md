# Synthesized-reference source provenance

Tested source: `8a14fa7d51554a4479fb9ea7fe8fb1af3060be83`. The later evidence checkpoint is
documentation-only. Upstream remains
[`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
Dependencies, toolchains, evaluation policy and safety limits are unchanged.
This closes a bounded diagnostic gap, not CPU parity.

## Correction and exact applicability

The parser previously synthesized logical and monadic references without the
reference node's source origin. Inference already attached a structured error,
but the CLI could not locate the generated reference and fell back to host
context. `reference_at` now registers the existing weak side-table origin when
constructing that same `Ref` node. Core node shapes, fixed sugar names, monadic
resolution, argument order, binder IDs and runtime representation are unchanged.

| Unchanged original | Reference and required source anchor |
| --- | --- |
| `check/error_window_assert_exists` | `Exists`, the `exs` clause |
| `check/error_window_inline_exists` | `Exists`, the inline `&` clause |
| `import/empty_neq` | `Empty`, the `!=` token |
| `check/do_missing_bind` | `Box.bind`, the bind statement |

All four **load successfully, then reject during checking**. Historical fixture
comments do not override these measured phases. Their CLI status remains 1,
stdout remains empty, and stderr now matches upstream exactly, including context
and source windows. C/JS/native compilation rejects before producing an artifact;
no execution is claimed for these negative programs. Originals remain byte-identical
with attribution in [the fixture directory](../tests/fixtures/cpu-synthesized-origins/README.md).

Pinned source grounds the anchors:
[inequality](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1875),
[inline existential](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2170),
[do statements](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2254),
and [law clauses](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2599).
Eight adjacent controls cover multiline `exs`, `where`, inline `&`, inequality,
missing `pure`, statement-form `bind`, and imported logical/monadic modules.
The first undefined name and the introducing line remain distinct from later
arguments, continuation bodies and the outer `do` header.

Two positive controls define the required names. Their outputs are `Z{}` and
`Full{}`; the complete constructor graphs agree with the pinned interpreter.
Native strict evaluation and strict-pure C/JS retain these values. Both executable
emitters still require Base; strict-pure compilation is a separate API. The local
logical control deliberately uses a live first parameter in its own `Exists`
definition and is not a replacement implementation of Base's existential type.

The census also found an independently correct location change in
[`check/monad_do_destructure`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/check/monad_do_destructure.bend):
the existing application-origin path now points to bind line 11 instead of outer
`do` line 10. The same pattern diagnostic and status now match upstream exactly.
This case **fails during loading; checking and execution are unrun**. It is
supplementary single-census/phase evidence, not a fifth completed fixture family.
Independent review confirmed the adjacent effect is consistent with pinned syntax.

## Review, guards and public boundary

The tracked/plain-book regression compares complete parsed structure, rejection,
fuel, depth and fresh-name state. Positive results are compared after temporary
tracked engines and diagnostic origins are dropped, with the checked runtime
engine explicitly verified to contain no origins. Full-gate existing identity,
strong-ownership, sharing/composition, demand, cancellation and budget controls
remain active. Debug structure equality alone is not claimed as proof of pointer
identity. There is no new mid-normalization cancellation claim.

Independent review found a test-lifetime issue in the first checkpoint: a
tracked engine retained an origin clone after `drop(origins)`. The test now drops
that engine first and accurately names its structural/budget guarantee. The
initial gate was intentionally stopped at 74 seconds (exit 143), its receipt
preserved, and the final source received a new full gate. No partial run is
counted as a pass. Initial test-development compile/probe failures are retained
separately; none establishes a runtime discrepancy.

No public signature or term field changed. These newly located failures use the
existing `Ok(CliOutput)` error-rendering path with `exit_code() == 1`.
Repository dispatch reads that code before output/cleanup completes and the main
entrypoint returns it. External consumers remain unaudited and must not use
`Result::is_ok()` as command success. Earlier emitter and
`DefDecl::completion_parameters` compatibility caveats remain.

## Validation evidence

| Check | Outcome |
| --- | --- |
| Final standard gate, clean tested source | **1122 passed / 0 failed / 49 ignored** |
| Format, production strict Clippy, all-feature build, Node provider | Pass |
| New integration-target strict Clippy | Pass on tested source |
| New regression coverage | Three integration tests and one structural/budget unit, included in the full gate |
| Primary differential matrix | 270 observations; 216 exact negative boundaries, two complete positive constructor results and explicit Base build refusals |
| Prior runtime/diagnostic renewal | 246 observations; all previous outcomes retained |
| Combined matrices | **516 observations**, zero timeouts, repeated process groups deterministic |
| Actual phases | 12 upstream negative loads succeed and checking rejects; port loader success is asserted by unit tests |
| Current status census | **1,302 unchanged outcomes**; 439 checker negatives reject, 366 other entries accept and 497 reject; no timeouts |
| Current changed streams | Five; all exactly match upstream (four check diagnostics, one load diagnostic) |
| Prior 489-case renewal | 30 exact diagnostics, 385 strict IO exclusions, **74 unresolved differences**; all upstream streams unchanged |
| Ten separately tracked old cells | 40 renewed observations, unchanged exactly, including `unsafe_many` |
| Positive / unfinished-hole controls | 14 positive observations pass; 18 preserve three unwaived hole/TODO disagreements |

The prior 489-case subset improves from 26 to 30 exact diagnostics. Its remaining
74 differences comprise 64 upstream-accept/port-reject outcomes and ten differing
rejections. Two of those ten concern constructor-definition labels or comptime
substitution locations; eight concern strict IO/foreign exclusions preempting
another upstream error. These are unwaived. The extra destructuring-load fix is
outside that preceding 489-case set. [Checking ledger](cpu-synthesized-origins-checking.csv).

The renewed runtime controls still contain two repeated upstream C-generation
failures for forged IO and two direct-IO Node host failures. Generated-C execution
remains unrun where generation failed. Strict IO proof rejection is distinct
from executable success, and generated JS success does not close the Node host
lane. Old imported-JS printer timeouts, dependent-pair/minimal-IO C failures,
F32.read, argument-error order, host/foreign/resource/API gaps and the incomplete
688-vector exploration remain unwaived. No new exception is introduced.

Historical supplementary all-target Clippy failures remain unwaived and were
not rerun. Standard/focused lint success is not an all-target pass. The prior
seven observed test lints and earlier fail-fast findings are preserved.

## Reproduction and storage

```sh
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo test --locked --features extended_observability --test cpu_synthesized_origins
cargo test --locked --features extended_observability --lib synthesized_reference_origins_preserve_structure_and_checker_budgets
cargo clippy --locked --features extended_observability --test cpu_synthesized_origins -- -D warnings
node "$UPSTREAM/bend2/main.ts" "$FIXTURE"
node "$UPSTREAM/bend2/main.ts" "$FIXTURE" -o "$ARTIFACT"
"$BEND" check "$FIXTURE"
"$BEND" compile "$FIXTURE" --executable --target c --output "$ARTIFACT"
```

Paths are caller-selected. The fixture manifest and expected graph/text/error
files retain the bounded cases. Matrix children have 30-second limits; the
standard gate has a 1,200-second outer limit. Successful upstream C runtime
controls use the pinned CLI's `-std=c11 -O3` with `-lpthread -lm`, then
`--threads 1 --gpu off`. Port C adds strict `-Wall -Wextra -Werror -pedantic`
and `-pthread -lm -ldl`. Matrix pure-C controls also use O3; repository pure-C
tests retain their established strict default optimization profile.

Frozen host: Rust/Cargo 1.96.0, nightly-2026-05-24 formatter, Node 24.19.0,
Clang 19.1.7 and PowerShell 7.5.4. Build overrides are
`CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0`
and `CARGO_BUILD_JOBS=4`; these are Rust host profiles, not Bend optimization.
`BEND_GPU=off CARGO_NET_OFFLINE=true`; work/depth limits remain 2,000,000 / 128.
Cargo.lock SHA-256 remains
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.

Disk preflight requires 11 GiB free and stops below 10 GiB, with a separate
1 GiB temporary floor. The final gate started at 11.51 GiB
and reached 10.47 GiB minimum. Rebuilt test binaries
account for the transient growth. Only verified inactive rebuildable caches
were retired after the interrupted and completed gates; source, unique historical
failures, snapshots and receipts remain intact. Final reclaimed headroom is
11.52 GiB. Compact fixtures and ledgers are in Git;
raw command/stream receipts, generated artifacts and CLI captures are explicitly
**outside Git preservation**.

Captured tested binary SHA-256: `e87d8ac048f1c990de0c61cd80f4f4390b031495c71580e8c4b399981262e08b`.

| Retained private receipt | SHA-256 |
| --- | --- |
| `snapshot.json` | `1eda5c7f48c94e22f98a802a32fdec2782fa6b121072e7441fee6efc00077bcb` |
| `full-gate.json` | `298c7f839279dd8c217e97cce626ed56c444e5ef02f56f1afc193888e1bf91e7` |
| `full-gate-initial.json` | `407dfbe4236097fc0829e9b4e4a0a950afc8e632e05f81bb9afb15c2fae24f41` |
| `gate-review-stop.json` | `7ab92db4a721ef005b67be54a7a1e79fd85e57b7d86b65a8cd89a480ee27a510` |
| `focused-clippy.json` | `b72cce7cd2abc115a171cee61135fe0496c7d3f02f5d61511eab43d1e8cd74af` |
| `matrix/receipt.json` | `1dc46a024f826e6fb1f124081d85ca3ce67678df3e602a1a8cbb867d4bf23465` |
| `baseline-matrix/receipt.json` | `bdc172cc41e0b8b3f8934a9bf4cbe6a6de4c32d4644040ad201545f8bc5c5a51` |
| `phase-controls.json` | `70aded6700e6fdeaf0ae67185a806f692c0f8442bb2107fdde930b860335b2d9` |
| `adjacent-load-phase.json` | `2d4fbfd6b4e32edb8ad9985e41b3804a2100d8b69ab1c31e2446672e7dd6eef8` |
| `audit/receipt.json` | `f652e4a3930d48f0cba1d5dcfe3195b56295a51ff762e757123504988670917b` |
| `audit-delta.json` | `86ad83002b38b3bfb4fd629aee02376a697f7cf805bf76821dc8e258d02da396` |
| `audit-upstream-delta.json` | `eac4fde4421524be3f2e1d46c9e40805a417c109fffdb8fc5b9905d0d8e4c3a1` |
| `renewed-checking.json` | `22284850c207910dbc01a74561d8269e9d7ed01d74e486691ef0a948d1df031d` |
| `checking-renewal-summary.json` | `9eca6bcb7a1f63e2bb7bf27fd93c8811d12027b3b87763e390fa356134f62540` |
| `retained-controls.json` | `ea28a8aa361c35748800eb7ffc999fcd3b60ea3e8f2f013b0b2e9f3dd0ea6c7f` |
| `controls.json` | `05ec55fcb30133d312907478aabde93e16caee52b0d35b42ea116020e632f1eb` |
| `independent-review.json` | `ad6b2a64e2dbcd0de91573f4fa47e87fdac709da73b53c4bc17c8ab3f9b48322` |
| `frozen-inputs.json` | `06e4f6276b83134056fb3f6d0694f94a627a649156ff7e67ae3b3eb2a72b8f92` |

## Remaining acceptance work

Detailed links now cover **399 originals**; **903 remain unlinked**:
352 checker negatives, one checking-only module, 13 interpreter-profile cases
and 537 compiled candidates. No runtime links were manufactured for negative
fixtures. Links include historical, partial and blocked receipts; source profiles
are applicability hints. The [acceptance criterion](cpu-parity-acceptance.md)
still requires resolving applicable contracts, not merely passing this gate.

Next independent diagnostic work: constructor-definition labels (including
`check/field_type_bound`) and the `comptime/later_def` substitution source site,
while preserving all older unwaived cases. The next substantive candidate batch
remains `parse/datatype_declaration_form`, `empty_datatype` and
`empty_param_group`, with actual printer/runtime applicability first. No credit
is assigned to proposed work.
