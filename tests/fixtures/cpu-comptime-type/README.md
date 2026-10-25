<!-- SPDX-License-Identifier: MPL-2.0 -->
# Application checking provenance

Six upstream originals are unchanged and attributed in NOTICE.md. Negative
controls exercise fresh argument variables, imported/multiline/nested templates,
ordinary applications, repeated uses, quantity atoms/types and adjacent quantity/name/function failures.
Expected stderr is captured from the pinned checking-only adapter; phase and
executable refusal observations are recorded separately from actual execution.

Five pure controls and two original IO programs cover repeated specialization,
shadowed lambda binders, reusable arguments and ordinary function application.
Native IO, pure normalization and generated C/JS execution are distinct lanes.
Strict proof checking continues to reject IO.

The correction changes external weak provenance only: core nodes and sharing are
unchanged. Public parser/KernelError messages, Display and equality are retained.
New formatted invoke failures use the existing Ok result with exit_code1;
external consumers remain unaudited. Cancellation coverage concerns existing
entry/diagnostic boundaries, not mid-normalization cancellation.
