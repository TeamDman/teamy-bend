# CPU U32 operation qualification

Target: unchanged [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
Candidate: `cc0f3e8e6a7de2fa0576c9869b8584662c9e86f6`, based on timer checkpoint
`7e1bc149b1617b8b26a000f25fb1317f563b9389`. Full CPU parity remains open.

## Audit and demonstrated defect

The audit verified all 1,302 upstream fixture hashes and expectation hashes,
exact inventory membership, the inventory/gate hashes, and all 41 existing
ledger source hashes. Before this expansion, 1,261 rows lacked a linked runtime
receipt; 646 JS and 631 C candidate lanes among those rows were unlinked. These
syntactic candidate counts are not supported-feature counts or parity percentages.

Seven unlinked fixtures were selected for wrapping arithmetic, unsigned
comparison, bit operations and total division. They need only Console output,
not foreign imports, Bun or the pending Nat-demand contract decisions.
The preserved pre-fix binary ran each twice: five passed; `word_div_mod` and
`u32_divmod` exhausted the native work budget with empty stdout and status 1.
The generated port and upstream backends agree on the expected successful
results. This is an implementation cost gap against the unchanged corpus,
not permission to waive the fixture as a resource-policy exception.

The pinned [Base division body](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L1370)
performs bitwise long division and repeated comparisons/subtraction. Native
execution followed that checked body for packed words, accumulating enough
reduction/collection work to exhaust the fixed budget in the complete programs.
The [compiled operations](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L182)
use machine unsigned division and remainder: `a / 0 = 0`, `a % 0 = a`.
The bundled port division bodies are identical to this pinned Base section.

## Scoped correction

The native [ordinary-operation optimization](../src/runtime/numeric.rs) now
includes sealed Base `U32.div` and `U32.mod`, using Rust checked unsigned
operations with the same zero-divisor results. The exact signature check retains
the divisor's `+b` quantity; it does not relax other parameter checks or mint
an opaque numeric/proof assumption.

Only fully available packed words take the shortcut. Outer arguments are still
demanded in source order. Encountering an ordinary constructor enters the stored
checked body with the original thunks, preserving lazy Word fields, partial calls
and errors. No additional field is forced to obtain the optimization. The
2,000,000-step, 131,072-slot and 4,096-frame limits are unchanged, as are generated
C/JS, error boundaries, CLI contracts and all pending demand decisions.

Four new native regression tests cover the unchanged complete programs,
full-width/zero boundaries, partial application, ordinary reconstructed words,
first-error/output order, bounded work and cancellation. Existing lazy-field,
provenance, exact-signature and collection controls were extended. In particular,
`div(lazy_word, 0)` ignores the unused field, `mod(lazy_word, 0)` permits shallow
inspection, and a demanded bad divisor or outer argument still refuses safely.
Copied upstream fixtures remain unchanged and attributed under Apache-2.0.

An initial test used nonexistent `String.cat`; its corrected version prints the
two results separately. A new nonzero-division probe under collection at *every*
safe point exhausted the existing accounting budget. Normal collection passes
that ordinary fallback case. The forced-collection regression uses zero divisors
to verify retained fallback arguments without increasing the budget; the original
nonzero stress failure is retained in private logs. No existing test was removed
or weakened, and that stress profile is not claimed as qualified.

## Exact results

Every row below ran twice in native Rust, generated port C/JS and upstream C/JS.
All 70 executions returned status 0, empty stderr and these exact stdout bytes:

| Unchanged upstream fixture | stdout |
| --- | --- |
| [run/word_wrap_chains](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/run/word_wrap_chains.bend) | `4\n` |
| [run/word_div_mod](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/run/word_div_mod.bend) | `9\n` |
| [run/word_compare](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/run/word_compare.bend) | `6\n` |
| [run/word_bitwise_literals](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/run/word_bitwise_literals.bend) | `11\n` |
| [run/bit_walking](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/run/bit_walking.bend) | `5\n` |
| [base/u32_divmod_zero](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/base/u32_divmod_zero.bend) | `6n\n` |
| [base/u32_divmod](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/base/u32_divmod.bend) | `14n\n` |

| Check | Result |
| --- | --- |
| Full Linux standard gate | **776 passed, 0 failed, 49 ignored**, exit 0; formatting, strict Clippy, all-feature build and Node-provider checks passed. |
| Strict all-target/all-feature Clippy | Passed, exit 0. |
| Focused native numeric, packed-word, reclamation and library checks | **235 passed, 0 failed, 1 ignored**. |
| Expanded positive suite, 29 cases twice | **278 exact / 290**; twelve unchanged upstream-JS Bun-host failures. Includes all 70 word observations above. |
| Scheduler renewal | **72 exact / 81** serial/multiworker observations; nine unchanged Bun failures. C workers 1, 2 and 4, three repetitions. |
| Six runtime-negative fixtures | **44 exact / 56**; eight resource-policy, two native foreign and two Bun gaps remain. Runner exit 1 is intentional; cleanup checks pass. |
| Sixteen positive comptime fixtures | **160 exact / 164**; four upstream-default-JS stack failures retained, supplemental stack-4096 runs pass. Every port execution passes, including original hand/tmpl. |
| Pending demand and wildcard review | All **50** original demand observations and **10** forced-wildcard observations unchanged from the timer checkpoint. No proposed contract was implemented. |

All repeats are deterministic and all child timeouts are false. Generation,
compilation and execution retain 60-second deadlines. Word C runs use
`--threads 1 --gpu off`; port C retains strict warning flags. Final binary snapshots
all have SHA256 `e2139684013ab0e8405b683f6d18465237214149e60cb58396ef32831f26d4b9`.

## Remaining requirements

The ledger now links **48** upstream fixtures, leaving **1,254** inventory rows
without a linked runtime receipt. All prior 41 rows were renewed on this candidate
with their previous explicit limitations. The seven new rows pass all five recorded
runtime lanes; their direct upstream CLI and broader checker/API coverage are
still unverified. This does not qualify whole inventory rows or establish parity.

The acceptance criterion is unchanged: complete lane/API review and zero required
missing, blocked or failed cells in the named profile. Still open are the pending
three demand-contract choices, resource-policy differences, arbitrary native
foreign support, unavailable Bun/direct-IO oracle lanes, mixed-duration timer
qualification, broader corpus/API coverage and fresh platform evidence. The
existing KernelError external literal/pattern compatibility caveat and requirement
to inspect nonzero `run_main` Ok statuses remain; this change adds no public API.

The next small independent CPU step is the unlinked operator/representation group:
`run/u32_mod_operator.bend`, `reg/word_copy_native.bend` and
`reg/u32_structural_view.bend`. The last fixture's comment describes a C emission
refusal; that must be reproduced and classified against the actual pinned oracle,
not accepted as a waiver from the comment alone. None has been qualified here.
CPU verification can continue in the cloud; no GPU desktop is required for it.

## Reproduction and retained evidence

With the activated frozen tools, `CARGO_NET_OFFLINE=true` and `BEND_GPU=off`:

```powershell
cargo test --locked --no-fail-fast --lib --test runtime_packed_words --test runtime_numeric --test runtime_reclamation
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo clippy --locked --all-targets --all-features -- -D warnings
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <reference> -OutputDirectory <fresh-word-dir> -Suite words -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <reference> -OutputDirectory <fresh-positive-dir> -Suite all -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <reference> -OutputDirectory <fresh-scheduler-dir> -Suite scheduler -Repetitions 3
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <reference> -OutputDirectory <fresh-negative-dir> -Repetitions 2
```

The final word receipt is the seven-row subset of `-Suite all`; the standalone
`words` command is also available. Private `cpu-word-operations` evidence retains
the audit, pre-fix binary and repeated failures, initial and passing test logs,
exact command arrays, bytes/statuses, fixture/binary hashes, final receipts,
worker/comptime/demand probes and summary assertions. Previous checkpoints and
logs remain intact. Upstream and docs checkouts, dependencies, lockfile and
toolchains are unchanged. Fresh Windows/macOS, GPU/hardware profilers, Bun and
remaining lanes listed above are unrun. Commits are local; no push occurred.
