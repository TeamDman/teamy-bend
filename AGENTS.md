Each subcommand must have its own directory module.
Each subcommand implementation must live in a new `{}_{}_{}_cli.rs` file that `mod.rs` re-exports to ensure fuzzy finders can find the file easily.

Read `docs/implementation-plan.md` before continuing the rewrite. The broad goal
remains active; the first milestone is a supported subset, not full Bend parity.
Unsupported constructs, incomplete proofs, unsafe assumptions and exhausted
resource limits must fail closed. Keep regression tests for false proofs and
for inputs that previously crashed. Do not treat matching rejections as full
upstream semantic compatibility.

Run `./check-all.ps1` for the standard quality gate. Compiler runtime tests need
Node.js on PATH or `TEAMY_BEND_NODE`. Keep translated Bend files under Apache-2.0
with their attribution; new project files use MPL-2.0. Never commit local paths,
raw audit diagnostics, captures or unrelated reference-checkout artifacts.
