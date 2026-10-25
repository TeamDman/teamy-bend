<!-- SPDX-License-Identifier: MPL-2.0 -->
# Final open-law TODO summaries

Two unchanged pinned originals and five authored controls exercise the final
inventory of unfilled ordinary laws. Exact upstream stderr has a count and no
source location. `internal-errors.json` preserves the public KernelError text;
CLI tests additionally require status 1, empty stdout and absent output artifacts.

Controls cover one/two/three open laws, no main, a partially completed inventory,
repeated imports counted once, and sealed Base declarations excluded from counts.
Zero-inventory empty/datatype-only modules are checking-only: no runtime is claimed.
Filled/erased pure entries normalize to 7; generated programs print `7\n`.
The sealed numeric/IO entry executes IO and prints `3\n` while strict proof mode
still excludes IO.

The three `?TODO` guards intentionally retain earlier source-hole refusals, even
when erasure would discard the hole. Upstream final hole counts remain a separate
unresolved contract. The false-proof and live-use guards keep their earlier errors
and source windows; this change does not turn every rejection into a TODO summary.
