# Import-header diagnostic checkpoint

Tested source: `ec91a1634d410781623961607b9cf85aab2e5f87`; baseline checkpoint
`2767537996d53ebf05534518f24d0b44fc27724c` (tested source
`169934b2c9ceabfb27040d31442e6bbc838ebbd9`). Upstream remains
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`; teamy-docs remains
`50e89d6897ce2a65167ef65b17c5fa83fe849158`.

## Correction

The unchanged `reg/import_head`, `import/c_suffix_module`,
`import/parent_segments` and `import/suffix_refused` originals now have exact
pinned load diagnostics: malformed import syntax or a refused `.bend` suffix,
the authored observed token/line, and the original source window.
[Pinned loader](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1024).
These were diagnostic differences; all four already rejected at load with exit 1.
The parent_segments original contains no `..` traversal and refuses its suffix
before filesystem lookup.

A failure-only, non-inline helper borrows original source without registering it
in the first-write-wins source map. Successful imports therefore remain blanked
in subsequent parser/checker diagnostics. Alias rejection takes precedence over
suffix presentation. Public ParseError fields, message, Display, cursor and loader
signatures are unchanged. No terms, sharing, evaluation, charging, guards or
cancellation boundaries change; no mid-normalization cancellation is claimed.

Path normalization and hash/Unicode-whitespace tokenization differences remain
unqualified. Conservative fallback covers embedded/initial path hashes, ambiguous
dot components, backslashes, non-ASCII whitespace and U+FEFF. No new resolution
policy or semantic exception is adopted. A narrow Clippy expectation explains
why the pinned case-sensitive string suffix cannot use Clippy's suggested
case-insensitive host extension rule; other strict warnings remain enabled.

Optional private metadata/Debug and CLI formatting change. These newly formatted
failures use the existing `Ok(CliOutput)` with `exit_code = 1`, rather than the
old host-error fallback. Repository production dispatch still forwards this
status and `src/lib.rs` reads it before emitting output. External invoke
consumers remain unaudited; callers must not equate Ok with exit success.

## Validation

Standard gate: **1282 passed / 0 failed / 49 ignored**, 1049.358 seconds. It renews prior host-stack,
shared-reference, aggregate-work, partial-application, import/foreign, guard,
writer/flush and cancellation tests. The focused pre-checkpoint batch passed
69 integration tests; four final unit controls passed. Initial targeted Clippy
and a test-only field-access compilation error are retained as failed attempts;
the final standard gate is the acceptance result.

[Compact exact results](cpu-import-header-diagnostics-results.json.gz):

- 364 matrix commands, 173 deterministic **within-lane** repetition pairs, zero
  unexpected results. There are 234 exact negative CLI stream comparisons and
  52 separate negative port phase probes. Rejected-source execution is unrun.
- Three valid import programs produce `9`, `11`, `7`: 24 generated C/JS executions
  across upstream and port, 12 pure normalization observations, 24 checking/phase
  commands and 18 generation/build commands. Six emitted port artifacts are
  byte-identical to baseline. Native pure run is normalization, not an IO engine
  execution claim. Integration tests additionally compare complete normalized
  shapes against expected terms; the external matrix records text separately.
- 48 adjacent observations: all eight port controls remain byte-identical to
  baseline. The following parser/checker errors match upstream; six hash/Unicode,
  path-normalization or filesystem diagnostics remain non-exact and unwaived.
- 24 direct observations of the untouched upstream original paths, including
  eight separate load-phase probes. Fixture copies and committed CRLF/EOF bytes
  are verified against originals and goldens.

Every subprocess is bounded at 30 seconds in the differential runner; test
helpers use their existing bounds. No timeout occurred in completed comparisons.
C uses pinned `-std=c11 -O3 -lpthread -lm`; port C also uses
`-Wall -Wextra -Werror -pedantic`. No invented float or other tolerances.
The 436 completed observations are retained losslessly after replacing machine
roots with placeholders. [Validation](cpu-import-header-diagnostics-validation.json)
records exact source/binary/library/log/receipt hashes and commands.

Reproduce the repository controls after the existing tool activation:

```sh
export PYTHONDONTWRITEBYTECODE=1 CARGO_NET_OFFLINE=true BEND_GPU=off
export CARGO_BUILD_JOBS=4 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --offline --features extended_observability --lib import_header -- --nocapture
cargo test --locked --offline --features extended_observability --test cpu_import_header_diagnostics --test cpu_lambda_binder_diagnostics --test cpu_reserved_name_diagnostics --test cpu_template_occurrences --test kernel_core --test kernel_normalization --test kernel_open_closures
cargo clippy --locked --offline --features extended_observability --lib --test cpu_import_header_diagnostics -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Per-oracle commands are retained in the results receipt. The complete checking
census is reproducible with [the existing adapter](../scripts/checking-census/README.md).
Host debug/cache overrides do not change Bend runtime optimization or budgets.
Rust 1.96.0, formatter nightly-2026-05-24, Node 24.19.0, Clang 19.1.7 and PS7
remain unchanged. All dependencies, lockfiles and upstream references are frozen.

## Census and remaining scope

The renewed 1,302-fixture checking census measured 366 both accept, 396 exact
rejections, 494 acceptance disagreements, 40 different rejections and six oracle
failures. All 385 historical IO-cohort observations renewed exactly. Outside that
cohort, 149 measured differences remain: 109 acceptance + 40 rejection. Only the
four targeted port stderr streams changed; every port status and phase is
unchanged. Upstream reg/borrow_fork_hold changed from acceptance back to a host
stack failure. The fifth decrease from the previous 154 residuals is therefore
an unavailable observation, not another port improvement. No waiver is added.
[Reconciliation and compact before/after streams](cpu-import-header-diagnostics-census.json)
and [five-row census delta](cpu-import-header-diagnostics-census-delta.csv) preserve
the historical receipt identity.

Coverage linkage: 504 linked / 798 unlinked. Unlinked applicability: 265 checker rejection, 520 interpretation and compiled candidates, 1 checking without main, 12 interpretation only.
A linked diagnostic receipt is not executable qualification or universal parity.
CPU parity remains incomplete. Base coordinates, nat_first ordering, unadopted
compact Nat/string lowering, external invoke audit, historical supplementary
all-target lint failures, and other recorded host/foreign/resource/F32/oracle
limits remain open. Invalid-source execution, desktop/GPU and newer upstream
adoption are unrun. Next independent measured gap: the five missing-import source diagnostics
(cross_file_io, cross_file_proof, cycle_terminates, diamond_dedup, path_canonical).
Their current original load/check invocations stop at absent imported files; their comments
do not qualify actual cycle, diamond or cross-file semantics.

## Storage preservation

The user-authorized cleanup first verified all 402 then-local reachable commits
against remote refs (no stashes or untracked source). A private Git archive
verified all 409 recorded files, refs and reflogs. A separate private archive
verified all 50 pending-note/audit/Library members. No source or unique evidence
was deleted; installed tools and offline dependency sources remain available.

The first consolidation batch recovered **1,287,000,064 workspace bytes
(1.20 GiB)**, raising availability from 11,791,708,160 to 13,078,708,224 bytes.
It archived 117 byte-identical Node-provider copies after independently restoring
and hashing every one. One live copy, a verified gzip, per-path original metadata,
a journal and a restoration tool remain. No shared mutable hardlinks were
introduced. Historical archived paths need explicit restoration; bulk restoration
must have sufficient headroom, while selective restoration is supported.
Remaining duplicate candidates were not removed merely to maximize free space.

Earlier authorized Cargo cleanup reclaimed 2,574,958,592 bytes on the separate
`/tmp` filesystem and did not increase workspace capacity. The new consolidation
did: mount accounting is separate. Visible overlay file totals can include lower
image layers and must not be added to writable filesystem usage.

The unchanged 11 GiB preflight, 10 GiB active workspace floor and 1 GiB temporary
floor remain enforced. Gate minima: 12.179562 GiB workspace and 2.539 GiB
temporary storage. New bulk records stay outside Git; compact sanitized receipts
are published. The storage report/extension CSV remain saved and backed up
locally: Library upload failed before creation because its hosted-tools connection
returned HTTP403. No Library IDs exist. The private archives/raw logs are not
preserved by a feature-branch push and must accompany any environment transfer.
