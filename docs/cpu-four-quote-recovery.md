# Bounded four-quote diagnostic recovery

Tested source: `02155ff3c30c6bcab683abb1fd8d0e73422acb1d`.
Standard gate: **1350 passed / 0 failed / 49 ignored**,
197 groups, 1061.465 seconds. Formatting, strict standard Clippy,
build and real TCP/UDP provider checks passed. Final source remained clean.
Focused validation passed 102 syntax tests and eight CLI/runtime/template tests,
including default-stack guards, stale metadata, prefix caps and shared-template identity.

The final-source matrix has **712 observations / 114 repeat pairs / zero timeouts**.
Adjacent controls improved from 3 exact/5 different to 5 exact/3 different.
The post-gate CLI hash equals the matrix snapshot. [Validation receipt](cpu-four-quote-recovery-validation.json)
and [compact streams and runners](cpu-four-quote-recovery-results.json.gz) preserve commands, hashes,
statuses and failure classifications. Raw matrix SHA-256: `7a23b123dc40baef0d66cd37cbec3d7175a4f80656866dc1c674ed22a2309702`.
Full-gate log SHA-256: `234f4c6925b5a65a9addb5569d4fa9f2abcc2da0b959287900450569bb878819`.
Gate minima: 12.161 GiB workspace / 1.498 GiB temporary.

The original four-quotes fixture is unchanged and now reports pinned upstream's
declaration-context error. The original 22-negative cohort improves from 21 exact
and one different to **22 exact and zero different**. Twelve preceding grammar
controls remain exact. The eight new neighboring contexts improve from three
exact/five different to **five exact/three different**; two of these closures
share the same mechanism as the original and are not separate semantic repairs.

Pinned [Char parsing](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1930)
consumes three apostrophes as code 39. A fourth quote is then observed by the
surrounding grammar. Top-level declarations expect a declaration keyword;
grouped expressions expect `)`. An error-string substitution at the legacy empty
Char failure cannot distinguish these contexts. Earlier malformed escapes,
grammar errors and duplicate declarations retain their priority.

## Recovery contract

The real parser retains its historical token stream and exact public error.
Only after it rejects can a captured corrected prefix produce optional metadata.
An initial-state snapshot preserves declaration order, imports, aliases,
template records and the fresh counter. It shares immutable term/token allocations
without traversing their graphs. Diagnostic origins are fresh; their mutable
tracker is never cloned or shared. Only a new diagnostic from a supported failure
can escape; no scratch book, term, fresh ID or template-cache mutation can escape.

The existing prefix ceilings remain 64 tokens and 4096 bytes. This new optional
recovery also caps the complete cleaned and original display sources at 4096 bytes,
and preflights at most 4096 copied header items and 64 KiB copied strings/path bytes.
These are additional presentation ceilings, not increased language/evaluation
budgets. Preflight stops before cloning on exhaustion. Source size is bounded
because excerpt rendering examines the whole source. Existing barriers stop
template expansion, imported-law traversal, compile-time substitution and
nontrivial lowering. `foreign=None` refuses foreign definitions before host-path
inspection. Unsupported or over-cap recovery keeps the primary parser result.

Regression coverage includes exact raw fields in strict/executable loading,
original and neighboring grammar contexts, earlier errors, actual book/template
copy-cap refusals, oversize display/source tails, isolated tracker/cache mutations,
shared node/token identity, and parser/import depth on the default test stack.
The existing 64-level parser/import guards and 128-character literal guards remain.
No semantic exception or new cancellation capability was introduced.

## Evidence and remaining differences

All 42 negative inputs reject at load. Checking, eval/run and executable C/JS/native
emission preserve status1, empty stdout and absent output artifacts; 84 negative
public-API results match baseline byte-for-byte. Ten positive raw-load observations
also match. Five positive controls retain four pure native outputs and one actual
native IO output. Strict IO proof exclusion is explicit and distinct from execution.
Native text is compared with the prior port binary. The four upstream pure
normalization rows establish successful exit only, not equality of value graphs.
Forty generated C/JS executions supply 20 exact corresponding-backend comparisons.
Ten port-generated artifacts remain byte-identical to baseline. C uses upstream's
O3 profile; port C additionally passes strict warnings. No new universal graph or
cross-backend contract is inferred from these controls.

Three adjacent cells remain **different and unwaived**. Argument/list contexts
reach an incomplete literal atom, where existing recovery deliberately refuses to
decode. The depth control retains the port's 64-level refusal rather than the
upstream syntax diagnostic. A future bounded task may investigate the two literal
metadata cases; it must preserve the barrier and raw error, not decode the fault.
The original requested four-quotes gap is closed; full CPU parity is incomplete.

Whole-census and older overlapping receipts were not renewed. The historical
140 non-IO differences at c5edea8 (109 acceptance/31 rejection), six upstream stack
failures and 516/1302 linked originals are not newly measured totals or universal
parity. `nat_first`, compact Nat/string lowering, Base coordinates, F32.read,
host/oracle/foreign/Bun/source-hole/resource limits and supplementary all-target
Clippy failures remain open. Standard Clippy is not the supplementary all-target
lane. External `invoke` consumers remain unaudited: newly formatted failures use
the existing `Ok(exit_code=1)` boundary, while raw loader `Err` fields are unchanged.

A new test initially failed to compile due to an overlapping mutable borrow;
scoping the test borrow fixed it. The first expanded fixture checks failed because
two reused sources have goldens in their owning manifest's directory. Independent
review caught this test lookup mistake; lookup now uses the owning cohort.
Both failed runs and renewed successes remain preserved, without weakening checks.

## Reproduction and preservation

Frozen upstream: `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.
Baseline checkpoint: `02680fd234b041a3935b385e862fa34d300c6fa8`, with unchanged
production source from `b383255b9b584408cb61927e7cd4b34567d78f99`.
Rust/Cargo 1.96.0, formatter nightly-2026-05-24, Node 24.19.0, Clang 19.1.7,
PowerShell 7.5.4 and dependencies remain frozen. Host profile overrides are
`CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true BEND_GPU=off`; they are separate from
the generated Bend C O3 runtime profile.

```sh
cargo test --locked --offline --features extended_observability --lib syntax::
cargo test --locked --offline --features extended_observability --test cpu_empty_character_context --test cpu_raw_apostrophe --test cpu_template_occurrences
cargo build --locked --offline --features extended_observability
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

The compact evidence includes bounded runners and the public-error probe.
Differential children have 30-second timeouts; the full gate has 1200 seconds.
Disk floors remain 11 GiB prebuild / 10 GiB active workspace and 1 GiB temporary.
Source bundles, unique raw failures and validation receipts are backed up locally.
Large binaries remain local with hashes; Git contains compact reproducible evidence.
No cleanup, dependency/toolchain change, desktop/GPU use or release was needed.
Blocked Library uploads were not retried. After independent final evidence review,
publication uses the standing authorized non-force feature/main workflow; remote
identities are verified and recorded separately.
