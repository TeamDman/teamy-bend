<!-- SPDX-License-Identifier: MPL-2.0 -->
# Nat-pattern depth and erased-binder diagnostic checkpoint

The unchanged `reg/nat_chain` and `reg/nat_table` originals remain blocked by the
64-level pattern loader. Measurement and independent review do not establish a
safe counter-only correction. No limit, representation, normalization, acceptance,
evaluation or demand-policy change is adopted. Compact-pattern research remains
unadopted. The independent erased-binder diagnostic gap is corrected.

Final clean tested source: `0aef62717e9587a21b7266c55e659ecdd6e65712`.
Diagnostic, Nat-boundary and census receipts use their actual binary source
`a1f8f4d528f741d2db1859689321e83f2d42e3fe`; production `src` is identical at both
commits. The final commit corrects two authored positive type annotations and adds
strict/native positive assertions. It does not redate A1 receipts.
Frozen upstream: `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.

## Depth finding and architectural boundary

| Unchanged fixture, definition `flag` | Loaded depth / tree nodes | Normalized depth / tree nodes |
| --- | ---: | ---: |
| `reg/nat_chain.bend` | 164 / 9,517 | 164 / 9,517 |
| `reg/nat_table.bend` | 160 / 8,700 | 159 / 8,631 |

Depth starts at zero and includes every semantic child. Node counts are tree
occurrences, including repeated shared nodes, not allocation counts. These are
upstream graphs; a failed port load yields no port graph. The
[measurement receipt](cpu-nat-pattern-metrics.json) retains deepest paths, tags,
canonical hashes and commands. Complete canonical strings, including both
annotation children and de Bruijn bound-variable identity, were compared before
hashing. Source metadata and binder display names are excluded; no cross-engine
pointer-identity claim is made.

Both originals contain a `122n` pattern. Pinned
[numeric parsing](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2210)
expands it into successor constructors, and
[flattening](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2679)
creates nested decisions. Result constructors add real depth. The pinned
[C chain/table optimization](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L2601)
runs after checking; it cannot solve the front-end guard by itself.

Eighteen controls vary literal size through 0, 1, 58, 59, 60, 61, 62, 122, 129
and result type Bool/U32. Eight load, check and normalize successfully, with
**16 complete loaded/normalized graph comparisons equal**. For this particular
single-pattern shape, Bool depth is N+2 with 4+3N nodes; U32 depth is N+35 with
134+68N nodes. Eight controls refuse at the 64-level pattern guard; two `129n`
controls refuse at the 128-value Nat-literal parser limit. All ten stop at load:
these are not observations of the kernel's input-depth guard.

The pattern traversal first refuses at N=60, even though the Bool core would
have depth 62. Its administrative traversal contributes overhead. Removing that
constant overhead would still leave the originals' actual cores above 128;
no accounting-policy change was adopted. Closing the originals requires a
coordinated compact typed-pattern/lowering and normalization design that also
preserves guard meaning, open-term structure, sharing and matching behavior.
That is an architectural decision/design task, not a demonstrated narrow fix.
No generated-tree exemption, guard reset, raised budget or evaluator substitution
was used. The initial private boundary runner only compared canonical strings when
present. Both the final evidence preparation and independent review explicitly
required both strings for all eight successful fixtures; future runner reuse must
retain that completeness assertion.

## Original and boundary execution evidence

[Exact command/result receipt](cpu-nat-pattern-depth-results.json):

- Unchanged originals: **54 observations**. Checking-only upstream accepts;
  upstream C O3 and generated JS each run twice, **eight successes**, producing
  `411210\n` and `305\n`. Direct upstream CLI separately fails in its IO host with
  `TypeError: require is not a function`; this is not checking rejection.
- Port strict/executable loaders, `check`, `eval`, `run` and all three executable
  emission targets refuse the originals at load. No port artifact exists; port
  generated execution is **unrun**, not a matching refusal or runtime pass.
- Six authored 59n-pattern programs test inputs 58/59/60 with Bool/U32 outputs.
  **60 successful five-lane runs**, twice each, plus 12 upstream interpreter,
  12 strict eval and 12 strict check observations, 24 generations, 12 C builds
  and 36 accepted-phase probes: **132 command observations plus 36 separate phase
  probes**. All executed
  results agree exactly and repeats are deterministic; this does not close the
  originals.

## Erased-binder diagnostic repair

The unchanged `parse/prefix_operator_dead.bend` body is `-5`. Pinned
[`parse_body_stmt`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2334)
treats leading `-` as an erased-let marker and demands a name.
Before: expected `a term`, observed `'-'`. After: expected `a name`, observed
`'5'`, with the same Bend source line as upstream. The compact
[pre-fix receipt](../tests/fixtures/cpu-erased-binder-diagnostics/before.json)
preserves nine cases, including two already-matching adjacent negatives.

`Parser::invalid_erased_name` records optional error context before restoring the
existing fallback cursor. Only the immediate expression failure receives this
context. Public parse errors, cursor behavior, successful terms, evaluation,
sharing and resource charges remain unchanged. The depth<64 condition prevents
this syntax context from replacing the expression-depth refusal. Independent
review caught that guard risk in the draft; the adopted version and a 63-binding
negative regression preserve it. No unary-minus feature is added.

Nine negatives cover Nat/parenthesis/EOF/multiline/imported/nested/argument and
missing-value forms: **162 exact status/stdout/stderr observations**, with **54
separate load-phase probes** across upstream and strict/executable port loading.
Six CLI entry points preserve exit 1 with no emitted executable. Successful
tracked/plain Books compare completely, and all four positive controls pass strict
normalization and native execution. No public API change in this patch; earlier
status-bearing CLI/emitter compatibility caveats remain, and external consumers
were not audited.

## Newly demonstrated independent C-emission defect

The positive reusable-binding control is deliberately retained unchanged:

```bend
import Base
def main() -> U32:
  +n = {3 : U32}
  (n + n : U32)
```

Strict evaluation, native, upstream interpreter/C/JS and port JS return `6`.
Port executable C generation exits 1 with
`resumable duplication requires a retained owner slot`; no C artifact is emitted.
The same failure is reproduced with baseline source `67b2035d525ea54b34c8d1ab4e1cf8a8f21e7b2f`,
so it is not introduced by diagnostic metadata. The failure is at the resumable
ownership check in `duplicate_owned`; the precise allocation/ownership cause is
not yet established. No emitter fix or exception is included.

The four-positive matrix contains **85 command observations and 24 phase probes**:
38 successful executable runs; eight interpreter, eight strict eval and eight
strict check observations; 15 successful generations and one failed C generation;
seven C builds. The missing C build and two C executions are explicitly unrun.
The required-all-lanes runner exits **1**, not success. All executed outputs match
and repeats are deterministic. This retained, small source is the next actionable
CPU defect: trace immediate reusable-value ownership/retained slots, then review
any correction against yielding, duplication, cleanup and aggregate regressions.

## Current checking census and acceptance scope

The complete 1,302-fixture census at A1 records **366 both-accept, 319 exact
rejections, 495 acceptance disagreements, 117 differing rejections and five
oracle failures**. All 385 historical IO exclusions renewed exactly before subset
subtraction. Current non-IO differences are **227 = 110 acceptance + 117 rejection**;
no observation is waived. [Current table](cpu-nat-pattern-census.csv) and
[reconciliation](cpu-nat-pattern-census.json) retain exact source/hash provenance.

Every port exit status is unchanged. The only port diagnostic change in the full
census is `prefix_operator_dead`, now exact. The prior 109 acceptance
[classifications](cpu-acceptance-priority.csv) remain unchanged. The additional
acceptance disagreement is `reg/dead_param_fork`: the previously stack-failing
upstream oracle accepted in this run. Five further fully captured probes yield
four acceptances and one host-stack failure. The historical six-failure cohort
therefore currently has five failures and one unstable acceptance, not a new
stable proof of port regression. This does not rehabilitate older sidecars lacking
streams or erase any historical failure.

Evidence links now total **445 linked / 857 unlinked**: 522 compiled candidates,
321 checker negatives, 13 interpreter profiles and one checking-only module
unlinked. Links and both-accept checking counts are not full CPU parity. Nat-first
C error ordering remains pending and distinct from the narrow approved wildcard
observations. Original string-pattern lowering, these Nat originals, F32.read,
source-hole/TODO, foreign/host/oracle/API/resource and supplementary all-target
Clippy gaps remain unwaived.

## Validation, reproduction and preservation

Standard gate at the final clean source: **1,187 passed / 0 failed / 49 ignored**. It includes previous
shared-reference, stack, aggregate work-accounting, guard and cancellation
regressions. This does not claim mid-normalization cancellation. Targeted strict
Clippy passes. [Validation receipt](cpu-nat-pattern-depth-validation.json) retains
commands, source/binary/log hashes, failed drafts and interrupted runs.

Use the frozen tool activation and build environment:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo test --locked --features extended_observability --test cpu_pattern_locations
cargo test --locked --features extended_observability --lib erased_binder_context
```

[Measurement instructions](../scripts/nat-pattern-depth/README.md) reproduce the
complete graphs. Exact individual matrix commands are in the result receipt:
`$REPO`/`$UPSTREAM` mean the frozen checkouts; `$A1_RUN`/`$FINAL_RUN` hold binaries
built from their named commits and disposable artifacts; `$EVIDENCE` holds the
matching phase/metrics probes; `$TEAMY_BEND_CC` is installed Clang 19. The
[checking-census instructions](../scripts/checking-census/README.md) reproduce the
full checking matrix. Command arrays use literal positional arguments; substitute
paths before execution. Each comparison child is bounded at 30 seconds.

Host `CARGO_PROFILE_*_DEBUG=0` and `CARGO_INCREMENTAL=0` control cache size; they do
not change Bend execution optimization. Primary upstream C uses `-O3`, matching
its CLI profile; port C additionally uses unchanged strict warning flags. Both
C lanes use `-lpthread -lm`, `--threads 1 --gpu off`. No packages, dependencies,
lockfiles, reference sources, toolchains, security policies or budgets changed.

Measured minimums were **11.345 GiB workspace / 4.788 GiB temporary**. Preflight requires 11 GiB workspace; stop floors stay 10 GiB
workspace / 1 GiB temporary. No cache deletion was needed. Compact expected
streams, source fixtures, complete-comparison hashes and commands are committed;
raw captures, large source-bearing graphs, generated programs and lossless binary
snapshots remain local and **are not preserved by Git publication**.

Two first-run matrix summaries were overwritten by their outer wrapper because
both used the same receipt filename. The wrapper/logs remain; both matrices were
renewed under distinct filenames with complete streams and actual A1 binary
identity. The published comparisons use those renewals. An early census-probe
bootstrap referred to a missing local file; the successful census uses the already
fingerprinted matching phase probe. The first gate failed Clippy function length;
helper extraction fixed it without suppression. A later task-owned gate was
terminated at exit 143 before correcting authored positive annotations. Those
runs are not passing final-source evidence. Final gate and positive matrix remain
separate: the passing gate does not clear the newly found C-generation defect.

Independent reviewers examined production changes, guard preservation and Nat
measurements; reviewers did not run the full gate. Publication is an isolated
reviewed checkpoint, not acceptance, main integration or a release.
