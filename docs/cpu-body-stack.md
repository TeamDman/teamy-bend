# Stack-safe private body traversal and cleanup — 2026-11-06

This slice changes the private syntax `Body` path on base
`1336797c81c4a689451ee086487486c5d7d3f1b3`. The pattern compiler repeatedly
substitutes into case bodies. `Body::replace`, `Body::clone` and propagation of
reusable bindings now use explicit work rather than recursively traversing body
nodes. Mapping retains scrutinee, row, pattern and value order, occurrence
metadata and shared core references. A private empty variant is only a work
placeholder and must not reach normal lowering.

Private `Body` values now automatically drain their nested bodies and release
owned term roots iteratively on normal drop, early error and unwind. Partial
mapped values have an unwind guard. Provenance-aware substitution also guards
its single intermediate result until origin propagation completes. The guards
avoid destroying shared core values while another owner exists. This covers
private body ownership; a public or other external last `Rc<Term>` owner still
has the original recursive drop and is not made safe by this change. The public
`TermRef = Rc<Term>` type and owned `Term` destructuring remain unchanged.

The implementation uses heap work proportional to pending bodies, mapped
results and owned term edges. It can allocate during cleanup, and no new
aggregate body-node budget, cancellation guarantee or allocation-failure bound
is claimed. The parser's source limits, 64-level pattern-compilation guard,
2,000,000-occurrence/fuel budgets and 128-level kernel guards are unchanged.
Derived body debugging, other parser/compiler recursion, public term
ownership/destruction and kernel paths remain open. The
[path map](cpu-stack-path-map.md) explains why a public `Term` destructor is
not a drop-in compatible change.

This is a substantive step on the lowering path, but it does **not** make the
original `"hi"` program pass or move its refusal: both port checking and
running still hit the 64-level pattern guard, while pinned upstream returns
`True{}`. Removing that guard requires safe remaining recursive paths and
ownership on success, error and unwind; the input and kernel depth-128 and data
output limits remain separate downstream work.

## Focused evidence

Threads requesting 64 KiB stacks exercised ordinary private-body cleanup for
100,000 nested bodies on early error and unwind, and clone/substitution plus
normal cleanup for 20,000 bodies containing a 20,000-node core term. Other
controls checked reusable-binding traversal at 20,000 levels, row/binder order,
shared and last owners, all 12 composite core-term families, mapping order,
partial-result cleanup on panic, and provenance-substitution cleanup when
origin propagation unwinds. Tests verify `Weak` references and retained `Rc`
counts. Where an external last owner remains, the test explicitly releases it;
ordinary public `Rc<Term>` destruction is still not covered. These direct
internal tests bypass parser depth limits and do not establish public source
acceptance at those depths.

Illustrative Linux child-process peak RSS for individual clone/substitution,
error/unwind and reusable-binding controls was 16,764 KiB, 16,296 KiB and
9,344 KiB, respectively. These are observations, not memory ceilings. The
cleanup and traversal work lists can grow with pending input; allocation failure
and all possible graph sizes are not proved safe.

Across 55 existing string, Nat and pattern-location fixtures, 220 before/after
check and run observations matched exactly in status and both output streams,
including refusals. Ten nearby string and nested-constructor controls matched
pinned upstream twice. The unchanged original returned `True{}` upstream twice;
port check and run each refused at the 64-level guard twice. The existing public
owned-term consumer compiled and ran. The
[comparison receipt](cpu-body-stack-evidence.json) records source, binary and
per-observation hashes. Matching refusal is not a runtime pass.

Each comparison subprocess had a 30-second deadline; the outer runner used
1,200 seconds with at least 11 GiB workspace free before launch, 10 GiB while
active and 1 GiB temporary-storage headroom. No additional OS memory cap was
applied. The upstream pin, dependencies and Rust 1.96.0 toolchain are unchanged.
GPU and other host-platform validation were not run.

Final-source offline tests passed in bounded batches: **1,412 passed, zero
failed and 49 ignored**, including all 200 integration targets plus library,
binary and documentation tests. Focused checks passed 19 surface unit tests,
ten origin unit tests and 94 integration tests. Strict production all-feature
Clippy passed. A supplementary all-target Clippy attempt failed on existing
lint in `tests/compiler_dead_residue.rs` and `tests/kernel_dependent_proofs.rs`;
it is not a pass.

The separate standard gate passed formatting, strict production Clippy, the
all-feature build and the real Node TCP/UDP provider check. It hit the
1,200-second limit during tests: 135 completed groups had **980 passed, zero
failed and 48 ignored**. The active group was interrupted and later groups
were unrun. This is an incomplete gate, not a pass; the separate bounded full
test-set pass does not change that result. The
[validation receipt](cpu-body-stack-validation.json) records commands, source
identity, timings and resource floors.
