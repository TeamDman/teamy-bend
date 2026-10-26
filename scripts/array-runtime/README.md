# Bounded array runtime comparisons

Run with the existing pinned tools, an explicitly selected port binary and a new
output directory. The runner refuses an altered upstream checkout or source
fixture, uses 60-second child limits, records exact streams/statuses and repeats
each executable twice. Output paths must not already exist. It preserves the
10 GiB workspace and 1 GiB temporary active-run floors; perform the separate
11 GiB workspace preflight before builds.

```sh
python3 scripts/array-runtime/compare.py \
  --repo "$REPO" --reference "$REFERENCE" --binary "$PORT_BINARY" \
  --manifest "$REPO/scripts/array-runtime/cases.json" --output "$NEW_EVIDENCE"
```

C uses pinned upstream's CPU `-std=c11 -O3 -lpthread -lm` profile; port C adds
strict warnings and its normal pthread/math/dl linkage. The native/port/upstream
generated C/JS values are exact byte comparisons, not textual normalization.
Negative checker rows require the precise stored diagnostics and no artifacts.
IO/F32 strict-proof refusals are not counted as runtime successes.

The depth-32 witness deliberately retains the port-C resource diagnostic
difference. The runner asserts the observed refusal, records it as unwaived, and
never labels whole CPU parity complete. It does not execute enormous depth-31
allocation experiments, change limits, or claim physical layout/heap inspection
for uninstrumented runtime lanes.
