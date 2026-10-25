# Pattern binder source controls

The three unchanged originals under `check/` and `halt/` are from Bend
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, Copyright 2026 HigherOrderCO,
Apache-2.0; see the repository NOTICE and licenses/Apache-2.0.txt. New controls
and Rust tests use MPL-2.0.

`negative-cases.json` contains the three originals and eight adjacent checking
refusals. Expected stderr comes from the pinned upstream CLI. The controls
cover multiline/imported fields, nested constructors, both named and synthetic
binders, fallback substitution, selected-row priority, shadowing and closing
binder order. Compilation after rejection must leave no artifact.

`guard-cases.json` retains one additional unresolved location difference:
reusable function-field refusal. Both implementations load and reject while
checking the same Data-versus-Type obligation. Upstream check-let relocates
that kind failure to the let binder; the port retains the field type origin.
This is not exact diagnostic parity or an approved semantic exception.

Positive programs cover nested matches and a reusable Data field. Complete
normalized graphs/text and constructor values are separate from executable
emission, which requires Base for these programs.

`before-receipt.json` preserves small unique pre-fix failure streams/statuses
for the three originals. Command roots are placeholders; stream bytes remain
unchanged, including ANSI escapes and a repository-relative host source path.
No machine paths, credentials or binary artifacts are included. The user's
explicit request authorizes this compact diagnostic evidence despite the
repository's general prohibition on committing raw audit captures.

Run `cargo test --locked --features extended_observability --test cpu_pattern_locations`.
Keep `kernel_core`, `kernel_normalization`, `cpu_template_occurrences` and the
occurrence/shared-reference unit controls in validation for the prior stack
and identity regressions. Detailed commands and current outcomes are in the
corresponding report under docs.
