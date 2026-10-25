<!-- SPDX-License-Identifier: MPL-2.0 -->
# Nat-pattern representation measurements

Build `metrics.rs` against the matching frozen `default,extended_observability`
Teamy Bend library and record its fingerprint/hash and the rustc command:

```sh
rustc --edition=2024 -C debuginfo=0 scripts/nat-pattern-depth/metrics.rs \
  --extern "teamy_bend=$RLIB" -L dependency=target/debug/deps -o "$METRICS"
"$METRICS" "$FIXTURE" "$DEFINITION"
node scripts/nat-pattern-depth/upstream-metrics.mjs \
  "$UPSTREAM/bend2/bend.ts" "$FIXTURE" "$DEFINITION"
```

Use definition `flag` for the two unchanged originals in
`tests/fixtures/cpu-nat-pattern-depth`, and `pick` for the authored boundary
controls. Bound each process to 30 seconds, capture status/stdout/stderr and retain
failures. A probe process completing successfully does not mean its subject loaded,
checked or normalized: the Rust probe explicitly prints those separate outcomes.
The upstream probe checks before measuring; it does not execute effectful main.

Depth uses root 0 and all child edges. Counts are tree occurrences, including
repeated shared nodes, not allocation counts. Loaded and normalized bodies are
separate. The canonical traversal includes every observed semantic node and both
annotation children, uses de Bruijn binder indices, and fails on unsupported tags.
It excludes source metadata and binder display spelling; it does not claim pointer
identity equivalence between the implementations. Compare complete canonical
strings before publishing their hashes; top-level counts alone are insufficient.

No guard is raised or bypassed, and no term is rewritten by these probes. A port
load refusal yields no port graph or normalization evidence. Upstream C emitter
optimization is a separate later phase. Neither C success nor a compact prototype
qualifies the original port source. See the checkpoint report for exact source,
commands, preserved failures and resource limits.
