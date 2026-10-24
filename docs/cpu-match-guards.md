# Match-domain, constructor and erased-scrutinee diagnostics

Exact tested source: **`a241e18b5b262453397e6195733e790ed83ee244`**. Oracle:
[`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
This closes a diagnostic family, not full CPU parity.

## What changed and why

All [eight unchanged originals](../tests/fixtures/cpu-match-guards/README.md)
already rejected during checking. The port reported generic host errors for a
non-datatype domain, a wrong/repeated constructor, or an erased scrutinee used in
live code. The pinned
[match checker](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3517)
reports the normalized domain, remaining datatype family and constructor term,
or a message-only erased-scrutinee diagnostic. The port now attaches those facts
after the same failed checks. Adjacent authored controls also cover a match whose
goal is not a function, using the pinned non-inferrable-term form.

The private diagnostic's observed expression is optional, matching the pinned
[renderer](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1450).
Missing observed content renders `- message`, rather than inventing a value or
parsing a host error string. Formatting still uses a checker clone with only the
remaining allowance; failure retains the original host error. Context ordering,
stdout contracts, cancellation and checked stderr writes remain unchanged.

Constructor errors need a different location from domain/quantity errors. The
pinned [literal parser](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1803)
and [pattern flattener](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2751)
retain a separate case-head span. Private surface patterns now retain that source
identity, including the iterative nullary-constructor path. The external weak
provenance index propagates it through actual transformations. Only a wrong or
repeated constructor selects this span; other match failures keep the enclosing
match location. Nested, imported, templated and multiline cases now point to the
actual offending constructor.

No core Term field, binder ID, serialization, strong sharing or evaluation rule
changed. Regressions compare tracked/plain books and fuel/depth/fresh counters;
verify pointer sharing and provenance transfer; preserve first-error ordering and
budget exhaustion; and retain output files on compile failure. Three original
trailing spaces remain intact because fixture byte identity takes precedence over
whitespace cleanup. All copied files retain Apache attribution.

## Validation

| Evidence | Result |
| --- | --- |
| Focused precommit library/CLI checks | 260 passed, 0 failed, 1 ignored |
| Standard gate at tested source | 929 passed, 0 failed, 49 ignored; 98 targets |
| Format, strict Clippy, all-feature build, Node TCP/UDP provider | Passed |
| 8 originals + 16 authored + 9 historical negatives, twice | 660 observations: 594 exact streams/status and 66 phase probes |
| Phase probes | 62 checking, 4 parse/load; no program execution |
| C/JS/native executable generation | Exact refusal; no output artifact |
| Positive controls | 32 observations passed |
| Erased-hole/TODO controls | 18 observations preserve three unresolved disagreements |
| 1,302 checking-status audit | 439 expected negatives rejected; others 366 accepted / 497 rejected; zero transitions from d824174 |

All repetitions were deterministic, with no timeout or resource-floor stop. The
new positives retain erased matching within an erased region, live constructor
matching and legitimate live-empty-context elimination. Negative original
`efq_erased_binder_000` remains covered by repository regressions; an erased empty
binder does not make live code unreachable. Full-gate controls also preserve
source-hole refusal and the approved runtime demand contract.

The final gate took 316.85 seconds. Matrix children are bounded at 30
seconds, audit children at 60 and outer commands at 1,200. The ignored tests remain
unrun: 47 hardware and two optional profilers. The first clean-commit gate failed
strict Clippy's function-length check after adding provenance. Its receipt remains;
extracting the metadata transfer into a helper fixed the lint without suppression.
The focused receipt predates that helper-only refactor; the full gate tests the
exact source named above.

These negatives generate no C binary, so no successful runtime or C optimization
profile is inferred. Existing O3/runtime/normal-form receipts keep their actual
historical commits. Passing checking statuses across the inventory does not
qualify missing execution or exact-diagnostic lanes.

## Next substantive CPU batch and open requirements

Select **`flatten/ragged_rows_000..004`** next. They are five unqualified interpreter
rows, with overlapping defaults and rigid patterns. `_000` tests nested Nat
patterns; `_001` tests a dominant leading default; `_002` combines three columns;
`_003` recurses through a decreasing W/Nat pair; `_004` tests first-match priority
when a default row precedes a rigid row. The saved single-run probes find exact normalized text for all five originals,
but this is not yet repeated structural/application qualification. Their unmodified
C/JS build requests stop first because they lack `import Base`. The port instead
reports an unprintable function result. Separate Base-importing wrappers with
byte-identical helper copies reach function-printer refusal in both implementations.
All these generation requests exit 1 and create no artifact; their diagnostic text
differs and remains unresolved. Wrapper evidence is distinct from original evidence,
and neither gives these rows detailed qualification credit. See the
[applicability probe ledger](cpu-match-guards-next-evidence.csv).

A bounded follow-up should compare normalized core structure and text separately,
then apply concrete arguments. A useful initial grid is depths 0..2 for the two-
and three-column rows (81 vectors total across the five originals); add targeted
depth-3 boundary cases only where needed to distinguish overlapping branches.
For `_003`, vary W fuel and Nat depth independently. Verify constructor values,
not merely function printouts. Preserve originals; use checked wrappers or the
`CheckedBook::evaluate` with explicit arguments. Establish generated-runtime applicability
from the actual printer results before claiming C/JS/native execution coverage.
This is an independent semantic batch, not another diagnostic-wording exercise.

Detailed coverage is **282 / 1,302 originals**, with **1,020** lacking detailed
receipts: **360 checker negatives, 1 checking-only module, 106 interpreter cases
and 553 compiled candidates**. The primary checking ledger has 90 rows, 17 renewed
here and 73 historical; interpretation/runtime ledgers remain at 99/93 historical
rows. Source hints do not make every unlinked entry a missing CPU runtime test.
The [acceptance criterion](cpu-parity-acceptance.md) still requires complete lane
review and qualified results, not a passing test count or linked receipt alone.

[Erased-hole/TODO disagreements](cpu-match-guards-open-evidence.csv) remain
unwaived, as do the existing host/resource/foreign/F32.read/runtime-error-order
requirements and the incomplete 688-vector exploration. No new semantic exception
or source-hole policy change was made. The existing external API caveat remains:
classified failures can return `Ok(CliOutput)` with exit_code 1. Repository dispatch
propagates this and the entry point checks it; external consumers are unaudited.
The diagnostic field changes are private and introduce no new public signature.

## Commands, provenance and storage

The full gate used the existing activation and frozen host profile:

```sh
source "$TOOLS_ROOT/activate.sh"
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

The focused command was:

```sh
cargo test --locked -p teamy-bend --lib --test cli_checker_diagnostics --test cli_compile_executable
```
Host debug overrides are
not Bend runtime optimization profiles. The matrix executes pinned `node bend2/main.ts FIXTURE`, port JSON `check/eval/run`, pinned C/JS generation and port
executable C/JavaScript/native generation. Exact argv, fixture hashes, bytes,
statuses, durations and timeouts are in the retained private receipts. The phase
adapter calls pinned load/check functions separately. No direct-stream or phase
comparison substitutes for the other.

| Evidence | SHA-256 |
| --- | --- |
| Exact binary | `252d6db2dd141fda9ba257ed3997ef6539c1338585fccb052a83f01c2b07302e` |
| Lossless binary archive | `f716f0236e3a67d955edf0b9c7bea1953c15ffac8a898a030a4f015c473f87a6` |
| Matrix receipt | `ae728f196287cf7bd077aa5a07f4d62f5e24f7ed77932d73b3d37e8f352297de` |
| Checking-status audit | `8deeb38c78cb4e1c0349fd6a9f7ec64ec23f09ce086acf8adccc0e5f441e215a` |
| Positive/open controls | `d44b5074db54248779c461a5e13bcabbb443026633f112814b414c95da6d641f` |
| New positive controls | `7aeaf0825ac4c11481c784d1ff11a6eeaf7f85bcd2f060353be93fbb25766961` |
| Full gate receipt | `6fe4c916280b863caf404a51d7bab4f6aaa229e9a6d6d9b9d1fa301da5179c97` |
| Full gate log | `daa5d331af6dcf16c53f9c2b498601b1f560414a6277103580be2357f3b2679a` |
| Base-wrapper printer characterization only | `7ef747e8fe7035b01f6e62ef5c35355413df86554cbaf44f5f29e31c4cee87be` |
| Next interpreter characterization only | `2868ac7086d177f4a0dec0345d9c77ae00143001c0a6c5d5d56db6ed596c98fe` |

Build-start reserve was 10 GiB; full-gate minimum headroom was
8.745 GiB. Final headroom is 10.033 GiB.
Only inactive rebuildable Cargo files were reclaimed. All source and unique
historical evidence remain. The new exact binary is retained in temporary storage
and as a verified lossless gzip (7,030,232 bytes versus
26,963,816 raw); public records contain compact hashes, not machine
paths or raw logs. The initial final-cache controller fell about 2 MiB short
of the mandatory reserve after retaining the new receipts. After verification, only the inactive
Cargo main-binary hardlinks and two non-current hashed example-profile binaries
were retired; current example binaries remain. Both the 10 GiB reserve and the
optional 32 MiB cushion are met. The unsuccessful cleanup receipt is retained,
not relabeled as a pass. Reuse the preflight and compact-snapshot controls before
the next build.

Dependencies, lockfile, upstream/docs pins, toolchains, budgets, guards,
cancellation, approved demand policy and network policy remain frozen.
No installation, push, desktop or GPU work occurred.
