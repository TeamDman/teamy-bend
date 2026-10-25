<!-- SPDX-License-Identifier: MPL-2.0 -->
# Checking-only census

This adapter measures all 1,302 unchanged pinned fixtures. It never runs or
normalizes `main`. Use a clean Teamy Bend checkpoint and upstream
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. No packages are installed.

Build the existing CLI with the frozen profile and compile the small phase probe
against the **same freshly built** library. `RLIB` below is the
`libteamy_bend-*.rlib` whose Cargo fingerprint lists exactly `default` and
`extended_observability`; record its SHA-256. Do not select an older/all-features
artifact merely because its filename matches.

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo build --locked --offline --features extended_observability
rustc --edition=2024 -C debuginfo=0 scripts/checking-census/phase-probe.rs \
  --extern "teamy_bend=$RLIB" -L dependency=target/debug/deps -o "$PHASE_PROBE"
python3 scripts/checking-census/run.py --reference "$UPSTREAM" \
  --binary "$BEND" --phase-probe "$PHASE_PROBE" --output "$FRESH_OUTPUT" --build-receipt "$BUILD_RECEIPT"
```

`UPSTREAM`, `BEND`, `RLIB`, `PHASE_PROBE` and `FRESH_OUTPUT` are caller-selected
paths. `BUILD_RECEIPT` is a JSON record containing `source_commit`,
`binary_sha256` and `phase_probe_sha256`, plus the actual build commands, library
hash and Cargo fingerprint. Capture it from a clean build; the runner validates
those identities before execution and rechecks the executable hashes afterward.
The host Rust debug settings above are separate from Bend runtime profiles.

The Node adapter follows `main.ts` checking through `book_load`, `book_valid`
and incomplete-proof refusal. A separate sidecar records the load/check phase
and distinguishes Bend errors from host-stack/unexpected-host failures. It
preserves checking stderr and avoids the CLI's later `main` normalization.
The Rust probe supplies an independent plain-loader/checker phase; any status
disagreement with the actual CLI is an infrastructure/qualification failure.

Four workers run each oracle for at most 30 seconds, and each port command/probe
for at most 60 seconds; only the timed-out child process group is killed. The
runner checks the existing 10 GiB workspace floor before starting and writes
results incrementally. Exit success means the census completed, not parity.

Mutually exclusive categories sum to the inventory: both accept (status only),
exact rejection (phase/status/streams), different rejection, acceptance
disagreement, and oracle/infrastructure failure. `missing_IO_candidate` is only
a diagnostic symptom, not an approved exclusion. Historical reviewed subsets
must be reconciled separately using unchanged fixture hashes and exact streams;
the runner never subtracts IO candidates from its measured differences.

The raw JSONL contains local commands and diagnostics and belongs outside Git.
A public compact CSV may retain fixture hashes, phases, statuses and categories
without private paths or raw captures. Runtime, normalization, host support,
safety-policy differences and open semantic decisions need separate evidence.
