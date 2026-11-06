# Stack-safe private pattern trees — 2026-11-06

This change addresses the private parser/surface `Pattern` tree, not the public
`Rc<Term>` representation. On base `cd17e5309fe90ec39a7ea1f22d32f3a3e4e61b31`,
converting a core constructor pattern to a `Pattern`, cloning that tree, and
ordinary destruction of it all recursively used the host stack. The parser now
visits children with explicit frames; `Pattern::clone` uses explicit visit/build
work; and `Pattern` automatically drains its owned children iteratively on
normal drop, error or unwind. `Pattern` is private to the syntax module, so this
adds no public `Drop` implementation and leaves `TermRef = Rc<Term>` and owned
`Term` destructuring unchanged.

The parser retains left-to-right constructor validation, binder creation,
occurrence-source lookup and error order. It does not eagerly reserve the whole
child list before the first child is validated. The traversal and clone use heap
storage proportional to pending input and results; automatic cleanup uses storage
proportional to pending owned children and does not abandon cleanup at a new
limit. The existing parser expression-depth limit of 256, compiler limit of 64,
kernel occurrence/fuel limits and other guards are unchanged. No new aggregate
pattern-node or cancellation budget is claimed; allocation behavior and peak
memory depend on input shape. `Pattern::term`, derived pattern debugging,
`Body` operations, other compiler paths and ordinary public `Rc<Term>` drop
remain recursive.

This is a real step on the source-pattern path but does **not** make the original
`"hi"` program work or move its refusal: both port checking and running still
fail at the 64-level compilation guard, while pinned upstream returns `True{}`.
Retiring that guard still needs stack-safe reachable surface/body operations,
resource accounting and automatic cleanup of all term owners, including public
`Rc<Term>` last-owner drops. The separate kernel input and engine depth-128
boundaries, plus output and runtime limits, must then be addressed independently.
See the [path map](cpu-stack-path-map.md) for the ownership/API tradeoff.

## Focused evidence

On threads requesting 64 KiB stacks, tests clone and ordinarily drop a
100,000-level private pattern, inspect every cloned constructor and origin, and
drop an 8,192-level branching pattern with pending siblings. Separate normal,
error and injected-unwind tests exercise ordinary pattern cleanup. Direct
internal parser tests convert and ordinarily drop a 20,000-level pattern and
exercise both an error and injected unwind after a completed deep child; they
also check first-error order, binder state and each constructor origin. The input
core term in these tests is explicitly released because ordinary public
`Rc<Term>` destruction is still unsafe at that depth. These direct tests bypass
the public parser's existing depth limits and do not claim public source
acceptance at 20,000 levels. A separate control checks all non-pattern core term
families retain their rejection; the public owned-term consumer still compiles
and runs.

Single illustrative Linux child-process measurements of the deep clone/drop and
parser-error tests peaked at 27,196 KiB and 16,204 KiB RSS, respectively. These
are observations, not memory ceilings. The private cleanup work list may retain
pending siblings; the test does not prove an allocation-failure bound or safe
ordinary destruction of public term graphs.

Across 55 existing string, Nat and pattern-location fixtures, 220 before/after
check and run observations matched exactly in status and both output streams,
including refusals. Ten nearby string and nested-constructor controls matched
the pinned upstream interpreter twice each. The unchanged original returned
`True{}` upstream twice; port check and run each refused at the 64-level guard
twice. The [comparison receipt](cpu-pattern-tree-evidence.json) records source,
binary and per-observation hashes. Matching refusal is not a runtime pass.

Each comparison child had a 30-second deadline; the outer runner used 1,200
seconds, with at least 11 GiB workspace free before launch, 10 GiB while active
and 1 GiB temporary-storage headroom. No additional OS memory cap was applied.
The upstream pin, dependencies and Rust 1.96.0 toolchain are unchanged. GPU and
other host-platform validation were not run.

Final-source offline tests passed in bounded batches: **1,404 passed, zero
failed and 49 ignored**, including all 200 integration targets plus library,
binary and documentation tests. Focused checks passed 12 surface unit tests,
five parser unit tests and 91 integration tests. Strict production all-feature
Clippy passed. A supplementary all-target Clippy attempt failed on an existing
`Rc` clone lint in `tests/kernel_constructor_continuations.rs`; it is not a
pass.

The separate standard gate passed formatting, strict production Clippy, the
all-feature build and the real Node TCP/UDP provider check. It hit the
1,200-second limit during tests: 135 completed groups had **972 passed, zero
failed and 48 ignored**. The active group was interrupted and later groups
were unrun. This is an incomplete gate, not a pass; the separate bounded full
test-set pass does not change that result. The
[validation receipt](cpu-pattern-tree-validation.json) records commands,
source identity, timings and resource floors.
