<!-- SPDX-License-Identifier: MPL-2.0 -->
# Contextual reflexivity rejection diagnostics

The two files under `check/` are unchanged copies from Bend pin
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, Apache-2.0; see the repository NOTICE
and licenses/Apache-2.0.txt. Authored controls are MPL-2.0.

Pinned interpreter stdout/stderr supplies the expected streams. `before.json`
retains compact pre-fix observations, actual source/binary identity and hashes;
machine paths alone are replaced with documented placeholders.

The wrong-shape proof remains rejected. Its normalized type, context and exact
proof occurrence become optional presentation metadata after rejection. Plain
KernelError message/equality remain unchanged. The CLI uses its existing
Ok(CliOutput) with exit_code1 boundary, which callers must inspect; external
invoke consumers remain unaudited. True/false equation controls, including closed
partial calls, retain their checking and execution behavior. Strict IO proof
refusal and actual IO execution remain separate observations.
