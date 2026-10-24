# Constructor/import-negative qualification

Tested source: `6442b2ca1362e36f84b74044091a0151fb17726d`, following layout
checkpoint `09e352b52fa09ac3328d87cc1396bf8ec3ac407a`. Target remains Bend
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.

The five requested originals now match the pinned checker’s exact stderr,
empty stdout and exit 1 through port check/eval/run. All five actually fail
**checking**, before execution or code generation. The comment in
`import/list_sugar` claiming a parse failure is inaccurate; the unchanged file
and its comment are retained. No acceptance or evaluation semantics changed.
Full CPU parity remains incomplete.

## Scope and correction

| Unchanged original | Actual cause |
| --- | --- |
| check/nat_literal_import.bend | Undeclared fixed Nat sugar constructors in an inferred expression |
| check/tuple_import_required.bend | Undeclared Tuple sugar constructor in an inferred expression |
| import/list_sugar.bend | Undeclared fixed List sugar constructors in an inferred expression |
| parse/qualified_constructor.bend | Nat.Succ is not the declared Succ constructor of Nat |
| page/ctor_list_mismatch_000.bend | Tru is not declared in MB, whose constructors are T and F |

The pinned [inference rule](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3399)
and [constructor-checking rule](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3478)
distinguish an undeclared constructor, a constructor of another declared
family, missing/extra fields and an otherwise non-inferrable known constructor.
Its [diagnostic printer](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1437)
normalizes reported terms and context, then prints the original source span.

The port now records these structured checker facts in the existing optional
failure metadata. It reuses the checked stderr writer and status-1 result
boundary introduced for false proofs. No diagnostic string is parsed to decide
failure kind or source location. Internal KernelError messages, Display,
equality and runtime-failure classification remain intact. Presentation uses
one cloned engine’s **remaining** allowance for both terms and context, never
fresh fuel or a larger depth guard. Cancellation and write/flush failures still
propagate as host errors; untracked or unavailable presentation falls back.

The loader tracks constructor atoms and cons expressions by retained node
identity. Constructor mapping/substitution retains the allocation only when all
fields are unchanged. Parenthesized expressions retain the inner atom’s actual
line. Changed nodes remain untracked; there is no structural/string search for
a plausible source line, global registry, unsafe dereference or public AST edit.

A shared printer defect appeared in the List controls: an open canonical Con
chain printed as `Con{head, tail}` instead of `head <> tail`. The correction
follows pinned [list sugar and precedence](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1241).
It inspects existing syntax without evaluating fields. Thirteen cases verify
exact upstream text and structural round trips, including nested heads, open
Nat fields, lambdas, applications, complete lists, malformed tails and qualified
constructors. The initial raw-parser printer probe incorrectly exposed fresh
binder indices; its failed capture is retained. The corrected probe uses the
pinned higher/lower conversion, matching the established printer-probe method.

## Results and unresolved cells

The expanded matrix contains the five originals, eleven existing Nat controls,
thirteen existing List controls and seven adjacent constructor controls:
**36 cases, two repetitions, 504 observations**. Each observation preserves
argv, input hash, exit, stdout/stderr bytes, elapsed time and timeout status.
All repetitions are deterministic; no child exceeds its 60-second deadline.
All original and helper hashes are verified; the five copied originals are
byte-identical to the clean reference.

- **24 negative cases exactly match** across upstream direct/C/JS front ends
  and port check/eval/run: 288 rejection observations. These include five
  originals, five Nat cases, eight List cases and six adjacent cases.
- Four negative cases give another 48 rejection observations with matching
  phase/status but unresolved raw diagnostics, detailed below.
- Eight positives preserve all 96 baseline observations: 64 successful checks/
  interpretations, 24 verified no-Base build refusals and eight successful
  generations. Generation success is not a generated-program execution claim.
- Seventy-two direct phase probes distinguish checking, parse/load and acceptance.
  Port regression tests separately establish load success then checker failure
  for the original/adjacent checking cases; existing pattern tests establish
  the two load failures. Rejected generation requests leave no C/JS artifact.

The capture harness exits 0 on successful collection. The qualification summary
explicitly exits **1** because four diagnostic cells remain unmet. They are
not accepted exceptions:

| Case | Remaining difference and cause |
| --- | --- |
| Nat local-tail | The original `1n+n` error is correctly rejected, but substitution reconstructs its constructor and loses loader identity. Upstream prints the normalized term/context/Bend line; the port falls back to its internal constructor error and host context. |
| Adjacent context-open | `Wrong{n}` has the same lost-identity limitation after parameter substitution. A regression requires continued nonzero rejection without a guessed Bend source excerpt. |
| Nat local-pattern | Both fail during parse/load. Upstream reports unknown global Zero; the port reports an unknown constructor pattern through its host error boundary. |
| List local-pattern | The corresponding parse/load diagnostic difference for fixed Nil remains. |

The initial exact Nat-tail expectation failed and is preserved in the candidate
log. Its unchanged source remains in the final differential matrix and the
existing rejection coverage. Passing closed cases does not qualify this open
case. Resolving it requires explicit origin propagation through parser
flattening and checker substitution; retaining an unchanged Rc cannot do that.
This is implementation work, not a reason to change the language contract.

## Renewed validation

All final receipts below identify clean source `6442b2ca1362e36f84b74044091a0151fb17726d`.
Candidate build/test/Clippy failures are retained separately and are not relabeled.

| Check | Result |
| --- | --- |
| Standard check-all.ps1 gate | **866 passed, 0 failed, 49 ignored**, 97 result-bearing targets; exit 0 |
| Focused parser/templates/checker diagnostics/normalization | 76 passed |
| Depth-guard controls | 5 passed |
| Diagnostic allowance/cancellation/writer controls | 8 passed |
| Existing false-proof/adjacent/positive/normalization controls | 144 successful observations |
| Full normalization suite | 49 cases; 386 successful observations, 124 verified build/printer refusals, 6 strict F32 exclusions; exact structure/text |
| Serial CPU C/JS/native baseline | 50 exact executions; two repetitions, all five runtimes |
| Full pinned checking-status audit | All 1,302 checked; 439 expected negatives rejected; other 366 accepted / 497 rejected; **zero status transitions** from 67cdcdff91f69e823b77157cf5dab7a48ecf350f |

The audit is status-only and does not qualify those 497 other rejections or
unexamined diagnostics. The ignored hardware/profiler tests remain unrun.
Runtime-negative and demand matrices keep their prior actual receipts; they
were not renewed here. F32.read disagreement, competing-error order, whole-program
resource limits, native foreign support, unavailable oracle hosts and API/lane
inventory work remain open. No pending discrepancy is waived.

Production CLI callers still propagate the returned exit code: Cli/Command
forward the result, lib::run inspects `CliOutput::exit_code()` before emitting,
and main exits with that value after guards drop. The documented status-1
`Ok(CliOutput)` boundary now also covers tracked constructor errors. Downstream
callers outside this repository must inspect the status; their compatibility
has not been audited. The pre-existing private-field KernelError construction
caveat remains; this change introduces no additional public field or AST shape.

## Coverage and next work

The updated [category ledger](cpu-coverage-summary.csv) has the same 1,302-entry
denominator. Detailed receipts include historical and unresolved outcomes:

| Category | Inventory | Detailed originals | Without detailed receipt |
| --- | ---: | ---: | ---: |
| Checker negatives | 439 | 8 | 431 |
| Checking only | 12 | 2 | 10 |
| Interpreter only | 164 | 15 | 149 |
| Compiled candidates | 687 | 115 | 572 |
| Total | 1,302 | 140 | 1,162 |

Runtime/interpretation union remains 130 originals; the five new entries are
checking negatives. Authored controls do not increase inventory coverage.
All 42 interpretation rows, three runtime-baseline originals, the false-proof
controls, Nat/List controls and six original checking rows have renewed receipts;
85 runtime rows and four other checking rows retain historical identities.
This is not a passing percentage. The [acceptance criterion](cpu-parity-acceptance.md)
is unchanged.

Next bounded diagnostic work: propagate source origins explicitly for rebuilt
constructor nodes, retaining the open Nat and Wrong{n} reproducers and the same
budgets, then address the distinct pattern-load boundary. The next independent
category batch is the nine ordinary no-main modules without detailed checking
receipts; io/far_types requires its separate foreign-mode review. Neither GPU
hardware nor a desktop is required for these CPU steps.

## Reproduction, profiles and evidence

Activate the frozen environment, then use these host settings and commands:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo build --locked --all-features --bin teamy-bend --example compare_normalization
cargo test --locked --features extended_observability --test cli_checker_diagnostics --test parser --test parser_templates --test kernel_normalization --quiet
cargo test --locked --features extended_observability --lib depth_tests --quiet
cargo test --locked --features extended_observability --lib diagnostic --quiet
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <clean-pin> -OutputDirectory <fresh-normalization> -Suite all -Repetitions 2 -RequireExactText
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <fresh-baseline> -Suite baseline -Repetitions 2
python3 <private-evidence>/compare.py --binary <immutable-snapshot> --output <fresh-name>
python3 <private-evidence>/audit.py
python3 <private-evidence>/proof-regression.py
python3 <private-evidence>/summarize.py
```

The final three private scripts retain their concrete snapshot/output targets;
choose fresh targets when reproducing so historical captures survive. Exact
argv and bounded-run receipts are retained in private cpu-constructor-diagnostics
evidence. Public ledgers contain sanitized identifiers and hashes only.

Rust 1.96.0, nightly-2026-05-24 fmt, Node 24.19.0, PS 7.5.4 and Clang 19.1.7 remain.
Host debug=0 omits symbols; CARGO_INCREMENTAL=0 prevents incremental cache growth.
Four build jobs remain. These host settings do not alter Bend optimization,
overflow checks or safety budgets. Separately, the baseline C oracle uses the
pinned O3 / -lpthread -lm profile, --threads 1 --gpu off. Port C keeps strict
C11 -Wall -Wextra -Werror -pedantic and -pthread -lm -ldl, without a new O3 flag.

The first focused-build preflight refused a 46-MiB shortfall below the 10-GiB
reserve. Only inactive rebuildable test executables were reclaimed; protected
binaries, source and evidence stayed intact. Full-gate minimum workspace
headroom was **9.92 GiB**; 2-GiB workspace and 1-GiB temporary stop floors held.
Focused renewals reused the completed gate’s artifacts. After all runs, the same
inactive-cache control restored **10.45 GiB** for the next build. Incremental
cache remained empty. No exhaustion, dependency/toolchain/pin/security change,
installation, desktop/GPU work or push occurred.

| Retained evidence | SHA256 |
| --- | --- |
| Final constructor/Nat/List matrix | 4e12a4d7ad42ca9759baf1bc11df1e0192624ea0a7abfd8816ee243e5cd7992d |
| Checking-status audit | 1afb69ea9fa40708fe9a219e3bba914adee7ba447021021f40974efa85e6b29c |
| False-proof regression receipt | d5cedf4bd5c2bd135d4a3ff936ffa0b5cb25eeda41be7c35583026556f3ebca3 |
| Full normalization receipt | 5c7aa17555d270738d87dd8a4e1b87f90ad1887025c09ff1a9fa297f676edb6c |
| C/JS/native baseline receipt | 2964e24c4bdd0ee6f3a218cf2ad44386136ecc9a8b468dce52663a06b80bf618 |
| Full gate log | cea0237b53dc7ad44a251e93e8b7cf2717d657a4ca76716b9ff2a23b61891991 |

Cargo.lock remains SHA256
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
Upstream and mounted teamy-docs remain clean at their recorded pins. Earlier
receipts retain their original hashes/commit identities; the documentation
checkpoint following this source does not claim a second execution of the gate.
