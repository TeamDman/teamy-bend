# Reconstructed values and row-arity diagnostics

Tested source **`77116c8ab4ab8eabbaea599679996941e41ed3b4`** closes the reconstructed matched-value and row-arity
diagnostic gaps against Bend **`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`**.
Full CPU parity remains incomplete.

## Demonstrated causes and fixes

The unchanged `flatten/error_window_matched_scrutinee.bend` rejects during checking.
Returning the reconstructed `One.new{}` where Type is expected previously produced
a generic expected-datatype error with Rust host context. The pinned
[constructor checker](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3479)
reports expected Type, observed One and the returned variable's source location.
The port now attaches that structured diagnostic after the same failed datatype
check. For an undeclared constructor the same rule uses the pinned
[typeless_show](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1446)
form. It is rendered on a clone with the remaining presentation budget; exhaustion
retains the underlying original error.

The existing private surface occurrence wrapper now also carries reply origins.
Generated lambda, match and let parents record per-edge anchors in the loader-owned
weak provenance index. Checking propagates an anchor only for a diagnostic rooted
at that substituted child, preserving independently sourced nested errors. Two uses
can therefore retain distinct locations while sharing one constructor allocation.
No field enters core Term or serialization; term identity, strong sharing, binder
IDs, substitution order, evaluation and resource accounting are unchanged. Tests
compare tracked/plain books, checker work/fuel/depth/fresh counters and successful
normalization, as well as direct pointer sharing for reconstructed replies.

Thirteen adjacent unchanged originals reject at parse/load on row arity. These
include the two arity_mismatch fixtures and old `as`/`2+k` spellings, whose actual
failure phases were verified rather than inferred from comments. The pinned
[parse_body_stmt](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2311)
checks the count immediately after the case colon;
[parse_fail](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1493)
reports the current raw character. The port's tokenizer had already skipped to the
body. Diagnostic metadata now uses the consumed colon's line/column and original
source to report the required count and actual whitespace, CRLF or EOF. The public
ParseError message, position and rejection remain unchanged.

[All fourteen original copies and attribution](../tests/fixtures/cpu-matched-values/README.md)
are byte-identical to the pin. Fifteen authored controls cover direct, nested,
field-bearing, let-tail, imported and templated reconstructed values; known and
unknown constructors; function goals; inline/multiline row counts; CRLF, EOF,
imports and templates. Existing source-hole and structural guards remain active.

## Exact validation

| Evidence | Result |
| --- | --- |
| Focused library/CLI run on the precommit source tree | 251 passed, 0 failed, 1 ignored |
| Standard gate at the clean tested commit | 920 passed, 0 failed, 49 ignored, across 98 result-bearing targets |
| Format, strict Clippy, all-feature build, Node provider | Passed |
| Fourteen original + fifteen authored + eight historical negative cases, twice | 740 observations: 666 exact stream/status and 74 separate phase probes |
| Phase probes | 48 parse/load, 26 checking; no program execution |
| Executable C/JS/native generation for these failures | Exact refusal, no output artifact |
| Positive worker/no-main controls | 14 observations passed |
| Erased-hole/TODO probes | 18 observations preserve three unresolved differences |
| 1,302 checking-status audit | 439 expected negatives rejected; other 366 accepted / 497 rejected; zero transitions from 1175aa3 |

Matrix and control repetitions were deterministic, with no child timeout or
resource-floor stop. Comparison children are bounded at 30 seconds, audit children
at 60 seconds, outer runs at 1,200 seconds. Exact stdout/stderr/status comparison
is separate from the phase adapter. CLI regressions cover text/JSON/CSV, unchanged
public load errors, retained compile outputs, cancellation, write and flush failures.
The 49 ignored tests remain unrun: 47 hardware and two optional profiler checks.

No C oracle binary is generated for these rejection cases, so this matrix makes
no runtime-profile or execution claim. Existing O3 runtime receipts remain
historical. The full gate renews repository regressions, including the selected
399 applications, but does not qualify the unfinished 688-vector exploration.
Unrelated detailed runtime and normal-form rows keep their actual historical commits.

The first candidate correctly handled the fourteen originals but omitted the
non-inferrable wrapper for the authored unknown-constructor case. Its exact failed
comparison remains in candidate-quick.json; the final structured renderer and
regression close it. This preliminary working-tree probe is not relabeled as the
final committed-source matrix.

## Remaining work and decision boundaries

The [current open ledger](cpu-matched-values-open-evidence.csv) retains three
unwaived differences: upstream accepts a beta-erased named hole and prints `0`,
while the port refuses the source hole; erased/direct TODO reach upstream's
post-check incomplete-source refusal, while the port refuses during source/goal
checking. No acceptance, source-hole refusal or provisional-proof policy changed.
These differences require separate review before any proposed policy change and
do not block independent CPU verification.

Single bounded probes identified the next coherent diagnostic family:

| Unqualified original | Actual pinned phase | Remaining diagnostic gap |
| --- | --- | --- |
| flatten/empty_match.bend | Checking | Non-inferrable empty matcher reports a generic host error in the port |
| flatten/pattern_shape_mismatch.bend | Checking | Missing True case lacks the structured expected/observed report and enclosing-match source |
| flatten/unsupported_pattern.bend | Parse/load | A lambda used as a pattern lacks the pinned pattern-grammar report and case source |

All three still reject with exit 1. The probes preserve exact streams and phase
observations but do not establish complete qualification or waive diagnostic gaps.
They are not added to detailed-coverage totals.

Source `77116c8ab4ab8eabbaea599679996941e41ed3b4` closes reconstructed matched-value and row-arity diagnostics for fourteen
unchanged originals, with fifteen authored controls. Per-use return provenance is
carried through generated match/lambda/let edges outside core terms. Constructor
checking and row-arity rejection keep their original failure decisions and budgets.
[Report](cpu-matched-values.md).

Full gate **920 passed, zero failed, 49 ignored**. The 37-case repeated matrix
contains 666 exact stream/status observations and 74 separate phase probes;
14 positive controls pass. All 1,302 checking statuses remain unchanged from
1175aa3. Three erased-hole/TODO disagreements remain unresolved and unwaived.
Next bounded probes expose empty-match inference, missing-case and unsupported-pattern
diagnostics; these probes are not counted as qualifications.

Detailed coverage: **245 originals / 1,057 without detailed receipts**: 397 checker
negatives, one checking-only module, 106 interpreter cases and 553 compiled
candidates. Twenty-two checking rows are current; 31 other checking, 99 interpretation
and 93 runtime rows retain historical identities. Linked receipts include unresolved
cells and do not establish universal parity. The incomplete 688-vector exploration,
host/resource/foreign/F32.read, runtime-error-order/API/lane requirements remain open.
External invoke consumers must still inspect exit_code; no external audit is claimed.
Frozen dependencies, references, toolchains, guards and demand policy; no installations,
pushes, desktop or GPU work. Independent CPU progress is not blocked on a decision.


## Reproduction, provenance and storage

Frozen tools: Rust 1.96.0, formatter nightly-2026-05-24, Node 24.19.0, PowerShell
7.5.4 and Clang 19.1.7. The upstream checkout remains unmodified; teamy-docs remains
at `50e89d6897ce2a65167ef65b17c5fa83fe849158`. Cargo.lock SHA-256 remains
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.

```sh
source <tools>/activate.sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked -p teamy-bend --test cli_checker_diagnostics --test cli_compile_executable --lib
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
python3 <evidence>/matrix.py --name matrix --binary <evidence>/final-snapshot/teamy-bend
python3 <evidence>/audit.py
python3 <evidence>/controls.py
```

Temporary Cargo debug/incremental overrides limit host build-cache growth;
repository build profiles and Bend runtime optimization policies are unchanged.
The saved binary has SHA-256 `0cfc01b7e113f57d68893c4136edf132a5827c885b148488311e13b3a04c98d4`.
Its recorded Cargo fingerprint uses default plus extended_observability, captured
after the standard all-feature build/provider passed, while non-Tracy tests ran.
Every formal comparison refers to that saved binary and the clean tested source.

Preflight requires 10 GiB workspace reserve. The gate started at
10.03 GiB and reached a minimum of 8.76 GiB; temporary storage stayed
above 4.38 GiB. Final workspace headroom is 10.02 GiB. The bounded
cache controller records intermediate insufficient-reserve results and reclaims
only inactive Cargo tests, libraries/metadata, build scripts and procedural macros;
no build starts below reserve. The final mandatory 10 GiB reserve passed; the
cache helpers' optional extra 32 MiB cushion was not reached. No source or unique
historical evidence was deleted.
Saved historical binaries dominate retained evidence; each new build must recheck
headroom. Actual byte counts, commands, profile settings and hashes are private
receipts, not source paths embedded in public documentation.

| Private receipt | SHA-256 |
| --- | --- |
| full-gate.log | `441e62b9e4a602df498446072824f2301311d0c136dc1b20a3abe59fbabb4e46` |
| matrix/receipt.json | `6f4243a5323975cee858c54dd1107949c0a6bdcbafa989617aff3e44e15e7f80` |
| controls.json | `2acddd5350b286389988b44bb7702c9cc0da040b18990b26653aeb364a3d38b7` |
| audit/receipt.json | `bc41b5b9e672fcfa71c8dc23a88be7ec8be39066e826f833aa27f9e566e89272` |
| original-hashes.json | `e0c40175039055df4939522cb06c325f12b90b597b267cef76c84d2f635658df` |
| next-gap.json (unqualified) | `37ca62e282a793fcb3935cee3cc954d4c170799c529da997683db4e89e92e98c` |

The [acceptance criterion](cpu-parity-acceptance.md#required-evidence-before-calling-cpu-parity-complete)
and existing external invocation-status/executable-emitter compatibility caveats
remain. No public API signature change or external consumer audit is introduced. Newly
classified CLI failures follow the existing `Ok(CliOutput)` with exit_code 1
contract. A renewed repository caller search confirms that src/lib.rs reads that
status after invoke; command dispatchers propagate the output. External callers
must inspect exit_code and cannot infer success from Ok alone.
