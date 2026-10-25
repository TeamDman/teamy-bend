# String kit and local string algorithms

Tested source: `237c527d099da0e8af42642620b8d48159a7a5b1`. The following checkpoint changes reports only.
**Both unchanged originals already execute correctly. This adds regression tests
and executable evidence; no production defect or full CPU parity is claimed.**
The upstream pin remains `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.

| Program | Before native | Current five executable lanes, twice each |
| --- | --- | --- |
| base/string_kit | 36 expected output lines | exact |
| run/string_algorithms | `4\n` | exact |
| bundled edge companion | new coverage | 14 expected lines, exact |
| local algorithm companion | new coverage | 9 expected lines, exact |

All **40/40 executions** exit 0 with exact complete stdout and empty stderr:
native Rust, port C, port JS, upstream C and upstream JS. The 23 edge output lines
are observations within two programs, not 23 separate executions. The originals
are byte-identical to the pin, with Apache-2.0 attribution through NOTICE. The
MPL-2.0 companions import them unchanged. Goldens and executable assertions are in
[the fixture directory](../tests/fixtures/cpu-string-kit-algorithms/README.md).

## Contracts exercised

The [bundled kit original](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/base/string_kit.bend)
covers length, comparisons, search, slices, Maybe-valued indexing, list conversion,
join/split, lines, repetition, case, trimming and character classes. The companion
checks supplementary-Unicode indexing/reversal, empty and trailing split fields,
CR retention when splitting LF, zero repetition, internal NUL split/join, ASCII
case rules, non-ASCII alphabet classification and NBSP retention. Segment lengths
distinguish an empty list from a list containing an empty string. NUL remains
internal; this does not qualify host printing of NUL characters.

Pinned [Char helpers](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L1676)
use ASCII class/case rules. [String.split and lines](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L1904)
preserve empty fields and split LF without removing CR. Accordingly,
`length("🙂é")` is 2, index 1 is `é`, uppercase `aé🙂` is `Aé🙂`, and
`Char.is_space(Chr{160})` is False. These are precise pinned observations,
not a claim of Unicode case folding or whitespace normalization.

The [algorithm original](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/run/string_algorithms.bend)
defines local search, splice and occurrence helpers. Its occurrence search advances
one character after a match, so overlaps count. `occurs(2n, "aa", "aaa")` returns
99; fuel 3 returns 2 because a final failed search is needed to finish. A missing
needle returns 0 with fuel 1 but 99 with fuel 0. The tested empty-needle case also
returns 99. This is the fixture's explicit fuel sentinel, not a machine safety
budget failure. The indexed splice returns `🙂Xé` for the tested Unicode case and
appends `X` when its index is beyond `ab`. No new parsing or evaluation policy was
selected. Independent review checked these expectations and the complete
stdout/stderr/status assertions; runtime oracles then confirmed them.

## Validation and unresolved scope

| Check | Result |
| --- | --- |
| Clean-source standard gate | **1,164 passed / 0 failed / 49 ignored** |
| Focused family, collections, packed-word/radix and sharing controls | **36 passed** |
| New integration test strict Clippy | Pass |
| Differential matrix | 88 observations; 40 exact executions; zero timeouts |
| Separate load/check probes | 24 observations; deterministic repeats |

The other matrix observations are 16 successful generations, eight C builds,
eight upstream direct-Node failures and 16 port strict check/eval refusals. Direct
upstream Node still exits 1 with `TypeError: require is not a function`; generated
upstream JS succeeds on the installed Node host. All four programs load. Port
strict proof checking then refuses unavailable IO, while upstream and port
executable checking accept. Those scope exclusions are distinct from production
failures. These IO entries make no pure-normalization or complete value-graph claim.
Exact unique failure streams are retained in [compact evidence](cpu-string-kit-algorithms-failures.json).

The full gate includes format, production strict Clippy, all-feature build, Node
provider and repository tests, retaining earlier work/depth, packed-word, sharing,
default-host-stack, hole, cancellation and output-order regressions. Its 49 ignored
hardware/profiler tests remain unrun. Historical supplementary all-target Clippy
failures were not rerun or cleared. The initial authored test needed rustfmt line
wrapping; after correction the clean-source format check passed. No production,
dependency, toolchain, safety-budget or cancellation change was made.

The 1,302-fixture checking census was **not rerun** for this tests-only change.
The last [full measured census](cpu-list-sort-map-census.json) remains tied to
`d224e79d33b4a0ab26f701b57f448056b63b790a`; identical production tree is verified in
[validation](cpu-string-kit-algorithms-validation.json). Its residual count stays
**300 differences beyond 385 historical IO exclusions, plus six upstream stack
failures**. Historical 302-row triage was 151 diagnostic candidates / 109 acceptance
/ 42 other; residual categories are 149 / 109 / 42. Those are historical measured
sets, not this family's results. F32.read, failure order, erased-hole/TODO,
printer/oracle, host/foreign/resource disagreements, external API caveats and the
incomplete 688-vector exploration remain unwaived.

Two new runtime links give **418 linked / 884 unlinked** inventory rows: 529
interpreter-and-compiled candidates, 341 checker negatives, 13 interpreter-profile
cases and one checking-only module remain unlinked. Links are not completed parity;
[acceptance](cpu-parity-acceptance.md) still requires all applicable lanes plus the
separate language/library/effect/API inventory. Next bounded substantive candidates
are `base/string_ops` (fixture-local operations) and `eval/string_roundtrip`
(bundled split/join composition). Both have IO entry points despite the latter's
“Pure” comment. Their source/applicability was inspected, but executable behavior
is unqualified by this checkpoint.

## Reproduction and preservation

After activating the provisioned environment, run from the checkout:

```bash
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test cpu_string_kit_algorithms --test cpu_map_set_lookups --test cpu_list_sort_map --test runtime_packed_words --test kernel_open_closures --test kernel_collections --test kernel_set_formatting
cargo clippy -p teamy-bend --locked --all-features --test cpu_string_kit_algorithms -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

These host debug/incremental overrides control build-cache size, not Bend runtime
optimization. Frozen Rust 1.96.0, formatter nightly-2026-05-24, Node 24.19 and
Clang 19.1.7 remain in use. Set `fixture` to each original/positive Bend file under
`tests/fixtures/cpu-string-kit-algorithms`, `reference` to the unchanged pinned
checkout and `artifacts` to a fresh directory. Execute each runtime command twice:

```bash
teamy-bend run "$fixture"
teamy-bend check "$fixture"
teamy-bend eval "$fixture"
node "$reference/bend2/main.ts" "$fixture"
node "$reference/bend2/main.ts" "$fixture" -o "$artifacts/upstream.c"
node "$reference/bend2/main.ts" "$fixture" -o "$artifacts/upstream.js"
"$TEAMY_BEND_CC" -std=c11 -O3 "$artifacts/upstream.c" -lpthread -lm -o "$artifacts/upstream"
teamy-bend compile "$fixture" --executable --target c --output "$artifacts/port.c"
teamy-bend compile "$fixture" --executable --target javascript --output "$artifacts/port.js"
"$TEAMY_BEND_CC" -std=c11 -O3 -Wall -Wextra -Werror -pedantic "$artifacts/port.c" -pthread -lm -ldl -o "$artifacts/port"
"$artifacts/upstream" --threads 1 --gpu off
"$artifacts/port" --threads 1 --gpu off
node "$artifacts/upstream.js"
node "$artifacts/port.js"
```

The primary C oracle uses upstream's O3 profile. Matrix processes have 30-second
process-group bounds. Integration C tests use the existing strict unoptimized
profile and 20-second execution bound. Phase probes only load/check; repeated
checked-book invocations test isolation, not allocator reclamation. No claim of
mid-normalization cancellation is made. Binary SHA-256:
`657eec0a568320a255d716952be644dacee664d122c79aca08499ff93efaab67`. Exact tested hashes, commands and receipt hashes are in
[validation](cpu-string-kit-algorithms-validation.json).

Disk preflight started at 11.202 GiB; monitored minimum was
10.060 GiB, above the 10 GiB floor. Only confirmed inactive rebuildable
unit-test/alternate CLI caches were reclaimed after those stages finished, and
remaining completed test caches after the gate; final headroom was
11.234 GiB. Source and unique historical evidence remain.
The capture checks the previous CLI archive for identical bytes before retaining
a new compressed snapshot; this build has a different binary hash.

Originals, goldens, regression test, [per-case receipt](cpu-string-kit-algorithms-evidence.csv),
compact failures and validation are in Git. Raw task logs, generated artifacts,
phase output and binary snapshots remain outside Git; checkpoint pushes do not
preserve those private artifacts. Normal feature-branch publication is separate
from acceptance, main merging or releasing.
