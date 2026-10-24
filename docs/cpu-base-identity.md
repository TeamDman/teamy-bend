# Base identity and undefined-reference diagnostics

Tested source: `81ec997d1e3c93c24a525548ff21396297390f27`. Upstream remains
[`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
The later evidence checkpoint changes only documentation. This batch does not
establish full CPU parity.

## Behavior and applicability

Two unchanged originals distinguish a user family named `IO` from Base effects
and exercise custom constructors without Base. Copies and hashes are in
[the fixture README](../tests/fixtures/cpu-base-identity/README.md).

| Program | Result | Applicable comparison |
| --- | --- | --- |
| `io/main_forged_io` | `Full{}` | Pinned interpreter/JS and native/port C/JS agree; upstream C generation fails |
| `import/base_prelude` | `Sx{Zx{}}` | Interpreter/native and strict-pure C/JS complete constructor results agree; both executable emitters require Base |
| Base plus imported user IO adapter | `../main_forged_io.Full{}` | Pinned interpreter, native and both implementations' generated C/JS agree |
| Retained `reg/framed_cell_dep` true IO | `3` | Native and generated C/JS agree; upstream direct Node host fails |

Successful processes return 0 with empty stderr. Four depth inputs (0, 1, 3, 8)
compare direct calls and typed observations, including every nested constructor
child. Four original definition graphs/text and the Base-import adapter graph
are asserted separately from runtime behavior. Strict-pure C/JS compare six
complete constructor values through their typed JSON output. This is a distinct
port API, not upstream executable-C qualification. Cancellation-before-entry
remains covered; no mid-normalization cancellation is claimed.

The forged-IO original's historical comment predicts refusal, but actual pinned
JS generates and executes. Upstream C instead exits 1 with
`TypeError: Cannot read properties of undefined (reading 'c')` before producing
an artifact. C build and execution are therefore **unrun**. A retained stack
points to [comp.ts:2828](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L2828),
where compilation dereferences a declaration from its runtime-ADT source set.
This is an unresolved upstream generation failure, not a new semantic exception
or a reason to make the port crash. The Base-import adapter succeeds in C.

[IO identity](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1177)
requires the actual Base declaration provenance. A user alias's spelling alone
does not grant effects. The retained true-IO direct interpreter still fails with
`TypeError: require is not a function` under Node. Generated JS success does not
close this host lane; no Bun installation or source workaround was used.

## Minimal production correction

Calling nonexistent `IO.print` already rejected, but the CLI exposed host context
instead of a Bend source diagnostic. Undefined-reference inference now attaches
the existing structured diagnostic at the same failed lookup, matching
[pinned inference](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3271):
expected `a defined name`, observed the reference, its local context and source
window. Internal `KernelError` text and public signatures are unchanged.

Regressions cover direct, imported and bound-context references, type and erased
positions, competing missing names, and wrong-domain arguments. Exact status,
stdout, stderr and absent-output-artifact checks cover check/eval/run and
C/JS/native compilation. A unit control preserves first-error order, term
identity/strong ownership, fresh-name state and unchanged work/depth accounting;
if diagnostic presentation exhausts the remaining budget, the original error
survives. Existing strict IO exclusions still reject. No dependency, evaluator,
demand, sharing, guard or cancellation policy changed. Independent review found
no blocking concern.

API caveat: more failures now take the **existing** `Ok(CliOutput)` diagnostic
path with `exit_code() == 1`. `src/lib.rs` reads that status and `src/main.rs`
returns it after cleanup. External invoke consumers were not audited and must
not equate `Result::is_ok()` with command success. Prior executable-emitter Err
and public `DefDecl::completion_parameters` compatibility caveats remain.

## Validation

| Check | Outcome |
| --- | --- |
| Standard gate, clean tested source | **1118 passed / 0 failed / 49 ignored** |
| Format, production strict Clippy, all-feature build, Node provider | Pass |
| New integration-target strict Clippy on tested source | Pass |
| New focused controls | Eight integration tests plus one budget/order unit; all included in the full gate |
| Full gate aggregate controls | Sharing/composition, named-call reconstruction, demand, error, cancellation and guard regressions pass |
| Differential matrix | **246 observations**, zero timeouts; repeated process groups deterministic |
| Runtime/interpreter | 36 exact status/stdout/stderr observations |
| Graph/text/check/eval | 16 full upstream graph captures, 36 exact text/check/eval observations; complete port graphs asserted by tests |
| Successful generation/build | 11 generation and five C build controls |
| Negative contracts | 126 exact checker refusals; eight exact Base-required refusals; four strict IO proof exclusions |
| Failed upstream lanes | Two repeated forged-IO C-generation failures and two repeated Node host failures, retained separately |
| Current checking-status census | 1,302 unchanged statuses; 439 checker negatives reject; remaining entries: 366 accept, 497 reject; no timeouts |
| Changed-stream checking comparison | 489 observations: 26 exact diagnostics, 385 strict IO exclusions, 78 unresolved contract differences |
| Ten previously tracked cells | 40 renewed observations; all ten unchanged, including `unsafe_many` |
| Positive and unfinished-hole controls | 14 positive observations pass; 18 preserve three unwaived hole/TODO differences |

The 78 remaining changed-stream differences comprise 64 upstream-accept/port-
reject cells and 14 differing rejection diagnostics. They were already divergent
before this presentation change; no acceptance changed. Among the latter are
four generated-reference source gaps (`Box.bind`, `Exists`, `Empty`), one
constructor-definition label, one comptime substitution source location, and
eight cases where strict IO/foreign exclusion preempts a different upstream
error. The 14 host-format fallback streams changed only their Rust context line
while retaining the same internal failure; they are not exact Bend diagnostics.
These observations are additional classifications, not waived exceptions or
new full-fixture coverage. [Checking phase ledger](cpu-base-identity-checking.csv).
The adapter uses upstream `book_load`, `book_valid` and TODO checking from
[main.ts:424](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/main.ts#L424),
without running `main`; it makes no runtime claim.

The historical supplementary **all-target** Clippy failure at `96f4d4759a91c8ba81c04550a238aa476269fcd4`
remains unwaived and was not rerun: seven observed existing test lints, with an
earlier fail-fast run showing four others. Standard and focused lint success do
not imply that supplementary command passes. No lint was suppressed.

## Reproduction and preserved evidence

Frozen host: Rust/Cargo 1.96.0, nightly-2026-05-24 formatter, Node 24.19.0,
Clang 19.1.7, PowerShell 7.5.4. Host build overrides are
`CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0`
and `CARGO_BUILD_JOBS=4`; they do not change Bend or upstream C optimization.
`BEND_GPU=off CARGO_NET_OFFLINE=true` throughout. Work/depth remain
2,000,000 / 128. Cargo.lock SHA-256 remains
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.

```sh
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo test --locked --features extended_observability --test kernel_base_identity --test compiler_base_identity
cargo test --locked --features extended_observability --lib undefined_name_diagnostics_preserve_first_error_sharing_and_remaining_budget
cargo clippy --locked --features extended_observability --test kernel_base_identity --test compiler_base_identity -- -D warnings
node "$UPSTREAM/bend2/main.ts" "$FIXTURE" -o "$ARTIFACT"
"$CC" -std=c11 -O3 "$ARTIFACT" -lpthread -lm -o "$PROGRAM"
"$PROGRAM" --threads 1 --gpu off
```

`UPSTREAM`, `FIXTURE`, `ARTIFACT`, `PROGRAM` and `CC` denote caller-selected paths.
The original paths/hashes, bounded adapter construction and expected graph/text
are in the checked-in tests. Port executable C additionally uses
`-Wall -Wextra -Werror -pedantic`, `-pthread -lm -ldl`; strict-pure compiler tests
retain their existing strict C11 flags without inventing an O3 claim. Matrix
children are bounded at 30 seconds. The gate runner has a 1,200-second outer
limit. Exact private commands, streams, statuses and artifact hashes are retained.

Disk preflight requires 11 GiB free, stops below 10 GiB (1 GiB temporary floor),
and records samples. This gate started with 11.48 GiB,
reached 10.49 GiB minimum, and was not stopped. Growth
comes from rebuilt test executables and alternate feature binaries. Storage
inspection reports roughly 2 GiB in build outputs and 18 GiB in the private
tooling/evidence tree, including pinned tools and unique historical receipts.
That historical tree is preserved; this batch does not treat it as disposable.
After completion, only inactive rebuildable caches are retired. Unique failure
captures, snapshots and source remain intact. Git preserves compact reproducible
fixtures and ledgers; ignored raw receipts, generated artifacts and CLI snapshots
remain **outside Git preservation**.

Captured binary SHA-256: `5f6be774b272a108ff60ccc39d96b51837eb6ff34aad67ac02bac54c0ed8af26`.
Raw receipt hashes below identify retained private artifacts without embedding
machine paths or raw captures in public documentation. The baseline undefined-name
assertion failure, an initial malformed erased-position probe (parse refusal),
and a premature audit launch before its script existed are retained privately.
Corrected probes and the actual completed audit supply the reported evidence;
none of those preparatory failures is counted as qualification.

| Receipt | SHA-256 |
| --- | --- |
| `snapshot.json` | `29900a643e312e86fdaa95bad536f1fa973052079f51c23b5a58cc2543ac614c` |
| `full-gate.json` | `428dd7813003e0a1ce54240d6a0ac483a945fd8ab5635c2b4f58ac0953c78dfd` |
| `focused-clippy.json` | `9102d5934fd7a819650ba39cfa4fdb440132c2ca152464cd003c471abde13656` |
| `matrix/receipt.json` | `b26b206eb213340427a22dce91fe2ce948bcf274d3bfcf532bd0d6228cc25031` |
| `audit/receipt.json` | `46dd01f54fb9b647f9ddfaa3ff578a8419a7c0abb0bd41beac7a95ed5462795d` |
| `audit-delta.json` | `8b4faec17b7b79fb10e3bfa30523a4d74d601d2282a7d70122971a6fa56f1405` |
| `audit-upstream-delta.json` | `af23691f1cf7f642c72ebaf93a7650a4d1de4357cf449e78d3f6f2808fe90c34` |
| `retained-controls.json` | `5dfa08e7ae810a2becaad5335c2b4a7d9bd783855b9ec8d062dfb8c05c16a887` |
| `controls.json` | `48fc7a20d56cc6e2ec545c5791a2e98367883c15b708f8f941d2c429989d00f6` |
| `independent-review.json` | `36515d3bac71c6e250c3f8619421fb460e2f451a08ee0bccb79b211bbf14268a` |
| `frozen-inputs.json` | `cef1f1bccc9444aa132a334ca782aa3167e445648ad0f0c7f802d8c83eeee7f4` |

## Remaining acceptance work

Detailed links now cover **395 originals**; **907 remain unlinked**:
356 checker negatives, one checking-only module, 13 interpreter-profile cases
and 537 compiled candidates. Links include historical, partial and failed lanes;
source profiles are applicability hints. The 489 checking probes do not create
489 runtime passes or detailed fixture links. Full parity still requires the
applicable checking/normalization/runtime/diagnostic contracts in the
[acceptance criterion](cpu-parity-acceptance.md), with failures and unavailable
oracles explicitly resolved or reviewed.

The prior ten cells, the additional changed-stream differences, imported-JS
printer timeouts, dependent-pair/minimal-IO C failures, F32.read, argument-error
ordering, host/foreign/resource/API gaps and incomplete 688-vector exploration
remain unwaived. Next bounded independent repair: generated `Exists`, `Empty`
and `Box.bind` source provenance, using the four original negative fixtures and
unchanged rejection status. A subsequent substantive batch can cover
`parse/datatype_declaration_form`, `empty_datatype` and `empty_param_group` with
actual applicability first. Neither proposed batch is counted as qualified.
