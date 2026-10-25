# Map/set and list/string executable qualification

Tested source: `6dc0d456e2a296bab513f8c067e5e70009d41e93`. The following checkpoint changes reports only.
**Both unchanged originals already execute correctly; this adds durable tests and
runtime evidence, with no production change and no claim of full CPU parity.**
Upstream remains `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.

## Scope and results

| Program | Before native | Current native / port C / port JS / upstream C / upstream JS |
| --- | --- | --- |
| original run/map_set_ops | `11111\n` | identical, all five lanes twice |
| original run/list_string_lookups | `1111\n` | identical, all five lanes twice |
| authored map edges | new coverage | 11 exact output lines, all five lanes twice |
| authored list/string edges | new coverage | 9 exact output lines, all five lanes twice |

Every qualified execution exits 0 with empty stderr. **40/40 runtime observations
match** across four programs. Twenty companion output lines are observations
within two programs, not twenty separately executed programs. Original fixtures
are byte-identical to the pin and retain Apache-2.0 attribution through NOTICE;
authored wrappers are MPL-2.0 and import those originals without editing them.

The [map/set original](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/run/map_set_ops.bend)
exercises bundled lookup, preservation of the returned map, deletion/pop,
right-biased union, prefix-key order and Set operations. The companion adds empty
versus single-NUL-character keys, replacement/deletion isolation, repeated pop,
Unicode ordering and duplicate Set elements. NUL stays internal; numeric lookup
results observe it without testing host NUL string output. The pinned
[Map.bit presence-bit encoding](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L2257)
explains why a string terminator and an actual zero code point must remain distinct.

The [list/string original](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/run/list_string_lookups.bend)
uses bundled consuming, Maybe-valued List.get and List.set. Its sfind, scontains,
ltrim, strim and seq functions are local fixture algorithms. Its header's claim
that bundled String.contains is absent is stale: both pinned and port Base define
it, but this fixture does not call it. The companion covers empty/out-of-range
reads and writes, empty needles, overlapping first matches, supplementary-Unicode
character indexing, and its local ASCII-whitespace rule (NBSP is retained).
These results do not qualify unrelated bundled string helpers.

No semantic divergence was demonstrated. Independent review strengthened one
new assertion: repeated pop now requires both Some{1} then None, avoiding a sum
whose compensating wrong values could pass. Upstream C/JS and native comparisons
were renewed after that correction. A separate early authored probe attempted
computed-value destructuring that both implementations reject identically; it was
factored through parameter helpers using the pinned rule. The rejected source
and diagnostic remain compact evidence. Neither preparation issue is a port fix.

## Validation and remaining differences

| Validation | Result |
| --- | --- |
| Clean-source standard gate | **1,163 passed / 0 failed / 49 ignored** |
| Focused originals, packed words, radix, sharing and collection regressions | 35 passed |
| Format, production strict Clippy, all-feature build, Node provider | Pass |
| Strict Clippy for the new integration test | Pass |
| Current matrix | 88 observations; 40 exact executions; zero timeouts |
| Separate checking phase probes | 24 observations, deterministic repeats |

The remaining matrix observations are 16 generations, eight C builds, eight
upstream direct-Node host failures and 16 strict check/eval refusals. Direct Node
still reports `TypeError: require is not a function`; these are attempted failures,
not unrun commands or qualified executions. Generated upstream JS succeeds on the
installed Node host for this family. All four inputs load, then port strict proof
checking refuses IO. Upstream and port executable checking accept them. These
intentional proof-scope exclusions are not production failures or new waivers.
No pure normalization or complete value-graph claim is made for these IO entries.

Full-gate controls retain earlier packed-word, sharing, default-host-stack,
false-proof, hole, work/depth, cancellation and output-order regressions. The 49
ignored hardware/profiler tests remain unrun. Historical supplementary all-target
Clippy failures were not rerun or cleared. No desktop/GPU work, production/dependency/
toolchain changes, budget increases, new semantic exceptions, merge or release.

The full 1,302-fixture checking census was **not rerun** for this tests-only change.
Its [last measured receipt](cpu-list-sort-map-census.json) remains attached to
`d224e79d33b4a0ab26f701b57f448056b63b790a`; production tree identity is verified in
[validation](cpu-map-set-lookups-validation.json). Last measured residuals remain
300 differences beyond 385 historical IO exclusions, plus six upstream stack
failures. The 302-row historical triage retains 151 diagnostic candidates / 109
acceptance / 42 other, with residual categories 149 / 109 / 42. Existing F32.read,
error-order, erased-hole/TODO, printer, host/foreign and resource disagreements,
external API caveats and incomplete 688-vector exploration remain unwaived.

Two new original runtime links give **416 linked / 886 unlinked** inventory rows:
531 interpreter-and-compiled candidates, 341 checker negatives, 13 interpreter
profiles and one checking-only module remain unlinked. Links are not completed
parity; [the acceptance criterion](cpu-parity-acceptance.md) still requires every
applicable lane and the separate language/library/effect/API inventory to close.
Next substantive batch: `base/string_kit` (direct bundled string helpers) and
`run/string_algorithms` (local compositions). Their IO applicability was inspected,
but executable behavior is unqualified here.

## Reproduction and preservation

Activate the provisioned environment, then run from the checkout:

```bash
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test cpu_map_set_lookups --test cpu_list_sort_map --test runtime_packed_words --test kernel_open_closures --test kernel_collections --test kernel_set_formatting
cargo clippy -p teamy-bend --locked --all-features --test cpu_map_set_lookups -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Host build-profile overrides control disk use, not Bend optimization or budgets.
The frozen Rust 1.96.0, formatter nightly-2026-05-24, Node 24.19 and Clang 19.1.7
remain in use. Set `fixture` to each original/positive file under
`tests/fixtures/cpu-map-set-lookups`, `reference` to the unchanged pinned checkout,
and `artifacts` to a fresh writable directory. For each input:

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

Matrix commands have 30-second process-group bounds and runtime observations
repeat twice; phase probes only load/check. Rust integration C tests retain their
separate strict unoptimized profile and 20-second process bound. Reusing a checked
book twice tests invocation isolation, not allocator reclamation. No new claim of
mid-normalization cancellation is made. Binary SHA-256:
`da198c85de69f80ebcab06786a61815d0cc3137c297d43d0c1bc38770a8d3094`. Frozen input hashes, exact gate/focused commands and
raw-log hashes are in [validation](cpu-map-set-lookups-validation.json).

Disk preflight started at 11.236 GiB and its monitored
minimum was 10.094 GiB, above the 10 GiB floor. Inactive
root-library/CLI build caches and completed test executables were reclaimed only
after checking they were unused. Final headroom: 11.259
GiB. All source, snapshots and unique historical evidence were retained.

[Per-case evidence](cpu-map-set-lookups-evidence.csv),
[compact failures and rejected probe](cpu-map-set-lookups-failures.json),
[validation](cpu-map-set-lookups-validation.json), originals and regression tests
are in Git. Bulk raw task logs, generated artifacts, phase output and binary
snapshots remain outside Git; pushing the checkpoint does not preserve those
private artifacts. Normal feature-branch push and remote-SHA verification are
separate from acceptance, merging or releasing.
