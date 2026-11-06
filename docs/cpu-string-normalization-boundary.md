# String-pattern normalization boundary — 2026-11-06

The original two-character string-pattern program remains unsupported. On port
source `7ed72e354b908ed402b61fc7ed74835aaff6c50c` and pinned upstream
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, the unchanged upstream
`string_pattern_sugar.bend` returns `True{}` in the upstream interpreter; the
port's check and run both refuse during pattern compilation at 64 levels.
The copied original has SHA-256
`44b3907ace3402fa1e69f265baf39539e247efc4308e16a62bf43a9d6bde97ed`.
The port does not reach artifact generation for that source. Previous C/JS
original observations are recorded in [the depth investigation](cpu-string-pattern-depth.md).

The research-only [manual factoring prototype](cpu-compact-patterns.md) was
repeated twice on that clean port revision, using the exact freshly built
default plus extended-observability library. Its compact core is 72 levels and
616 tree occurrences; expansion is 138 levels and 610 occurrences. Its fully
expanded alpha-tree equals the pinned upstream normalized `is_hi` tree, and
the direct-cut open term matches upstream in that comparison. The explicitly
called variant does not match that open term. Six closed inputs agreed through
both port evaluation APIs, but normalizing the whole factored `is_hi` still
hits the kernel guard. This prototype is not source lowering and is not a fix.

## Reduced boundaries

The separate fixtures use `def main() -> Nat: Nat.add(64n, Nn)` and import
`Base`. All four pass port checking. The following observations were each
repeated twice; values denote the number of `Succ` constructors. Upstream
normal forms were independently measured as exactly that depth and one more
tree node. The checked data API results concern materializing an output, not
proof that the internal lazy computation did or did not complete.

| N | Upstream interpreter | Port pure run / normalize / evaluate | Port evaluate_data |
| ---: | ---: | --- | --- |
| 32 | 96 | 96 / 96 / 96 | 96 |
| 33 | 97 | 97 / 97 / 97 | output depth or node budget exhausted |
| 62 | 126 | 126 / 126 / 126 | output depth or node budget exhausted |
| 63 | 127 | kernel nesting limit exhausted / same / same | output depth or node budget exhausted |

The 126 and 127 cases also returned their expected result in all of upstream C,
upstream JavaScript, port generated C and port generated JavaScript, twice each.
These are bounded examples, not a general runtime qualification. C artifacts
were built with Clang 19.1.7 and `-std=c11 -O3`; port C additionally used
`-Wall -Wextra -Werror -pedantic`. C execution used one thread and GPU off;
JavaScript used Node 24.19.0. There were no errors or timeouts in these lanes.
The [compact receipt](cpu-string-normalization-boundary-evidence.json) retains
case, stage, exit and stream hashes, fixture hashes and generated-artifact
hashes; it deliberately omits raw streams.

An additional literal `126n` fixture checks successfully and, twice,
`normalize_definition("main")` returns 126 while `evaluate("main", [])`
hits the kernel nesting limit; `evaluate_data` hits its output limit. The APIs
start from different forms: the former normalizes the stored definition, while
the latter normalizes a checked application of a reference. Unfolding that
reference also freshens the stored body under the shared guard. This observation
does not establish that removing a frame preserves work accounting or error
ordering. It is a separate boundary to investigate, not a proposed fix.

The normalizer charges and enters the shared 128-level guard in both
`normalize` and `whnf`, and traverses constructor fields; its fuel is 2,000,000
steps. The data runtime independently limits output depth to 96 and output
nodes to 16,384, with a 2,000,000-step budget, 131,072 arena entries and 4,096
frames. The zero-based upstream tree depth is not identical to the number of
active kernel operations. These observations isolate the second obstacle after
pattern compilation, but do not prove that eliminating it alone is sufficient
for safe source-pattern lowering. A sound design still needs canonical open
normal forms, typed continuations, demand and error order, and equivalent
resource and cancellation behavior. Changing guard accounting or exempting
generated terms has not been established safe.

## Reproduction and scope

The sources are in `tests/fixtures/cpu-compact-patterns/nat-boundary-*.bend`.
The adjacent [research probe](../scripts/compact-patterns/nat-boundary-probe.rs)
prints all three checked API outcomes; it reports refusals as observations,
not passes. With the pinned toolchain and a freshly built default plus
extended-observability library:

```bash
cargo build --locked --offline --features extended_observability
rustc --edition=2024 -C debuginfo=0 scripts/compact-patterns/nat-boundary-probe.rs \
  --extern teamy_bend=target/debug/libteamy_bend.rlib \
  -L dependency=target/debug/deps -o target/nat-boundary-probe
target/nat-boundary-probe tests/fixtures/cpu-compact-patterns/nat-boundary-127.bend
```

Use the pinned upstream interpreter and
`scripts/compact-patterns/upstream-normalized.mjs` for independent results;
[the earlier report](cpu-compact-patterns.md) gives the prototype runner.
The measured port executable SHA-256 was
`05daad715321f0fd9d72689f5699af85b1b5e42741e746f57b012b3dea178b77`, and its
matching research library SHA-256 was
`6966e5e6129941169fc4ae8c00201c6e87e5b7b15a7208cc2230b50ab4809164`.
Individual comparison subprocesses had 30-second deadlines; the bounded outer
runner had a 1,200-second deadline and required 11 GiB free before launch,
10 GiB free while running, and 1 GiB free in temporary storage. No additional
OS memory cap was applied. GPU and other host platforms were not tested.

Focused verification on the same production source passed 44 tests, zero
failed and zero ignored: `cpu_string_pattern_depth` (3),
`kernel_normalization` (36), and `runtime_data` (5), with the
`extended_observability` feature. The research probe compiled with Rust
1.96.0 and `-D warnings` and passed the pinned nightly formatter. These test
results do not measure full corpus or platform compatibility.

The standard single-command gate on the same production source and these
research fixtures passed formatting, strict production Clippy, the all-feature
build and the real Node TCP/UDP provider check. At the unchanged 1,200-second
outer limit it stopped during the test phase: 134 completed groups contained
946 passed, zero failed and 48 ignored; the active group and subsequent groups
were incomplete or unrun. This is not a gate pass. A previous complete test-set
run at production-identical `c3a775c` passed 1,371 tests in bounded batches,
with zero failed and 49 ignored; it is not an uninterrupted gate. See
[NaN-payload validation](cpu-float-nan-overflow.md) for its exact scope.

No production implementation, dependencies, pinned reference, toolchain,
recursion guard or resource budget was changed by this investigation.
