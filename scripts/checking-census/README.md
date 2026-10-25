<!-- SPDX-License-Identifier: MPL-2.0 -->
# Checking census and executable applicability

`run.py` measures all pinned fixtures in strict checking mode. `upstream.mjs`
loads and checks without running `main`. `phase-probe.rs` defaults to strict
checking; `--executable` selects executable loading/checking only. Neither probe
executes a program.

Build the probe against the same clean checkout, Cargo feature set and matching
library artifact as the CLI. Preserve the exact rustc command and artifact hash.
The build receipt must include `source_commit`, `binary_sha256` and
`phase_probe_sha256`; these are validated against the checkout and binaries.

```sh
python3 scripts/checking-census/acceptance.py \
  --reference "$UPSTREAM" --binary "$BEND" --phase-probe "$PROBE" \
  --census docs/cpu-lambda-arm-census.csv \
  --build-receipt "$BUILD_RECEIPT" --output "$FRESH_OUTPUT"
```

The reconciled **CSV**, not raw JSONL, selects acceptance disagreements outside
the explicitly renewed historical IO set. This input selects 109 rows. The
runner preserves every selected observation: strict checking, pinned checking,
executable checking and, for executable refusals, JavaScript generation errors.
It never executes generated programs or interprets acceptance as runtime parity.
Generation happens only after a recorded checking refusal and must emit no file.

Four workers, 30-second upstream and 60-second port deadlines, process-group
cleanup, an 11-GiB workspace preflight and 10-GiB workspace/1-GiB temporary-space
stop floors are host validation controls. They do not alter Bend budgets.
Keep raw results outside tracked source; publish sanitized compact evidence and
record any failed run before retrying. `completed` means all rows were recorded;
`acceptance_observations_renewed` separately reports consistency checks.
