# Rewrite checking and parse diagnostics

Tested source **`4c90907a78db96cc9e46da6d25659f10b4d978cc`**, following documentation checkpoint
`b7be1951d851acef50f9232b3f4e79ff2165d769`, closes the recorded diagnostic cells for
three unchanged originals at upstream **`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`**.
Full CPU parity remains incomplete. No acceptance, evaluation or resource-policy
exception was introduced.

## Actual applicability and corrections

| Original | Actual phase | Initial state | Final state |
| --- | --- | --- | --- |
| parse/rewrite_non_equation | Checking | Already exact at the preceding source | Exact renewed receipt |
| parse/rewrite_ctor_brace | Checking | Correct carrier rejection, host-style diagnostic | Exact normalized expected/observed types, context and source |
| parse/dead_motive_syntax | Parsing/loading | Correct token rejection, host-style diagnostic | Exact expected token, observed character and source |

The [original fixtures](../tests/fixtures/cpu-rewrite-checking/README.md) retain
upstream bytes, comments and Apache-2.0 attribution. Independent load/check probes
establish the phases; a nonzero exit alone is not the evidence. No main program
is executed, and no generated/native output artifact is created for these failures.

The pinned [checking rules](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3597)
reject the ill-typed motive. The port now retains structured expected/observed
terms at the existing inferred-type comparison failure. Display normalizes them
and their context using a clone with the remaining allowance. Rejection order,
checker work and guards remain unchanged; exhausted presentation work still
falls back to the underlying error.

Substituting the motive binder can make several variable occurrences share one
replacement allocation. The optional source index now retains independently
anchored equality-operand locations through that substitution. It does not copy
or modify terms, add strong edges, change IDs, alter serialization, or guess
locations from diagnostic strings. An independently sourced replacement keeps
its origin; an untracked operand does not acquire an invented anchor. Tests
verify distinct locations with a shared replacement, metadata cleanup, original
term identity, work counters and successful evaluation.

Recovering the context exposed missing parentheses around lambda equality
endpoints. The [pinned equation printer](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1402)
uses precedence one for both endpoints and the carrier. The port now uses its
existing precedence helper for all three. Four syntax-only printer observations
match the pinned parser/printer and retain every parsed AST field, including
binding IDs. This is presentation evidence, not a claim that those open terms
are checked executable programs.

The [pinned parser failure rule](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1493)
reports the expected token and first observed character, or end of input.
Token expectation failures now retain these facts in the private load diagnostic.
Original excerpts preserve imported-source ownership, CRLF and missing final
newlines. The public ParseError and loader signatures remain unchanged. The
private optional payload is boxed to keep Result error size within strict Clippy.

## Failure boundary and API implications

The existing checker/load writer emits language stderr and returns status-1
CliOutput for classified failures. Check, eval, run and executable compile now
use this boundary for the newly classified cases. Tests cover text/JSON/CSV,
write/flush failure, cancellation and ordinary untracked host-error fallback.
C/JS/native compilation refuses before staging or invoking a native toolchain;
pre-existing output remains byte-identical even with `--force`.

Repository production dispatch propagates CliOutput, and `src/lib.rs` reads
`exit_code()` before emitting and returning the process status. No additional
production invoke caller was found. **External callers remain unaudited:** more
classified failures can return `Ok(CliOutput)` with status 1 instead of `Err`.
Callers must inspect the exit code. Public signatures are unchanged; diagnostic
and equation Display text are intentionally more precise, so string consumers
must not assume byte compatibility with previous host-style errors.

## Validation and provenance

| Check | Result at tested source |
| --- | --- |
| Standard formatting, strict Clippy, build, provider and tests | **901 passed, 0 failed, 49 ignored**, exit 0 |
| Three originals plus ten adjacent negatives | 260 observations: 234 exact stream/status results and 26 phase probes |
| Original failure phases | Two checking; one parse/load |
| Normalization suite | 106 cases / 1,200 observations: exact structure/text; 842 successes, 352 printer/artifact refusals, six strict-F32 exclusions |
| Constructor/origin and Nat/List renewal | 42 cases / 588 observations pass |
| Previous plus-binder negative matrix | 216 observations pass |
| Previous local-binder and rewrite diagnostics | 60 observations each pass |
| False-list/brace and positive proof controls | 144 observations pass |
| No-main checking, compilation and named normalization | 306 observations pass |
| Four prior plus-binder IO originals | 40 exact executions; eight direct-oracle Node host blocks |
| Serial C/JS/native baseline | 50 exact executions |
| Checking-status audit | 1,302 rows; zero transitions from e9d9b30 |

All final repetitions are deterministic and finish within bounds. The 439
expected checker negatives still reject; other statuses remain 366 accepted and
497 rejected. Eight direct upstream IO attempts still fail with
`TypeError: require is not a function`; generated upstream JS passes. These
host-blocked cells remain blocked. Hardware/profiler ignores remain unrun.

The focused candidate passed 283 tests with one ignore before the source commits;
its receipt records dirty base b7be195 and is supporting evidence, not a clean
source claim. The first clean source, 920b742, passed its retained comparisons
but failed the full gate at strict Clippy (formatter length and private Result
error size), before full tests. The final source extracts the formatter helper
and boxes only private diagnostic metadata; its gate and matrices are renewed.

Candidate failures remain saved: EOF display location, the new printer test's
unsupported bounded Shape vocabulary, a misplaced test assertion, a malformed
authored multiline probe, and an oracle adapter that initially omitted the
upstream higher/lower conversion. The final metadata summarizer also initially selected full-lane agreement where
completed-lane agreement was required; that correction retains every host-blocked
cell. Corrected receipts are separate. Original
fixtures and all unique historical failure evidence remain untouched. The
previous **688-vector exploration remains incomplete and unqualified**.

## Coverage and next work

The detailed inventory union remains **213 / 1,302 originals**, including
historical and unresolved cells. This batch closes existing diagnostic rows;
it does not manufacture additional inventory coverage from authored controls.
There are **1,089 without detailed receipts**: 429 checker negatives, one
checking-only module, 106 interpreter cases and 553 compiled candidates.

This source renews twenty of 21 checking originals, all 99 interpretation rows,
and seven of 93 runtime rows. One checking and 86 runtime rows retain historical
commits; prior plus-binder helper-oracle rows also retain their original identity.
The [acceptance criterion](cpu-parity-acceptance.md) remains unmet: complete
lane/API coverage, original resource failures, native foreign support, Bun/direct
IO host availability, F32.read and competing-error ordering remain open.

The next coherent independent family is page/rewrite_non_equation,
check/error_window_rewrite_proof and check/missing_ih_rewrite: annotated
non-equation evidence, generated proof-binder source location, and missing-IH
proof diagnostics. They lack detailed receipts and are not marked qualified here.
CPU progress does not require desktop or GPU.

## Reproduction and storage

After the existing activation script, the exact host overrides are:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --lib --test cli_checker_diagnostics --test cli_compile_executable --test kernel_normalization --test kernel_core --quiet
cargo build --locked --all-features --bin teamy-bend --example compare_normalization
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <clean-pin> -OutputDirectory <fresh-directory> -BinaryDirectory <saved-snapshot> -Suite all -Repetitions 2 -RequireExactText
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <fresh-directory> -BinaryDirectory <saved-snapshot> -Suite plus-binders -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <fresh-directory> -BinaryDirectory <saved-snapshot> -Suite baseline -Repetitions 2
python3 <private-evidence>/matrix.py --name matrix --binary <saved-cli>
python3 <private-evidence>/constructor-compare.py --binary <saved-cli> --output constructors
python3 <private-evidence>/qualify.py --output no-main-final --snapshot final-snapshot
python3 <private-evidence>/marker-matrix.py
python3 <private-evidence>/diagnostics.py
python3 <private-evidence>/proof-regression.py
python3 <private-evidence>/local-binding-matrix.py
python3 <private-evidence>/audit.py
python3 <private-evidence>/constructor-summarize.py
python3 <private-evidence>/verify.py
```

The private cpu-rewrite-checking evidence and validated subdirectory preserve
exact argv, source/binary hashes, raw streams, statuses, timing and output absence.
Fixed-output scripts need fresh destinations. A shared immutable snapshot is used
per source build. Existing historical inputs are referenced without duplication.
New retained evidence uses about **127.5 MiB**;
no unique evidence was deleted. Reclaimed files are inactive rebuildable Cargo
caches only. The retained inventory's dominant cost remains historical ELF
binaries (about 18 GiB); those were preserved.

Every Cargo/full-gate preflight met the unchanged 10-GiB reserve. Full-gate minimum was
**8.91 GiB**, with **10.04 GiB** free at the final receipt.
Process bounds remain 30/60 seconds per child and twenty minutes per outer run,
with workspace/tmp stop floors of 2/1 GiB. Host debug/incremental overrides are
storage controls, distinct from upstream C O3 and `--threads 1 --gpu off`.

Rust 1.96.0, nightly-2026-05-24 formatting, Node 24.19.0, PowerShell 7.5.4,
Clang 19.1.7, dependencies, upstream/docs references and all Bend budgets/guards
remain frozen. Cargo.lock SHA256 is
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
No installation, push, desktop/GPU or security-policy change occurred.

## Receipt hashes

| Receipt | SHA256 |
| --- | --- |
| matrix/receipt.json | 8bdaacb3fd6a65a9583af37fac5ddaab134a5c82b4156cc857181863e4777429 |
| full-gate.json | caf4663143968207935354744b5c928bc224a7972f7e290e61a252a7b3b2c289 |
| full-gate.log | 1cc93da1cd545e5df28c4dd04652549c6fe2e614fede659cfb09666094c21be8 |
| normalization/receipt.json | c57cfd87062f933fe7308a41714972f4a57cbfb578d5e5f4ce29555a935a5b4f |
| constructors/receipt.json | 6ad695616feab17f9d60a86f665447326da8e662dc31c90bc44637715091b861 |
| audit/receipt.json | 5a5d508bc7a08482fff610083a85a45001932bfec246d6052aa486a9d9d5de05 |
| snapshot.json | 7b4857b02a5f4454289868fc84311346a353f2b19d24b2bd6c841cca8c9b8f2a |

CLI SHA256: `fd37725da21127d4e9db032051e2317bde012231d3d7e1531beb82d1ba57aea1`.
Probe SHA256: `6082da73d302be6f92df6696e1615677e8605ab1000b9e53b78fb68af4ef42de`.
The later documentation commit records this tested source; it is not another
full-gate execution. No task-owned test remains running.
