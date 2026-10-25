# One-name lambda-binder diagnostics

Tested source: `169934b2c9ceabfb27040d31442e6bbc838ebbd9`; baseline checkpoint
`a54e49f48746cdba43a0a3cfbc8558c83d900c1a`. Upstream remains
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, documentation
`50e89d6897ce2a65167ef65b17c5fa83fe849158`.

## Correction and contract

The unchanged `parse/reserved_lambda_binder.bend` now matches upstream's
`a lambda binder (one name: k => body)` diagnostic, raw following space and
source window. Pinned [parse_bind](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1679)
fails immediately after [consuming `=>`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2100), before body parsing or whitespace skipping.

The port attaches optional source metadata only to its existing non-Ref/non-Var
refusal. It borrows the consumed arrow's own line/column, including an arrow on
a later line. A failure-only non-inline helper keeps formatting temporaries out
of the recursive expression frame. Renaming the existing raw-position helper to
`expected_after_token` changes no bounds logic. Public ParseError fields, message,
Display, cursor, fresh IDs and loader signatures are unchanged. Accepted-name
paths and the earlier template-binding refusal are unchanged.

Optional metadata and derived Debug can change. Newly formatted CLI failures use
the existing Ok(CliOutput)/exit_code1 boundary; external invoke consumers remain
unaudited. No new public field, semantic exception, termination rule, stack size,
resource budget, dependency or toolchain change. Sharing and cancellation
boundaries remain unchanged; no mid-normalization cancellation is claimed.

## Validation and applicability

Full gate: **1276 passed / 0 failed / 49 ignored**, 1037.956 seconds. Focused controls: **70 passed**;
three integration tests were renewed after replacing two lint-rejected
`assert!(Result.is_err())` calls with exact error assertions. Final targeted
strict Clippy passed. The initial lint failure is retained; no warning suppression.
The full gate renews previous parser/host-stack, sharing, aggregate-work,
closed-partial proof, writer/flush, cancellation and guard regressions.

The [lossless JSON receipt](cpu-lambda-binder-diagnostics-results.json.gz) retains
every command, repetition, phase, status, output and raw-stream hash:

- Main matrix: **456 commands**, 216 deterministic pairs, 288 exact negative
  stream comparisons, 64 separate negative phase probes, 32 generated C/JS
  executions and 16 pure normalizations. Eight emitted port artifacts are
  byte-identical to baseline; baseline emission alone is not execution evidence.
- Adjacent matrix: **58 commands**, 29 deterministic pairs. All three control
  port streams are unchanged. Template-binding and earlier-depth diagnostics
  remain non-exact; the earlier-syntax control already matches exactly.
- The unchanged uppercase-binder original normalizes to `Qv2{}` in six observations
  and has ten checking/phase observations. All twelve C/JS emission attempts
  correctly refuse missing Base. Its generated execution remains unrun.
- Four separate Base-importing positives produce `7`, `6`, `13`, `8` across
  generated C/JS. Complete normalized shapes and text are checked independently.

All sixteen negative cases reject before artifact generation; invalid-source
execution remains unrun. They cover scalar/application/constructor binders,
parentheses, multiline arrows, comments, tab/LF/CRLF/EOF, imports and malformed
body priority. The template negative covers declaration-site provenance, not
helper execution during specialization replay. Existing template replay tests
remain separate regressions. Two upstream originals and raw goldens are preserved
byte-for-byte; see [fixtures and compact pre-fix evidence](../tests/fixtures/cpu-lambda-binder-diagnostics/README.md).

The receipt is 18,122 bytes of gzip-compressed JSON, not an executable. All 514
observations remain present after decompression; only machine-root paths are
replaced. Independent review checked every row and both compressed/uncompressed
hashes. Read it using existing Python, without adding a project dependency:

```sh
python3 -c 'import gzip,json,sys; print(json.loads(gzip.decompress(open(sys.argv[1],"rb").read()))["matrix_counts"])' docs/cpu-lambda-binder-diagnostics-results.json.gz
```

Exact gate and focused commands (existing activation first):

```sh
export PYTHONDONTWRITEBYTECODE=1 CARGO_NET_OFFLINE=true BEND_GPU=off
export CARGO_BUILD_JOBS=4 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --lib lambda_binder -- --nocapture
cargo test --locked --features extended_observability --test cpu_lambda_binder_diagnostics --test cpu_reserved_name_diagnostics --test cpu_template_occurrences --test kernel_core --test kernel_normalization --test kernel_open_closures
cargo test --locked --features extended_observability --test cpu_lambda_binder_diagnostics
cargo clippy --locked --features extended_observability --lib --test cpu_lambda_binder_diagnostics -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Rust 1.96.0, formatter nightly-2026-05-24, Node 24.19.0, Clang 19.1.7 and
PowerShell 7.5.4 stay frozen. Host debug/cache overrides do not change Bend
semantics or C optimization. C comparisons use the pinned `-std=c11 -O3` profile;
port C additionally passes `-Wall -Wextra -Werror -pedantic`. Exact per-lane
commands are in the compressed receipt; census invocation and log hashes are in
[validation](cpu-lambda-binder-diagnostics-validation.json).

Initial authored declarations/imports hit earlier grammar refusals and were
corrected to pinned syntax. A premature executable classification of the
uppercase original was corrected after its truthful missing-Base refusal. The
four partial preparation runs and initial lint failure remain local evidence;
none is counted as a completed matrix or a production semantic defect.

## Storage and preservation

The 11 GiB build preflight was restored by removing 190 independently reviewed
Python bytecode caches and their empty directories: **2,994,176 bytes reclaimed**.
Every Python source hash remained unchanged. Source, receipts, archives, unique
failure evidence, toolchains and credentials were preserved. New bulk evidence
uses temporary storage; bytecode generation is disabled only for these processes.

Normal loose-object checkpoint creation would consume the remaining preflight
margin. Independently reviewed `git -c fastimport.unpackLimit=0 fast-import
--quiet --done` instead wrote new objects directly packed, creating one ordinary
child commit on the isolated feature branch. Parent, exact file bytes/modes/tree,
index and other refs were verified; old packs stayed byte-identical. No force,
reset, pruning, old-object repacking, persistent configuration change or required
hook bypass occurred. The index-lock allocation can transiently cross the
pre-build reserve; the distinct 10 GiB active floor stays unchanged.

Settled source-checkpoint space was 11 GiB plus 36,864 bytes and passed the next
build preflight. Gate minima: **11.000034 GiB workspace / 2.772 GiB temporary
storage**. After the gate exited and no PowerShell process remained, removal of only its
disposable startup performance profile reclaimed another **98,304 bytes**; all
cache siblings, including telemetry, stayed unchanged. This is separately recorded. Every subsequent build still requires fresh preflight;
this does not solve long-term storage capacity.

## Current parity census and next work

The 1,302-fixture census measured **366 both accept, 392 exact rejections,
495 acceptance disagreements, 44 different rejections and five current oracle
failures**. The historical 385-member IO cohort is exactly renewed. Outside it,
**154 residuals = 110 acceptance disagreements + 44 rejection differences**.

Only the targeted port stderr stream changed; all other 1,301 port streams and
all port statuses/phases are unchanged. Upstream `reg/borrow_fork_hold` alone
changed from its historical host-stack failure to acceptance. Its port refusal
is unchanged. The arithmetic is previous 154 minus one diagnostic fix plus one
newly measurable oracle-dependent mismatch. This is neither a port regression
nor an additional exception. Both transitions and exact streams appear in the
[census receipt](cpu-lambda-binder-diagnostics-census.json) and canonical delta.
The five current oracle failures and the historically unstable sixth case remain
unqualified. The 386th missing-IO symptom does not enlarge historical exclusions.

Receipt linkage: **500 linked / 802 unlinked**. Unlinked categories: 269 checker rejection, 520 interpretation and compiled candidates, 1 checking without main, 12 interpretation only. Linked
partial/blocked receipts do not establish universal parity; both-accept checking
status is not executable parity. The existing acceptance criterion remains.

Next bounded demonstrated gap: import-head/suffix diagnostics shared by
`reg/import_head`, `import/c_suffix_module`, `import/parent_segments` and
`import/suffix_refused`. These are measured load refusals, not runtime import
qualification. Missing-file import fixtures remain distinct environmental cases.

External invoke audit, Base coordinates, pending nat_first ordering, unadopted
compact Nat/string lowering, prior resource/foreign/IO/source-hole/F32 and oracle
issues remain open. Supplementary all-target Clippy was not rerun; its historical
failures remain outstanding. No desktop/GPU, installs, main merge or release.
Compact evidence is in Git; bulk logs, binary snapshots, phase sidecars and initial
preparation failures remain local outside Git preservation.
