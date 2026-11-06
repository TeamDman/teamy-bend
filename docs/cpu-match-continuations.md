# Variable-column and skipped-binder continuations — 2026-11-06

The pattern compiler's variable-only column and skipped-binder branches
previously re-entered `flatten_match` through Rust calls. They now schedule
explicit continuations in the existing match driver. A skipped binder wraps the
completed child in its lambda after that child succeeds, in the same order as
before. Positive branches and fallbacks retain their existing ordering and
origin handling. When skipping binders, the driver moves an already-owned match
state instead of repeatedly cloning its rows; the initial borrowed state is
copied once. Variable-only columns copy their remaining scrutinees and variable
vectors into the next state. That additional storage depends on the input and
the existing 64-level bound; no absolute allocation bound is claimed.

The compiler depth guard is unchanged. Nested body compilation can still
re-enter `flatten_at` and `flatten_match`, and the separate input and kernel
128-level boundaries remain. The unchanged original `"hi"` pattern still
fails at the same 64-level compilation guard in port checking and execution;
pinned upstream returns `True{}`. This change does not make that witness pass.
No deeper compiler path has been exposed by removing a guard.

Focused tests on a thread requesting a 64 KiB stack check variable-only columns
and skipped binders through 60 levels, lambda order and the respective existing
depth refusals. Other tests exercise error order, source origins, shared terms,
ordinary cleanup and isolated allocation failures. A first focused run failed
because the new test incorrectly expected skipped binders to hit the guard one
level earlier; that test was corrected to account for the existing nullary
pending-frame behavior and the suite passed. This was a test expectation error,
not a passing first run.

The final focused runs passed 20 surface unit tests and 26 integration tests
covering pattern locations, the string-pattern regression, ordinary term
cleanup, allocator-failure subprocesses and binder/constructor behavior. An
additional test command initially named a fixture directory as a Cargo target;
no tests ran in that attempt. The corrected target selection passed. Ten neighboring programs
matched pinned upstream twice. Across 55 string, Nat and location fixtures,
220 candidate observations matched the previous binary in exit status and both
output streams. Matching refusals are not runtime passes. Original upstream
`"hi"` returned `True{}`; candidate checking and execution each refused at the
64-level compiler guard. Comparison subprocesses had 30-second deadlines; the
outer runner used a 1,200-second limit, an 11 GiB preflight and 10 GiB active
workspace disk floor, and a 1 GiB temporary disk floor. No extra OS memory cap
was applied. GPU and other host platforms were unrun.

The base commit is `3d4f4de5f8e04e6279b70657c1624ad08a351cfc`, and the pinned
upstream commit is `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. Strict
all-feature production Clippy and build, workspace formatting, and the Node
TCP/UDP provider check passed. This intermediate change did not rerun
the complete bounded test set or the uninterrupted standard gate; the previous
base's 1,417 passed, zero failed and 49 ignored remains evidence for that base,
not a test-set result for this source. The prior standard gate timed out at
1,200 seconds and was incomplete.
See the [validation receipt](cpu-match-continuations-validation.json) for
source and executable hashes and stage outcomes.
