<!-- SPDX-License-Identifier: MPL-2.0 -->
# Diagnostic quantitative-meet precedence

The two `check/` originals are unchanged Apache-2.0 fixtures from Bend pin
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`; see NOTICE and licenses/Apache-2.0.txt.
Other controls are MPL-2.0. Expected stderr comes from the pinned interpreter;
`.internal` retains pre-fix public KernelError Display text.

`printer-cases.json` separates pinned diagnostic spelling from measured legacy
Term Display. Its oracle parses each expression, converts to higher-order then
back to level syntax (without reduction), and calls pinned term_show. These are
presentation observations, not claims that arbitrary displayed terms typecheck.

The diagnostic formatter preserves nested-meet grouping and scope. Public term
Display, plain errors, normal-form text and semantic graphs stay unchanged.
Status-bearing CLI errors remain Ok(CliOutput) with exit_code 1; no new Err/Ok
transition is introduced. Derived Debug includes diagnostics and can change.
External invoke consumers remain unaudited. False proofs and quantity overuse
remain rejected; invalid-source executable runs are unrun.
