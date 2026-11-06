# Unified body and match continuations — 2026-11-06

Body lowering and pattern-match lowering used to call each other recursively.
They now share one explicit continuation driver. It schedules nested bodies,
matches, local bindings, positive arms and fallbacks without re-entering either
lowering function on the Rust stack. Local continuations reconstruct binders,
source origins and `Let` nodes after their child succeeds; match continuations
retain positive-before-fallback order. The parser transfers ownership of its
body into the driver rather than cloning the complete tree at entry. Existing
owned match rows transfer their selected body into a continuation while the
remaining row state stays alive for diagnostics and fallbacks.

The public 64-level pattern-compilation limit is unchanged. A private test
uses a larger bound to exercise 10,000 nested local bodies on a thread
requesting a 64 KiB stack: normal completion, bounded refusal, shared-last-owner
release and an injected invalid internal placeholder during unwind all clean up. A second
test compiles 128 nested matches on the same requested stack. These are direct
driver tests, not proof that the parser accepts programs of that depth.

The driver stores continuations and pending states on the heap. Body mapping,
pattern cloning and term substitution can each traverse or allocate in
proportion to their input, and substitutions can revisit shared term
occurrences. There is no shared, cumulative lowering-work or allocation budget
covering those operations yet. Removing the logical depth guard without such
accounting could turn a bounded refusal into excessive work or allocation.
Allocation failure in standard infallible Rust allocations terminates the
process; tests from the previous ownership slice inject failures in term
destruction, but allocation failure in a newly suspended body frame was not
directly injected in this slice. Ordinary error and unwind cleanup is tested;
no recoverable out-of-memory result is claimed.

The unchanged original `"hi"` witness still refuses in both port checking and
execution at the 64-level pattern-compilation guard. The pinned upstream
interpreter returns `True{}`. The separate 128-level kernel input and engine
limits remain, as do other parser and kernel recursive paths. The next safe
step is to introduce an explicit shared lowering budget that also charges
body mapping, pattern copying and occurrence-aware term substitution, then
prove error ordering and cleanup on exhaustion before retiring the compiler
guard. The old guard cannot simply be raised to claim progress.

## Validation

The base is `928bde016ea4d4c7be4c6d8136005cad343ee4ab`; the pinned upstream
is `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. Focused validation passed
130 tests: 22 surface tests, 26 pattern/constructor/cleanup tests and 82
parser/diagnostic tests. A first prototype run failed two tests because reply
binders were not reconstructed; this implementation error was fixed and the
suite rerun. The first production Clippy attempt failed on four diagnostics;
those were corrected and the strict all-feature rerun passed.
Ten neighboring programs matched upstream twice, and 220 before/after
observations across 55 pattern fixtures matched in exit status and both output
streams. Matching refusals are not runtime passes. Comparison subprocesses
used 30-second deadlines; bounded commands used 1,200 seconds, with 11 GiB
workspace free preflight, 10 GiB while active and 1 GiB temporary headroom.
No additional OS memory cap was set. GPU and other host platforms were unrun.

The complete offline test set passed in bounded batches: **1,420 passed,
zero failed and 49 ignored**, covering all 202 integration targets and the
library, binary and documentation tests. Workspace formatting, all-feature
production Clippy and build, and the Node TCP/UDP provider check passed. The
uninterrupted standard gate was not rerun; the prior 1,200-second gate timed
out and was incomplete. Bounded test-set coverage does not turn that gate into
a pass. See the [validation receipt](cpu-unified-body-match-validation.json).
