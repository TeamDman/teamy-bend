# Computed scrutinees and checker failure order

Tested source **`75085cb0a27dd8fb01f6bf50bb5e03198dddda80`** closes the computed-scrutinee diagnostics and the
three preceding unfinished-hole ordering discrepancies against Bend
**`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`**. Full CPU parity remains incomplete.
The approved runtime demand contract, evaluators, dependencies and safety limits
are unchanged.

## Computed scrutinee rejection

The unchanged originals `check/motive_suffix_join.bend` and
`check/computed_match_split.bend` reject during **parse/load**, before checking or
execution. The port already refused them but reported a generic flattener error
at a later parser position. It now preserves the computed scrutinee's identity
through the existing provenance map and emits the pinned structured message and
source excerpt. The public ParseError string/position remains unchanged.

The [pinned flattening rule](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2640)
permits matching a parameter or field and rejects a computed value. Its failure
uses the expression's source, which matters for multiline parentheses and imported
files. Nine authored cases cover calls, annotations, multiline expressions,
imports, CRLF, missing final newline, nested fields, destructuring and multiple
scrutinees. No Body/Term field, binder ID, sharing or evaluator change is needed.
[Original fixture attribution](../tests/fixtures/cpu-computed-scrutinee/README.md).

Constructor and bare-global scrutinees use different upstream branches. They are
not mislabeled as computed calls; their existing diagnostic gaps remain open.

## Why the three hole-order discrepancies were defects

The [application checker](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3350)
infers the function and rejects a non-function before checking its argument.
The [book checker](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3629)
reports the first error in declaration/checking order. The port's structural
preflight instead rejected any hole anywhere in the book first. Thus a U32
parameter used as `n(?unused)` or `n(0, ?unused)`, including the imported version,
reported the hole instead of the non-function. These now match exact stderr,
empty stdout and exit 1 across check/eval/run and executable emission.

Structural preflight still traverses the source before recursive checking with
the unchanged 128-level and 2,000,000-node limits and discovers all binder IDs.
It records the first source hole per declaration instead of returning it early.
Ordinary checking then follows its existing rule order. If a declaration would
otherwise succeed, its saved source-hole refusal fires **before a checked book
or executable book can be returned**. This preserves holes hidden by beta
reduction, erased arguments, unreferenced definitions and unreachable branches.
Public checked-argument inspection retains its immediate hole refusal.

Named holes reached normally now report the pinned goal or cannot-infer diagnostic,
context and actual source. The declaration boundary fills absent datatype names
without changing existing nested names. Twelve authored hole cases include the
three original ordering probes, competing holes, both declaration orders,
rewrite-evidence ordering, signatures, datatype fields and multiline use sites.
No ordinary runtime argument evaluation or IO ordering is changed.

## Validation and API behavior

| Check at the tested source | Result |
| --- | --- |
| Standard format, strict Clippy, all-feature build, Node provider and tests | **911 passed, 0 failed, 49 ignored**, 98 result-bearing targets; exit 0 |
| New originals and authored cases | 23 cases / 460 observations |
| Prior rewrite/token/local-binder/Nat-import controls | Five cases / 100 observations |
| Combined matrix | **560 observations: 504 exact stream/status results and 56 phase probes** |
| Phase probes | 26 parse/load and 30 checking; none starts execution |
| Positive worker/no-main proof controls | 14 observations pass, with scope below |
| Checking-status audit | All 1,302 statuses unchanged from abd6d22; zero timeouts/transitions |
| Expected checker negatives / other statuses | 439 rejected / 366 accepted and 497 rejected |

All comparison lanes repeat twice deterministically, reject with empty stdout and
exit 1, and finish within 30 seconds. Phase probes exit 0 and are counted separately.
C/JS/native emission refuses before creating output or invoking a C compiler;
C optimization profiles are therefore inapplicable to these failures. Existing
output files survive `--force` unchanged in integration tests.

The positive `check/computed_match_specialize.bend` worker checks and produces
exact `Zf{}` pure CLI output; its strict eval JSON reports the same value.
`proof/rewrite_type_family.bend` remains accepted with no main. These are focused
regression controls, not a new blanket runtime/core/artifact qualification.
The worker is not added to the detailed inventory union.

New adversarial tests retain the depth boundary, reject the next level and a
2,000,001-node input, discover IDs beyond a deferred hole, and retain original
rejection when diagnostic work is exhausted. Existing false-proof, demand,
collection, cancellation and writer-failure checks pass. The focused dirty
candidate passed **273 tests with one ignore**. An earlier authored test accessed
a private field and failed compilation; it was corrected to use public loaders.
Source `845db436b1977b29b7864a61112c944b0d247a3e` then failed strict Clippy for a
clone assignment. The final source uses `clone_into`; both failed receipts survive.

The established language failure boundary returns status-1 CliOutput. Repository
production dispatch reads `exit_code()` before emission; no new public signature
is introduced. **External invoke consumers remain unaudited** and must inspect
status rather than treating every `Ok` as success. More holes/load errors now use
that boundary, and some underlying hole messages gain declaration context.
String-consuming clients must account for these intentional diagnostic changes.

## Unresolved observations and contract boundary

Five authored cases remain explicitly unresolved in the
[open ledger](cpu-computed-scrutinee-open-evidence.csv), with 30 retained observations:

| Case | Pinned upstream | Port | Classification |
| --- | --- | --- | --- |
| Constructor literal scrutinee | Parse/load refusal naming an already destructed constructor | Same phase/status, generic host diagnostic | Diagnostic/source gap |
| Bare global scrutinee | Parse/load refusal naming an unbound/consumed/out-of-order variable | Same phase/status, generic host diagnostic | Diagnostic/source gap |
| Erased named hole | Pure evaluation succeeds with `0`, exit 0 | Source-hole refusal, exit 1 | Unresolved acceptance/safety-policy disagreement |
| Erased TODO | CLI refuses after checking due to incomplete source | Saved source-hole refusal after declaration checking | Phase/diagnostic difference |
| Direct TODO | CLI refuses after checking due to incomplete source | Goal checking refuses the hole | Phase/diagnostic difference |

The erased named-hole reproducer is:

```bend
import Base
def main() -> U32: (x => {0 : U32})(?unused)
```

The pinned [contract comment](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L228)
states that named holes fail checking. However, infer-app beta-reduces a direct
lambda before checking its argument; the [CLI completeness scan](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/main.ts#L432)
counts TODO/open entries, so the named hole above disappears. TODO instead checks
provisionally under [check-hol](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3584)
and the CLI rejects the incomplete book afterward.

The three original ordinary application-order defects did not require choosing
between these contracts. They are fixed while the port's existing whole-source
hole refusal remains intact. Relaxing that refusal would require a separate
contract review; no exception is adopted here. The approved executable demand
choices do not authorize a change to proof incompleteness policy.

## Coverage and next independent work

Detailed inventory coverage is **220 / 1,302 originals**, including historical and
unresolved cells; **1,082 lack detailed receipts**. Only the two unchanged original
negatives add inventory coverage. Authored probes do not inflate it.

| Applicability | Inventory | Detailed | Without detailed receipt |
| --- | ---: | ---: | ---: |
| checker_negative | 439 | 17 | 422 |
| checking_only | 12 | 11 | 1 |
| interpreter_only | 164 | 58 | 106 |
| compiled_candidate | 687 | 134 | 553 |

Eight of 28 primary checking rows carry this source. Twenty checking rows and all
99 interpretation/93 runtime rows retain their historical identities. Passing
the full gate does not renew every old detailed lane. The 688-vector exploration
remains incomplete. Existing host/resource/foreign/F32.read/runtime-error-order,
API/lane and unavailable-oracle requirements remain unwaived.

The next coherent independent family is the remaining scrutinee branches:
`flatten/error_window_substituted_scrutinee`, `flatten/consumed_column_error_001`,
and adjacent `flatten/order_ban_*`/`dup_guard_*` originals. Establish actual
applicability and preserve match-site versus variable-use provenance; these have
not been qualified by this batch. CPU progress does not require desktop or GPU.

## Reproduction and storage

After the existing activation script:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --lib --test cli_checker_diagnostics --test cli_compile_executable --test kernel_core --test executable_check
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
python3 <private-evidence>/matrix.py --name <fresh-directory> --binary <saved-cli>
python3 <private-evidence>/controls.py
python3 <private-evidence>/audit.py
python3 <private-evidence>/verify.py
```

The private cpu-computed-scrutinee evidence retains exact command arrays, immutable
source/reference/binary hashes, raw streams, statuses, timing, output absence and
all failed explorations. Fixed-output scripts require fresh destinations. One CLI
snapshot serves every final comparison; historical evidence is referenced without
duplicating ELF/generated executables. Hardware/profiler ignores remain unrun.

The 10-GiB pre-build reserve and 2/1-GiB workspace/tmp stop floors remain in force.
Cache preflights that could not restore the reserve stopped; inactive rebuildable
Cargo archives/metadata were reclaimed before proceeding. Preserved historical
ELF evidence remains the dominant storage cost (about 18 GiB). No source or unique
historical/failure evidence was deleted. Full-gate minimum headroom was
8.84 GiB; final headroom is 10.01 GiB.
The mandatory reserve was restored; the final archive-only cleanup could not
restore its optional extra 40-MiB cushion after exhausting inactive library
archives. Its nonzero receipt and actual free-space measurements are retained.
Other rebuildable build-script caches remain available for a later preflight.
New retained evidence is about 28.0 MiB. Host debug/incremental
overrides are storage controls, distinct from Bend runtime policies or C O3.
Outer commands remain bounded at twenty minutes; audit children at 60 seconds.

Rust 1.96.0, nightly-2026-05-24 formatting, Node 24.19.0, PowerShell 7.5.4 and
Clang 19.1.7 are unchanged. Upstream/docs trees remain clean at their frozen pins.
Cargo.lock SHA256 remains
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
No installations, pushes, desktop/GPU work or security-policy changes occurred.
Saved CLI SHA256: `82e5065cc91886ac230492b3810d75fe20733f04b7abf9a85a3afb90dcc4e02f`.

## Receipt hashes

| Receipt | SHA256 |
| --- | --- |
| matrix/receipt.json | `0fe2d8991874e1c38bf489d1b5f9cabfbf52a17c933bbc7b9864cd46d5c1d3e6` |
| controls.json | `38e070824bff75cc1243be6ff28809508e1bc01b599b1694e96f414697b18950` |
| audit/receipt.json | `03e51ce2e3ef02cbb3a81ac4913c4f973003fea6a36fc1d985ff499e9183eb71` |
| full-gate2.json | `baddebd2247f5aad588e39f1175e7a07c63042e3dd63d04479a920058a5bb87b` |
| full-gate2.log | `61a1ca3d2df4d51c6b7c7a0261a40e0b50ae0bf0933fb1d43859018f1fd0ef02` |
| focused2.json | `57f2c92eafe402b96e47a76851c6f7af21526b8fbb9ec272880b7276dba4d85b` |
| holes-initial.json | `c1d37323b86ebb9e2c44be6226d1ea4d0520ef2f4761d283da1aced4989498be` |
| initial.json | `560293a0fe30e1ecd19fe9f78482a417936fd9f8eb473230b89601fa5c5e9f60` |
| original-hashes.json | `3e4159ef7cb8fe57fd61205405d38999c0dc4c28f989f414556cb5063a974613` |
