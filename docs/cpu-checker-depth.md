# Checker depth accounting at the pinned CPU target

Source fix `2f349c8f1df48047df039688f04ff1dcdc7bfc1f` qualifies the unchanged
[combined observation program](../tests/fixtures/cpu-float-specials/float-specials-combined-depth.bend).
The 128-level nesting guard, input inspection bound and 2,000,000-step work
budget are unchanged. Full CPU parity remains incomplete.

## Diagnosis and repair

The original SHA256 is
`2cfb8c79e614785350d3b55fc23c5edd51f56c6790d453acde2f766e82ef0846`.
It previously failed before execution in all three port paths, while pinned
upstream C and JavaScript produced the same seventeen output lines. The separate
factored helper control was never treated as passing evidence for this source.

Removing only leading IO continuations from private copies isolates the growth:
0, 8, 12, 13 and 14 continuations passed; 15 and all 16 refused. Temporary stack
instrumentation showed 87 checking frames, 36 inference frames, one reduction
frame and four freshening frames occupying the 128-level budget before another
type-freshening descent. Thirty-five inference frames were entered by checking
**the same term**, rather than descending into a child. The 32-bit constructor
representation and nested IO continuations exposed this duplicate accounting;
no numeric evaluation had begun. Instrumentation is absent from the fix.

The [fallback checking rule](../src/kernel/check.rs) now dispatches directly to
`infer_inner` under its active checking frame, retaining the inference work
charge. Recursive child inference/checking, beta expansion, reduction,
freshening and comparison retain their guarded entries. Inferred-type comparison,
quantity usage, dependent substitutions and error order are unchanged. This
removes a redundant phase-entry wrapper; it does not skip a checking rule or
relax the global nesting/fuel limits. The pinned
[bidirectional rules](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3418)
also delegate non-introduction terms from checking to inference.

Regression coverage includes the exact original source in native CLI tests and
in the full differential suite; ninety nested identity applications with valid,
wrong-typed and duplicated-affine-variable leaves; a deliberately exhausted
work budget; and a nonterminating beta expansion that must still hit the nesting
guard and unwind. Existing deep-input, unsafe normalization, false-proof,
termination, quantity and runtime budget tests remain active. An initial new
assertion expected the wrong diagnostic substring; it was corrected to the
actual affine-use error. No production behavior was changed to satisfy it.

## Results at the source commit

| Check | Result | Remaining boundary |
| --- | --- | --- |
| Standard gate | **818 passed, 0 failed, 49 ignored**, exit 0 | 47 CUDA tests and two optional profilers unrun |
| Focused checks | 32 passed, 0 failed, 0 ignored | Original source, types/quantities and budget controls |
| Strict all-target Clippy / build | Both exit 0 | Frozen toolchain and lockfile |
| Exact combined original | **10/10 exact**, exit 0; repeated in full matrix | Native, port C/JS, pinned upstream C/JS, twice |
| Positive matrix | **618/630 exact**, 63 fixtures, exit 0 | Twelve known Bun-FFI host failures; 53 originals and ten authored controls |
| Direct upstream entries | Four exact, 50 host-blocked | 48 positive-matrix observations plus six separately renewed collection observations; no direct-host pass implied |
| Runtime negatives | **44/56 exact**, runner exit 1 | Eight safety-limit, two native-foreign and two Bun-host observations open |
| Approved demand controls | **92/100 exact**, runner exit 1 | Six approved C-wildcard exceptions; two unwaived first-error-order differences |
| Normalization | 176 successful observations, 28 expected printer refusals; exit 0 | Eighteen original and four authored fixtures; exact structure/text |
| Comptime | **156/160 primary exact**, runner exit 1 | Four default upstream-JS stack failures; four supplemental stack-4096 passes |
| Strict audit | 1,302 fixtures; all **439 checker negatives rejected**, exit 0 | Status-only evidence; no acceptance changes |
| Adjacent checking | One exact acceptance; three diagnostic mismatches remain | Same rejection phase/construct is not exact diagnostic parity |
| F32.read | Corresponding backends agree; runner exit 1 | Three unchanged cross-backend contract disagreements |

The combined source has ten exact observations (five runtimes, twice), repeated
again in the full positive matrix. Every run produces the preserved seventeen
lines, empty stderr and exit 0. Exact finite/infinity/zero bits and text remain
separate from NaN text/classification; no NaN payload parity is inferred. The
[depth ledger](cpu-float-specials-depth-evidence.csv) retains historical refusals
at `9b18fbb` alongside the current passing observations.

The status-only audit still accepts 366 expected positives, refuses 487 and
rejects all 449 nonzero-expectation fixtures, including all 439 checker negatives.
No status changed relative to `bd6a228`; matching rejection does not establish
phase or raw diagnostic parity. A separate adjacent rewrite renewal retains one
exact check-only result and three phase/construct agreements with raw diagnostic
differences. Numeric printing in one diagnostic changed since its old `571d1ce`
receipt: `5` replaces the expanded U32 constructor tree. That predates this fix
and does not close the upstream diagnostic mismatch.

All matrix inputs are unchanged, source identities are recorded per receipt,
and each comparison child has a 60-second deadline (30 seconds for the adjacent
rewrite probe). All comparisons completed without timeouts and repeats were
stable. Upstream C uses its pinned CLI profile, `-std=c11 -O3 -lpthread -lm`,
with `--threads 1 --gpu off`; port C retains
`-std=c11 -Wall -Wextra -Werror -pedantic -pthread -lm -ldl`.
The original/positive/comptime/audit/adjacent probes use the pre-gate
`extended_observability` snapshot; negative/demand/normalization/read use the
binary subsequently built by the standard gate. Both snapshots come from the
same clean source commit; their distinct hashes are recorded below and per
receipt. Historical O0/O2, multiworker and platform receipts retain their original
commits.

## Commands and retained evidence

After sourcing the existing activation script, set `CARGO_NET_OFFLINE=true` and
`BEND_GPU=off`. `$reference` denotes the clean pinned checkout; `$evidence`
denotes a fresh private evidence directory. Exact expanded argument arrays,
raw streams, exit statuses, source/binary hashes and deadlines are saved with
the private `cpu-checker-depth` receipts. The private audit/comptime/adjacent
runner sources are retained there too.

```powershell
cargo test --locked --features extended_observability --lib depth_tests -- --nocapture
cargo test --locked --features extended_observability --test kernel_core --test executable_check --test cpu_float_specials
cargo clippy --locked --all-features --all-targets -- -D warnings
cargo build --locked --features extended_observability --bin teamy-bend --example compare_normalization
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference $reference -OutputDirectory "$evidence/original" -Suite float-specials-depth -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference $reference -OutputDirectory "$evidence/positive" -Suite all -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference $reference -OutputDirectory "$evidence/negative" -Suite negatives -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference $reference -OutputDirectory "$evidence/demand" -Suite demand -Repetitions 2
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference $reference -OutputDirectory "$evidence/normalization" -Suite all -Repetitions 2 -RequireExactText
python3 scripts/compare-float-read.py --reference $reference --output "$evidence/read"
python3 "$evidence/audit.py"
python3 "$evidence/probe-comptime.py"
python3 "$evidence/adjacent.py"
python3 "$evidence/direct-collections.py"
```

Full gate outer deadline: 1,200 seconds; strict audit outer deadline: 900 seconds.
The gate completed normally; its 49 ignores are 47 CUDA tests and two optional
profilers, still unrun. Initial failing traces/tests and prior logs remain.
Seventy identical inactive evidence binaries were hardlinked after hash checks,
reclaiming 9,624,817,664 bytes without removing a path or changing file content.
The deduplication receipt records every replacement and hash; build settings were
unchanged. No diagnostic instrumentation, private filesystem path or raw host
capture is committed.

| Receipt / input | SHA256 |
| --- | --- |
| original | `a5a57ef90aa948bf4c977358605f8628a716df5e58476a6013c354d42de82ef8` |
| positive | `682e92736e48a69e0921a8471b5a7e134f6cc28247b96eb25349f763c84330e7` |
| negative | `eefa609efab96301f293f6dc8bb21c758237b9adb3e264ef34809e04655ab206` |
| demand | `f1b76ce13e51c5d80dd19840ad212deefae75c396c05bd372db422a1120573a0` |
| normalization | `aaa07422075b5c2a3970b826abd0f48c5a488f673a1d79f6f84311ec61884e8a` |
| comptime | `f5cece33981583a64430603dcbf90ea6520361baa9a3159581aa1cf80f42351c` |
| read | `5eb867f6afb344efd37bb878c03e7751c8ff4121e1ffbc27cf06e9c28c37cb6a` |
| audit | `da8fd333e7e33c02dd8f338948731a77b6ccce13430cd72ba71ef38736bac155` |
| adjacent-renewed | `0d3d71b37fba62fbed05950b891fe7df2a28f34c327cf106b6d95e853100f9fb` |
| direct-collections | `b58e7e221244bb7a4fe6f083e452e24dfee84909dd2b64ca40a5c327d588bc1a` |
| Pre-gate comparison binary | `016dc4c77da411664ed79a7f931709129ba3ac00243110b963c20ec2d8ec361e` |
| Gate-built comparison binary (negative/demand/normalization/read) | `290264d4ceef5169d21efca65d5c1883a5ee90e126b95dbe3e66cc996cafd745` |
| Normalization probe | `70353d79d9aeb69b4ce3379bd6051ca6e0a42b6f8e01ca80a10d50e1e63eb984` |
| Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |

## Remaining acceptance requirements

The [acceptance criterion](cpu-parity-acceptance.md) is unchanged. The runtime
ledger still links 75 originals; with eighteen interpretation originals, 93
have these receipts. The remaining 1,209 are classified by applicability:
439 checker negatives, twelve check-only modules, 153 interpreter-only cases and
605 compiled candidates. This authored regression adds no upstream inventory row.
All current runtime/interpretation/demand receipts were renewed at the fix.
Historical multiworker/platform receipts do not qualify this new commit.

Three F32.read C/JS disagreements, competing first-error diagnostics, eight
negative resource-policy observations, native foreign support, unavailable
Bun/direct-IO oracle lanes, broad per-lane/API qualification and the existing
[downstream API caveat](cpu-nat-failure-contract.md#review-of-rust-api-consumers)
remain open. The approved private-wildcard C exception remains narrowly named;
no new exception or parsing policy is introduced. Dependencies, lockfile,
toolchains, pin, approved demand semantics and budgets are frozen; no push.

Next independent bounded family: normalize `parse/chain_suffix_f32_f`, `_g` and
`_h`, first establishing actual upstream function-printer applicability. This
work does not require a runtime F32.read policy decision or GPU hardware.
