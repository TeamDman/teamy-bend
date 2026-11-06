# String-pattern stack-safety path and ownership boundary — 2026-11-06

The unchanged upstream `parse/string_pattern_sugar.bend` returns `True{}` in
the pinned interpreter. At port source `56954a2a06002dd20f284c948b1725b9adc89579`
it still refuses in pattern compilation at 64 levels. This investigation maps
the path and tests an ownership design; it does **not** change production code,
remove a guard, or make the source execute.

## Path and dependencies

1. The parser turns a string literal into `SCon`/`SNil` containing `Chr`,
   `U32` and 32-bit `WCon` constructors; `Parser::pattern` then recursively
   converts the constructor tree and records binder and occurrence origins.
   Expression and body parsing have their own depth limits, and string length
   has a separate 128-character limit. `Pattern` and `Body` have derived
   recursive clone/debug/drop paths. `Body::with_reusable`, `Body::replace`
   and `Pattern::term` also recurse.
2. `flatten_occurrence` enters `flatten_at` and `flatten_match`. Nullary and
   field-bearing constructor branches now use explicit frames, but nested
   bodies, variable-only columns and binder skips can still re-enter
   recursively. Three checks in those functions enforce the logical 64-level
   pattern-compilation limit. Conversion must preserve positive-before-fallback
   order, fresh IDs, source origins, substitution/sharing and failures;
   replacing host frames also adds pending-state allocations that need bounded
   accounting. Dropping partial `Pattern`, `Body` and surface/core values must
   work on success, error and unwind.
3. The output graph uses public `TermRef = Rc<Term>` values in public `Book`
   declarations and as results. Both normal and failed operations can leave
   the final strong owner outside internal cleanup wrappers. The public enum's
   default destruction recursively drops unique child `Rc`s; an explicit
   `release_term_iteratively` call does not protect an ordinary external drop.
   This ownership problem is independent of whether a checker accepts a term.
4. Both strict and executable checking use `prepare_book` first. Its
   `inspect_input` is already iterative, but rejects depth greater than 128
   and charges a 2,000,000-occurrence budget. Its traversal order and depth
   checks cannot be replaced by unique-node counting: a shared occurrence may
   appear on a deeper path and occurrences are separately budgeted. Removing the
   input-depth refusal without making later consumers safe would expose their
   recursion. The earlier compact matcher expanded to depth 138 and 610
   occurrences; that is a measured manual prototype, not an unguarded
   compilation of the original source.
5. The kernel's separate `Engine::enter` guard rejects at depth 128, and its
   per-run work fuel is 2,000,000. `freshen`, `check`/`infer`, `whnf`, strong
   normalization and comparison retain recursive calls; substitution and
   portions of weak-head evaluation already use explicit work structures.
   Exact fuel charging, early-error order, binder freshening, demand, canonical
   open forms and provenance must survive any conversion. Upstream's
   `term_wnf` uses frames for part of the machine but also has recursive calls;
   it is an oracle, not a proof that this port's guard can simply be deleted.
   The checked data API's depth-96 output limit is a further independent
   boundary, not the pattern or kernel limit.

Safely retiring the **compilation** guard requires the whole reachable
parser/surface traversal and automatic ownership/cleanup path to be safe, with
explicit work and allocation limits; the kernel input guard can remain in
place at that stage and may then reject the generated graph. Executing the
original requires separately resolving input inspection and the kernel's
checking/normalization/ownership paths, then validating runtime and output.
No claim is made that just moving the refusal would establish compatibility.

## Ownership design experiment

The tracked files under `scripts/stack-safety` are isolated research probes,
not a production implementation. On Linux with Rust 1.96.0 and a thread
requesting a 64 KiB stack, an actual public `Rc<Term>` chain of 100,000 nodes
aborted with stack overflow in each of four subprocesses: unique normal drop,
last drop of a shared root, early error, and unwind. Core dumps were disabled.
The test does not assert that the parser or checker accepts such a graph.

A reduced three-variant model with a custom iterative `Drop` handled unique,
shared-last-owner, diamond, error and caught-panic cleanup at the same depth
and requested stack. One illustrative run peaked at 8,448 KiB child RSS; that
is not a memory bound. The model allocates a work vector and placeholder nodes,
is not the real `Term` enum, and has not established full graph, allocation,
first-error, weak-reference, cycle, destructor-order or low-memory behavior.
It must not be adopted as a production fix on this evidence.

There is a concrete public-interface tradeoff. A real consumer can currently
`Rc::try_unwrap` a `TermRef` and move the child from `Term::Typ`. A model with
an otherwise equivalent enum plus `impl Drop` rejects that operation with Rust
error E0509. Adding `Drop` to the real public enum would also require changing
internal owned destructuring, including the existing iterative-release helper;
it preserves the `Rc` identity type but breaks this source pattern for
consumers. Replacing `TermRef` with a guarded newtype is also a public type
change and cannot by itself protect consumers who construct a bare
`Rc<Term>` from the still-public enum. Guarding only `Book` or internal
intermediates leaves external last-owner drops unprotected.

The next coordinated design should first establish the intended public API
migration for owned destructuring, then prototype cleanup across **all** real
`Term` variants and all owners, preserving `Rc`/`Weak` sharing and covering
ordinary success, rejected inputs, early errors and unwinding. Measure and
bound pending storage and allocation failures; cleanup cannot safely abandon
owned nodes on a budget error. In parallel within the same design, convert
parser/surface recursion and cleanup, then the separate input and kernel paths,
with tests for shared occurrence depth, work accounting and first errors. The
small `Drop` model alone is not sufficient authorization to change the public
interface or drop the guard.

## Reproduction

From the repository root, build the unchanged feature-matched library and
compile the standalone model and consumer probes with the pinned Rust toolchain:

```sh
cargo build --locked --offline --features extended_observability
rustc --edition=2024 -C debuginfo=0 -D warnings scripts/stack-safety/rc-drop-model.rs -o target/rc-drop-model
rustc --edition=2024 -C debuginfo=0 -D warnings scripts/stack-safety/owned-term-consumer.rs --extern teamy_bend=target/debug/libteamy_bend.rlib -L dependency=target/debug/deps -o target/owned-term-consumer
target/rc-drop-model
target/owned-term-consumer
rustc --edition=2024 -C debuginfo=0 scripts/stack-safety/owned-term-drop-model.rs -o target/owned-term-drop-model
```

The last compile is expected to fail with E0509. The separate
`public-rc-drop.rs` intentionally aborts under the tested conditions: compile
it against the same library, disable core dumps, and run each of `unique`,
`shared`, `error` and `unwind` only in an isolated process with a deadline.
These probes exercise Rust-host ownership, not Bend source acceptance.

## Validation and limits

Ten neighboring string and nested-constructor controls matched the pinned
upstream interpreter twice each. The unchanged original returned `True{}`
upstream twice and port checking and running each refused twice at 64 levels. The
[compact evidence](cpu-stack-path-map-evidence.json) records source and binary
hashes, per-observation statuses and the expected negative cleanup outcomes.
The research programs compiled with `rustc -D warnings` except the deliberately
failing compatibility example, which produced E0509; the model and current
public consumer passed. Each comparison or isolated negative subprocess had a
30-second deadline; the outer runner used 1,200 seconds with 11 GiB workspace
free before launch, 10 GiB while running and 1 GiB temporary headroom. No extra
OS memory cap was imposed. GPU and other host platforms were unrun.

The unchanged production source completed the full offline test set in bounded
batches: **1,397 passed, zero failed and 49 ignored**, across all 200
integration targets plus library, binary and documentation tests. The
standalone research probes are separate from that test count; their formatting
check passed. The prior production revision's supplementary all-target Clippy
attempt failed on an existing lint in `tests/compiler_open_closures.rs`; it was
not rerun for this documentation and research-only change.

The separate standard single-command gate passed formatting, strict production
Clippy, the all-feature build and the real Node TCP/UDP provider check. It hit
the 1,200-second limit during tests: 135 completed groups had **965 passed,
zero failed and 48 ignored**. The active group was interrupted and later
groups were unrun. This is an incomplete gate, not a pass; the bounded full
test-set pass does not change it. The
[validation receipt](cpu-stack-path-map-validation.json) records commands,
timings, source identity and limits.
