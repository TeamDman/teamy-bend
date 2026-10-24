# Compilation/rewrite diagnostics and nested-field qualification

Tested source **`02b244e529e978a683b7f77a79ae04089665748d`**, following local checkpoint
`f59353117b871b0f88a97ec49451ae49796f4920`, closes the two diagnostic gaps
recorded in the [previous report](cpu-no-main-interpreter.md). The pinned target
remains **`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`**. Full CPU parity is still incomplete.
No dependencies, toolchains, evaluator policy, guards, budgets or cancellation
behavior changed. No upstream source was modified.

## Diagnostic corrections and compatibility

For a checked no-main book, both executable emitters still return Err before
artifact construction. Private typed metadata now distinguishes the missing-entry
condition from unrelated compiler failures. It records whether the book actually
has an unqualified IO declaration, matching pinned
[show_main](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1203).
The CLI prints exactly `Error: no main to run` followed by a newline when IO is
present, or `Error: a build needs import Base` followed by a newline when absent.
It is a declaration check, not a search for a literal import statement; synthetic
IO datatype and definition controls agree with upstream. No blanket Base
restriction was added to existing executable programs.

The nine original no-main modules retain successful checking/default-run behavior
and thirty exact named normal forms. The renewed 306-observation matrix now
qualifies all 72 upstream/port C/JS build refusals with exact status/stderr,
empty stdout and absent artifacts. Regression tests also cover native compilation,
existing output preservation with `--force`, cancellation and failed diagnostic
writes/flushes. Other compiler failures keep the host diagnostic fallback.

Public executable-emitter signatures, Err behavior, Display text (`no main to
run`) and message-based equality are unchanged by this batch. Private metadata
is now visible in derived Debug output; Debug representation is not claimed
unchanged. The prior successful-stub-to-Err correction remains a compatibility
change for external callers. Repository production use consists of the two
branches in `src/cli/compile/compile_cli.rs`; both propagate Err before writing
output or invoking a native compiler. Tests are separate callers.

`CompileArgs::invoke` now returns Ok with `CliOutput::exit_code() == 1` after
successfully writing a classified diagnostic; it does not return an eyre Err
for that particular failure. Repository dispatch forwards the output,
`src/lib.rs` reads the status and flushes output, and `src/main.rs` exits only
after the guards have dropped. External direct invoke callers were **not
audited** and must inspect the exit code; checking only Result::is_ok is not
sufficient. The earlier KernelError private-field compatibility caveat remains.
No external consumer compatibility is inferred from repository tests.

For false map fusion, the existing rewrite goal mismatch now attaches structured
expected/observed types and source origin. The pinned
[rewrite checker](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3597)
checks the specialized goal before checking the body. The port preserves that
order and rejection. Existing rendering normalizes on a clone of the remaining
checking allowance; presentation cannot replenish fuel or increase the depth
limit. If presentation exhausts its allowance, the original failure survives.
Parser-side provenance tracks rewrite nodes through the existing weak side table;
there are no new Term fields, term identity/equality changes, strong sharing,
serialization changes or evaluator changes.

The unchanged false-fusion reproducer now reports normalized expected/observed
types, `fuse` and source line 52 exactly. Five adjacent controls cover direct,
imported, template, nested and body-first failures. Across two repetitions,
48 CLI observations match raw status/stdout/stderr and 12 direct phase probes
confirm checking rejection before execution. All six cases reject with status 1
and empty stdout. Existing false-list/brace diagnostics and positive normalized
controls pass 144 renewed observations. Counter/state/term invariance, cancellation,
negative guards and writer failures remain covered by the full gate.

## Ten nested-field originals

`flatten/nested_field_match_000` through `_007`, plus `_004_go` and `_007_go`,
are unchanged attributed copies. Actual pinned interpretation agrees with each
original golden value, strict normalized core, typed eval and native pure output.
The reusable `compare-normalization.ps1 -Suite nested-fields` selection covers
all ten. The complete suite now has 76 cases: 69 originals and seven authored
controls. It preserves actual-core comparison and exact text; no normalization
repair or tolerance was introduced.

These function-valued mains do not support standalone result printing. Original
upstream generation refuses the missing Base/IO prerequisite. Separate authored
probes add Base and prefix local names, confirming function-printer refusal in
both C and JS, twice: 40 refusals. These are applicability observations, not
compiled execution passes. The unchanged originals remain the basis for
interpreter qualification.

A direct upstream core adapter applied each original to constructor arguments,
covering five Nat depths, parameter/field combinations, Leaf/Node alternatives,
sibling fields and helper functions: **171 distinct vectors**, observed twice
(342 values). The committed table drives checked Rust applications with exact
value/core assertions in the clean-source gate. These bounded vectors do not
prove equivalence for all possible inputs. Oracle-only probes were collected
before the source commit against the same clean pin; the renewed full gate and
76-case matrix qualify the committed port. Their receipt identities remain separate.

## Clean-source validation

| Check | Result |
| --- | --- |
| Standard format/strict Clippy/build/provider/test gate | **882 passed, 0 failed, 49 ignored**, 98 result-bearing targets; exit 0 |
| Focused candidate integration tests | 46 passed before commit; included in the clean full gate |
| Exact normalization suite | 76 cases, 840 observations: 602 successful, 232 artifact/printer refusals, six strict F32 exclusions |
| No-main matrix | 306 deterministic observations, all applicable contracts pass |
| Rewrite diagnostic matrix | 48 exact CLI observations plus 12 checking-phase probes |
| Prior proof controls | 144 observations pass |
| C/JS/native baseline | 50 exact executions at pinned upstream O3, two repetitions |
| Checking-status audit | 1,302 rows, zero status transitions from 2417a271 |

All 439 expected checker negatives still reject. The other audit rows remain
366 accepted and 497 rejected; status agreement alone is not semantic parity.
No timeout or abnormal exit occurred in the final matrices. Expected diagnostic,
printer and strict-proof refusals are distinct from failed validation. All 49
hardware/profiler ignores remain unrun. Historical negative/demand, constructor,
Nat/List and broader host/resource receipts remain tied to their actual commits;
regression inclusion is not a newly executed full differential matrix.

Resolved candidate failures are retained privately: a test-only import typo,
Clippy style errors and a failed edit-script precondition. They do not appear in
the final gate and no warning was suppressed. The earlier failing diagnostic
receipts remain preserved rather than overwritten.

## Coverage and remaining work

| Category | Inventory | Detailed originals | Without detailed receipt |
| --- | ---: | ---: | ---: |
| Checker negative | 439 | 8 | 431 |
| Checking only | 12 | 11 | 1 |
| Interpreter only | 164 | 42 | 122 |
| Compiled candidate | 687 | 115 | 572 |
| Total | 1,302 | 176 | 1,126 |

These are linked detailed originals, including historical and unresolved cells,
not universal passes. Runtime/interpretation union: 157. All 69 interpretation
rows, ten checking rows and three runtime rows have renewed receipts; nine
checking and 85 runtime rows remain historical. Authored negative controls and
concrete argument vectors do not inflate inventory coverage. `io/far_types`
remains the sole checking-only module without a detailed receipt and requires
separate foreign applicability review.

The [acceptance criterion](cpu-parity-acceptance.md) is unchanged. Full API/lane
review, F32.read disagreements, competing-error ordering, whole-program resource
limits, native foreign support and unavailable Bun/host oracle lanes remain
unmet where previously recorded. Neither diagnostic closure creates an exception.
The next independent bounded family is the four `nested_multi_scrutinee_000`
through `_003` originals: nested pattern columns, fallback reconstruction and
binder reuse, with printer applicability established before execution claims.
No desktop or GPU access is required for that work.

## Reproduction and resource controls

After activation of the frozen environment:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test cli_checker_diagnostics --test cli_compile_executable --test cpu_no_main --test kernel_normalization --quiet
cargo build --locked --all-features --bin teamy-bend --example compare_normalization
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <clean-pin> -OutputDirectory <fresh-directory> -Suite all -Repetitions 2 -RequireExactText
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <fresh-directory> -Suite baseline -Repetitions 2
python3 <private-evidence>/qualify.py --output <fresh-name> --snapshot final-snapshot
python3 <private-evidence>/diagnostics.py
python3 <private-evidence>/proof-regression.py
python3 <private-evidence>/audit.py
python3 <private-evidence>/verify.py
```

The private cpu-compile-rewrite-diagnostics directory holds bounded runners,
exact argv, hashes, stream bytes, statuses, determinism and disk receipts. Fixed
output paths require fresh destinations on rerun. Child deadlines are 30 or 60
seconds; the command wrapper enforces a 20-minute outer deadline. Public files
contain sanitized receipt identifiers only.

Rust 1.96.0, nightly-2026-05-24 formatting, Node 24.19.0, PowerShell 7.5.4 and
Clang 19.1.7 remain frozen. Four host build jobs, debug=0 and disabled incremental
are temporary host build controls, not Bend runtime optimization changes. The
baseline uses upstream's O3 / -lpthread -lm profile, `--threads 1 --gpu off`;
port C retains strict C11 warnings and -pthread -lm -ldl.

Builds started above the 10-GiB reserve. Full-gate minimum workspace headroom was
**9.38 GiB**; stop floors stayed 2 GiB workspace / 1 GiB tmp.
Rebuildable executable-cache growth and retained binary snapshots account for
the measured usage. Only inactive rebuildable test executables and three obsolete
example-cache executables were reclaimed. The first cleanup left a 6-MiB reserve shortfall;
the recorded additional cache cleanup restored **10.02 GiB**. Source, protected
binaries and all historical evidence remain intact. No dependency, network/security, installation, push, desktop or
GPU action occurred.

| Evidence | SHA256 |
| --- | --- |
| no-main-final/receipt.json | 1898d90a5cd11bcb0a986f1ab29c431e95158c9b6646f1b01b7c74b24c07dab7 |
| normalization/receipt.json | 46b591c25493b5e1ec7ae6a969352bcd51f6b5fec0844a8303161e302bd67a1b |
| nested-adjacent/receipt.json | 43c060b3a155aab7838edd35f175a22c1aed86de6d37bf42cb53f466fa0ffd7f |
| audit/receipt.json | a8e5f4810b47845f4e6d441ca88e924502aec08cbe2330b1ddd27a6ea5bd9461 |
| baseline/receipt.json | 7bcc5624c190a3e9eec355d4ff70a94713aa0380134bcc2ec120a7e3ba6b0ece |
| full-gate.log | 39524e1d82a7d08db9b5a88be5c2964083009b37dbd1c65ab0de438668c18316 |
| full-gate.json | 57a7cfea3f57760141c59185242e453cafae324a51c8496680fb13aa85c7ad7c |
| diagnostics/receipt.json | aa501c4c5b91e87090700aee25d73c37119eb82d98ba67774927dc5a889e7584 |
| proof-regression/receipt.json | 88be3828a386d75b53b0364fb4b0a2f317b79dd4859a8f38dfcc8ff8262029cb |
| caller-audit.json | 0c3f29a4b2b6547359e5efe739212bff408bec17bf2e4e3e41d6413244b5064e |
| nested-provenance.json | 0647b591b736f4cd49d15a16afc2c9053927e2ccfa4cd9b9a2acce34e231f3db |
| final-cache-preflight.json | 80d790c1588bfe84cac37694fc2de69cebd5f38bb369bea127306a85e482f3ae |
| obsolete-probe-cache.json | f5c7a273d332bb20efeafc92a99a564dac364c9454aca016039b0b9322c89f54 |
| final-cache-complete.json | c0ff9839b92f119531d2a04ec698924bff91a1c7472102125440ae9f67c8fd35 |

Tested binary SHA256: `dd22ee1d94236ecf1491a3fe601147eda9a1e306e566e60cbc76f78084943a9d`.
Normalization probe SHA256: `4b27a16243f30f6c954af3987633d4a0b40ef8e6b04700049793400c2b6642a1`.
Cargo.lock remains `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
The upstream pin and teamy-docs `50e89d6897ce2a65167ef65b17c5fa83fe849158`
remain clean. The later documentation checkpoint reports this tested source;
it does not claim another gate execution. No test is left running.
