<!-- SPDX-License-Identifier: MPL-2.0 -->
# Lambda rejection diagnostics

The files under `proof/` and `check/` are unchanged Apache-2.0 originals from Bend
pin `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`; see NOTICE and
licenses/Apache-2.0.txt. Authored controls are MPL-2.0.

The checker still rejects a lambda against an equation or other non-function
expected type. Optional diagnostics report the original expected type, typeless
lambda, outer context and the lambda's source occurrence. No function-extensionality
rule is introduced. The misleadingly named upstream no_eta_conversion fixture is
a positive control: pinned definitional eta conversion already supports its refl.

Stderr goldens come from pinned checking. `.internal` preserves measured pre-fix
KernelError Display text. `before.json` retains source hashes and compact pre-fix
streams with only machine paths replaced. Three pure successful controls separately
exercise normalization and generated C/JS; no native IO claim is made.

The plain kernel message/Display/equality remains unchanged. Newly sourced CLI
failures use existing Ok(CliOutput) with exit_code1 instead of their previous host
Err; callers must inspect status. Derived Debug can change with metadata. External
invoke callers remain unaudited. Source-hole refusal, live budgets, guards,
termination and sharing remain unchanged; failed presentation falls back.
