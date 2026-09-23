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

On Windows, launch Compute Sanitizer through `scripts/compute-sanitizer.ps1`.
Custom bounded runners must set the child environment variable
`NV_COMPUTE_SANITIZER_LOCAL_CONNECTION_OVERRIDE=named-pipes` and record that
transport in their receipts. TCP sanitizer transport has caused firewall
approval prompts for each temporary test executable. Keep the override scoped
to validation processes; do not change system firewall settings or disable
Bend's networking effects to silence these prompts.

Windows network integration tests must keep listeners local: compile disposable
C-network programs with `TEAMY_BEND_TEST_LOOPBACK_NETWORK=1` and set
`TEAMY_BEND_TEST_LOOPBACK_NETWORK=1` only in CLI/JavaScript test child
processes. Internal Rust host-adapter tests use loopback under `cfg(test)`.
Keep normal TCP/UDP bind addresses unchanged; do not set this environment
variable globally.
