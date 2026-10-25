# Decimal read bounds: originals, controls and retained failures

Tested clean source: `7305b890a201ead43dd3029a6e8e40c9aba22409`. No production change was adopted. The unchanged
`base/read_bounds` original matches native Rust, port C/JS and upstream C, while
upstream JS fails in its unavailable file-error host. A new combined word-reader
control exposes native cumulative work exhaustion and remains **unresolved**.
The compact string-pattern design remains [unadopted research](cpu-compact-patterns.md).

| Program | Native | Port C/JS | Upstream C | Upstream JS |
| --- | --- | --- | --- | --- |
| Original base/read_bounds | exact 11 lines | exact | exact | first 10 lines, then bun:ffi failure |
| Original base/show_read | `5\n` | exact | exact | exact |
| Nat boundary companion | exact 16 lines | exact | exact | exact |
| Helper companion | exact 16 lines | exact | exact | exact |
| Empty-path/mode companion | exact 4 lines | exact | exact | bun:ffi failure before output |
| Full 16-case word companion | budget refusal after five lines | exact 16 lines | exact | exact |

Every executable observation is repeated twice with exact stdout, stderr and exit
status, under a 30-second process bound. The main matrix has **132 observations**:
**54 exact executions**, four upstream-JS host failures, two native budget
refusals, 12 direct-Node host failures, 24 strict check/eval refusals, 24 generations
and 12 C builds. There are zero timeouts. All requested artifacts are generated;
the failing executable lanes actually ran and are not labelled unrun.

Originals are byte-identical to upstream
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, with Apache-2.0 attribution retained.
MPL-2.0 companions import their original formatters. [Fixtures and required stdout](../tests/fixtures/cpu-read-bounds/README.md),
[per-case receipt links](cpu-read-bounds-evidence.csv), and
[compact before/current failures](cpu-read-bounds-failures.json) are in Git.
Preliminary runs at checkout HEAD 456cff5 were not a fresh clean-binary build;
the authoritative repeated results above use the clean source identified here.

## Pinned contracts and observed boundaries

[Nat.read](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L1991)
accepts nonempty ASCII decimal through **281474976710655**. It checks that the
accumulator can accept the next digit before multiplication/addition. Empty,
malformed or overflowing input returns `None`. Leading zeroes do not shrink the
numeric range. The companion covers leading-zero maxima, max-minus-one, max-plus-one,
signs, spaces, tab, Unicode characters/digits, NUL suffix and hexadecimal-looking text.

[U32.read](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L2058)
accepts through **4294967295**. Its quotient check after wrapping arithmetic is
equivalent to an ASCII decimal digit and accumulator below 429496729, or equality
with digit at most 5. The port's explicit precheck preserves this value contract;
the numerical comparisons show no wrong value. The word companion has analogous
malformed and leading-zero/max boundaries plus an extra digit after the maximum.

The `.go("", acc)` helpers return `Some{acc}`, unlike public `read("")`.
The `.if` helpers trust their condition. The helper companion exercises final
digits 4/5/6, an excessive accumulator, ordinary 123, and false conditions whose
unselected arithmetic would overflow. It does not authorize eager evaluation of
that branch. Large helper inputs are expressed using ordinary Base arithmetic.
An earlier direct-Nat-literal version refused the existing 128 literal cap; its
exact source/error is retained as a failed experiment, not erased or promoted to
runtime success. The parser's 129n tests remain intact. This distinction does not
make the original numeric runtime range smaller.

The original's final `File.open("", "r")` returns Linux ENOENT, exactly
`2 No such file or directory\n`. The companion renews modes r/w/a and invalid r+:
the latter returns `22 Invalid argument\n` before trying the missing path.
These empty paths cannot open or truncate an existing file. Existing embedded-NUL,
ordering and cleanup tests remain under the full gate; this companion does not
claim new cross-platform errno qualification.

Pinned upstream generated JS prints all ten numeric lines of `read_bounds`, then
fails while importing `bun:ffi` to format the file error. This is an oracle-host
failure, not a complete match or an unexecuted program. Numeric-only companions
are separate programs and do not close that original lane. Direct upstream Node
fails earlier with `TypeError: require is not a function`. No Bun installation or
host substitution was attempted.

## Next substantive gap: cumulative native word-reading work

The complete unresolved program and its full expected output are retained in
`unresolved/word-edges.bend`; it is excluded explicitly from the passing integration
test, and remains a failing differential cell. Splitting it is diagnostic evidence,
not a repair. [Native isolation receipts](cpu-read-bounds-isolation.json) renew:

- All 16 individual cases pass, twice each.
- Prefixes 4 and 5 pass; prefixes 6 and 16 fail after the same first five lines.
- Total: 40 native-only observations, 36 exact successes and 4 budget refusals.
- Port C/JS and upstream C/JS complete the full aggregate twice each.

The unchanged native limit is 2,000,000 steps per invocation. This proves a
cumulative resource gap; it does **not** establish faulty accounting or repeated
reduction of the same thunk. Independent source review identifies a concrete
hypothesis: [U32.read](https://github.com/TeamDman/teamy-bend/blob/7305b890a201ead43dd3029a6e8e40c9aba22409/src/syntax/base.bend#L2321)
computes each digit with `U32.sub`, which lacks a sealed numeric fast path.
Its ordinary U32 constructor result causes subsequent additions to use the
[ordinary fallback](https://github.com/TeamDman/teamy-bend/blob/7305b890a201ead43dd3029a6e8e40c9aba22409/src/runtime/numeric.rs#L333),
and accumulator arithmetic can remain structural. That fallback preserves lazy
Word fields and must not be removed merely to fit this workload.

Next measure packed-versus-fallback calls and separate evaluator, environment
lookup and GC charges at IO boundaries for individual/prefix/aggregate forms.
If that confirms representation loss, investigate a sealed packed-only U32.sub
optimization with ordinary fallback. Preserve first-error ordering, lazy fields,
source identity, cumulative budget and cancellation. No such optimization,
guard increase, accounting exemption or semantic waiver is adopted here.

## Gates, census and applicability

The clean-source standard gate passes **1,170 / 0 / 49**. Targeted strict Clippy for
the new integration test passes. The preliminary focused positive controls and
existing numeric suite passed 7 tests, including false-proof rejection and the
unsealed large-Nat representation refusal. Two earlier focused attempts failed:
the literal-cap experiment, then the full word aggregate. Both receipts remain;
neither is relabelled a pass. The standard gate includes the existing parser,
packed-word, sharing/aggregate, stack, work/depth, source-hole and cancellation
regressions. The 49 ignored hardware/profiler cases remain unrun, and historical
supplementary all-target Clippy failures are not cleared.

There are **36 separate load/check phase observations**. Upstream and port
executable checking accept all six programs. Strict proof checking rejects
unavailable IO or File declarations. Exact observed-symbol diagnostics are
preserved; phase agreement is not diagnostic or runtime parity. The raw matrix
label `strict_IO_unavailable` covers both IO and its File host type.

The full census was not rerun. Its last measurement at
`d224e79d33b4a0ab26f701b57f448056b63b790a` remains **300 differences beyond 385
historical IO exclusions plus six upstream stack failures**, with unchanged
production tree verified. Current File/IO scope explanations do not silently
subtract from that historical set. Residual triage remains 149 diagnostic candidates,
109 acceptance differences and 42 other. F32.read, failure order, erased-hole/TODO,
printer/oracle/foreign/resource cases, external API caveats and the incomplete
688-vector exploration remain unwaived.

Inventory now has **424 linked / 878 unlinked**: 523 interpreter-and-compiled
candidates, 341 checker negatives, 13 interpreter profiles and one checking-only
module remain unlinked. The new read_bounds link explicitly records upstream JS
as blocked; a link is not completed parity. Full acceptance remains governed by
the [coverage checklist](cpu-parity-acceptance.md).

## Reproduction and preservation

With the provisioned environment activated:

```bash
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test cpu_read_bounds --test kernel_numeric_text
cargo clippy -p teamy-bend --locked --all-features --test cpu_read_bounds -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Debug/incremental overrides control host build-cache size, not Bend runtime
optimization. For each original/positive/unresolved fixture, set `fixture`, the
pinned `reference` checkout and a fresh `artifacts` directory. Repeat executions twice:

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

The integration test uses strict unoptimized C and 20-second generated-runtime
bounds; the differential and isolation processes use 30 seconds. For isolation,
retain the word fixture imports and replace main with each listed IO.print alone,
then prefixes 4/5/6/16. Repeated native checked-book invocation tests isolation,
not allocator reclamation or mid-normalization cancellation.

[Validation](cpu-read-bounds-validation.json) records exact sources, commands,
frozen inputs, reviews, binary hash and raw receipt hashes. Production, dependency,
toolchain, pin, guard, budget and demand/cancellation behavior are unchanged.
Disk began at 11.908 GiB; minimum workspace headroom was
11.892 GiB, above the 10 GiB stop floor, with
5.031 GiB minimum temporary headroom. The existing
verified cache relocation was retained; no unique evidence was removed.
Git preserves originals, controls, the failed full aggregate and compact receipts.
Bulk raw logs, generated artifacts and binary archives remain local and are not
preserved by the feature-branch push. No desktop/GPU, install, merge or release.
