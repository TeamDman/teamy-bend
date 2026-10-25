# Local string operations and bundled round trips

Tested source: `a6857164bbcd4fb6a0b3aecdac0344ba3c63583a`. The report checkpoint changes documentation only.
**The two originals already execute correctly. This adds durable tests and
runtime evidence, without a production change or a claim of full CPU parity.**
Upstream remains `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.

| Program | Before native | Current five executable lanes, twice each |
| --- | --- | --- |
| base/string_ops | `11\n` | exact |
| eval/string_roundtrip | `341\n` | exact |
| local edge companion | new coverage | 15 expected output lines, exact |
| bundled round-trip companion | new coverage | 13 expected output lines, exact |

All **40/40 executions** have exact complete stdout, empty stderr and exit 0:
native Rust, port C/JS and pinned upstream C/JS. The 28 edge output lines are
observations within two programs. Originals are byte-identical to the pin and
retain Apache-2.0 attribution through NOTICE. MPL-2.0 companions import them
unchanged; [sources and goldens](../tests/fixtures/cpu-string-ops-roundtrip/README.md)
are in Git. Independent review found no blocking issue.

## What the comparisons establish

The [string_ops original](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/base/string_ops.bend)
defines local length, search, reverse, slice, trim, upper, split and join helpers.
Its `strim` removes only U+0020. The companion confirms `strim(" \t ")` retains
the tab, whereas bundled `String.trim` returns the empty string. The distinction
comes from the local `x == 32` test versus bundled ASCII whitespace classification;
it is not a backend disagreement. Local search counts Unicode code points, returns
index 0 for an empty needle, and its observed 99 is a Maybe.default sentinel for
None, not an exhausted machine resource budget. Zero/oversized slices, supplementary
characters and ASCII-only uppercasing are covered.

The [round-trip original](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/eval/string_roundtrip.bend)
uses bundled String operations. Both originals have IO(Unit) mains despite the
round-trip fixture's “Pure” comment. The companions split `🙂é🙂🙂` on the
supplementary separator using local U32-code and bundled Char APIs: both yield
four fields, lengths `[0, 1, 0, 0]`, and alternate-separator join `|é||`.
Those observations constrain the split itself rather than relying only on a
round trip. Empty input, concatenation length and unequal/equal Unicode characters
are checked. Joining an empty list then splitting returns one empty field, so
no reverse `split(join(xs)) == xs` identity is asserted. Internal NUL controls
check round-trip preservation and length, not complete partition structure or
host printing of NUL. No new language policy was selected.

## Validation and residuals

| Validation | Result |
| --- | --- |
| Clean-source standard gate | **1,165 passed / 0 failed / 49 ignored** |
| Focused originals, strings, packed-word/radix, sharing and collection controls | **37 passed** |
| Strict Clippy for the new integration test | Pass |
| Main differential matrix | 88 observations, 40 exact executions, zero timeouts |
| Separate load/check probes | 24 observations with deterministic repeats |
| Separate unresolved pattern-gap reproduction | 14 observations, not counted as qualified parity |

The other main-matrix observations are 16 generations, eight C builds, eight
upstream direct-Node host failures and 16 strict check/eval refusals. Direct Node
fails with `TypeError: require is not a function`; generated JS executes correctly.
All four programs load, then strict proof checking refuses unavailable IO.
Upstream and port executable checking accept. These proof-scope exclusions do not
imply failed executable semantics, and execution does not imply strict-proof or
normalization support. [Compact before/current failure streams](cpu-string-ops-roundtrip-failures.json)
retain the distinction.

The full gate includes formatting, production strict Clippy, all-feature build,
Node provider and repository tests. Earlier packed-word, sharing, stack, work/depth,
source-hole, cancellation and output-order controls remain intact. The 49 ignored
hardware/profiler tests are unrun. Historical supplementary all-target Clippy
failures were not rerun or cleared. No dependency, toolchain, reference, safety-limit,
cancellation or approved-demand change was made; no desktop or GPU work occurred.

The full 1,302-fixture checking census was **not rerun**. Its [last measurement](cpu-list-sort-map-census.json)
remains tied to `d224e79d33b4a0ab26f701b57f448056b63b790a`, with identical production
tree verified: **300 differences beyond 385 historical IO exclusions, plus six
upstream stack failures**. The historical 302-row triage was 151 diagnostic
candidates / 109 acceptance / 42 other; residual categories remain 149 / 109 / 42.
These are retained measurements, not this family's result totals. F32.read,
failure-order, erased-hole/TODO, printer/oracle, host/foreign and resource gaps,
external API caveats and the incomplete 688-vector exploration remain unwaived.

Inventory links are now **420 linked / 882 unlinked**: 527 interpreter-and-compiled
candidates, 341 checker negatives, 13 interpreter-profile cases and one checking-only
module remain unlinked. Links do not satisfy [full acceptance](cpu-parity-acceptance.md).

## Next substantive gap: tiny literal string pattern

Independent census/source review selected the unchanged
[`parse/string_pattern_sugar.bend`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/string_pattern_sugar.bend),
a pure Bool program matching the two-character string `"hi"`. Renewed at this
checkpoint, upstream checking accepts and interpreter/C O3/JS each return
`True{}\n`, exit 0, empty stderr. Port strict and executable loading fail;
check/eval/run and both executable emitters exit 1 with the 64-level pattern
compilation refusal. No port C/JS artifact exists, so those executions are **unrun**,
not failed runtime comparisons. This case is already among the historical census
differences; it is not a newly added exception or an extra census row.

[The compact current reproducer and outcomes](cpu-string-ops-roundtrip-next-gap.json)
preserve the original source, hashes, statuses and diagnostics. Source inspection
suggests literal lowering through `SCon`/`Chr`/32-bit Word patterns and accumulated
pending match decisions contributes to depth. That is a mechanism hypothesis,
not proof of incorrect accounting. The next work should minimize empty/one/two
character patterns, mismatches and fallback, measure active versus pending depth,
and seek an implementation repair that retains the existing 65-Nat and 65-column
guard regressions. Do not raise the guard or replace structural matching with
an equality shortcut without establishing semantics. No correction is claimed here.

## Reproduction and evidence

After activating the provisioned environment, run from the checkout:

```bash
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test cpu_string_ops_roundtrip --test cpu_string_kit_algorithms --test cpu_map_set_lookups --test cpu_list_sort_map --test runtime_packed_words --test kernel_open_closures --test kernel_collections --test kernel_set_formatting
cargo clippy -p teamy-bend --locked --all-features --test cpu_string_ops_roundtrip -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Host debug/incremental overrides control cache size, not Bend runtime optimization.
Rust 1.96.0, formatter nightly-2026-05-24, Node 24.19 and Clang 19.1.7 stay frozen.
For each original/positive Bend file, set `fixture`, `reference` to the pinned
checkout and `artifacts` to a fresh directory. Repeat runtime commands twice:

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

For the separate gap, set `fixture` to the unchanged pinned
`tests/parse/string_pattern_sugar.bend`; port generation fails before artifacts,
so do not execute stale outputs. Matrix and gap subprocesses are bounded at 30
seconds. The integration harness retains strict unoptimized C compilation and
20-second generated-runtime execution bounds. Phase probes load/check only;
repeated checked-book invocations test isolation, not allocator reclamation.
No mid-normalization cancellation claim is made. Binary SHA-256:
`d66cc12c993a67d92ce0507af2854f65c1cc8326a8317fce27af893225669c97`. [Validation](cpu-string-ops-roundtrip-validation.json)
records exact commands, source/frozen-input hashes and raw receipt hashes.

The gate began with 11.168 GiB free and its minimum was
10.027 GiB, above the 10 GiB stop floor. Only inactive,
rebuildable caches were retired after their stages finished; final headroom is
11.211 GiB. Source and unique historical evidence remain.
Originals, tests, [per-case evidence](cpu-string-ops-roundtrip-evidence.csv), compact
failures and the unresolved gap are preserved in Git. Bulk raw logs, generated
artifacts, phase output and binary archives remain outside Git; publication does
not preserve those private artifacts. No main merge or release is performed.
