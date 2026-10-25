# Word-reader work observation

MPL-2.0. These are archived observation tools, outside the production module tree.
`probe.rs` is preserved byte-for-byte from the measured experiment, including its
original formatting; its source hash is recorded in the measurement receipt.

Use a separate disposable checkout at baseline
`e381cc7b96d3683d1a5e33cf5e630d707cc0b6ec` and another at candidate
`7bf8cec77c94866ff5ad467335526f19b7ccaa86`. From each checkout root, copy this
folder from the checkpoint, then run:

```sh
git apply --check scripts/word-reading-work/instrumentation.patch
git apply scripts/word-reading-work/instrumentation.patch
cp scripts/word-reading-work/probe.rs src/runtime/work_probe.rs
CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4 \
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 \
cargo test --locked --features extended_observability --lib \
  runtime::work_probe::measure_word_reader_work -- --nocapture --test-threads=1
```

Use the existing frozen tools and cache. Do not install dependencies. The probe
checks the unchanged 16-case original plus individual and prefix controls, twice.
It reports results and successful budget charges, partitioned by call site,
plus packed/fallback/available operation-path selections and output boundaries.
An exhausted run still records its outcome; the probe test passing is not a claim
that every Bend program succeeded. The baseline refuses prefixes six and sixteen.
No wall-clock benchmark or cross-engine cost equivalence is asserted.

Counters are recorded after the normal cancellation check and decrement. They do
not call cancellation again or change guards, work limits, normal tick sites,
GC scheduling, matching or forcing. `track_caller` propagates charge attribution.
Do not add this instrumentation to production builds. The clean validation and
runtime matrix used uninstrumented source. Compare charges against
[the recorded measurements](../../docs/cpu-word-reading-work-measurements.json).
The after-source metadata initially copied the baseline numeric tick line 516;
correct line 520 is recorded, with the original metadata hash preserved. That
site has no observations in these runs, so no total changed.
