# Automatic core-term cleanup — 2026-11-06

Core `Term` now implements iterative ordinary destruction. `TermRef` remains
`Rc<Term>` and existing weak references continue to point to the same nodes.
The destructor extracts child edges into a work list and unwraps only nodes
whose strong owner is unique; a shared node is left to its eventual last
owner. This applies to owned terms and to ordinary `Rc` release on success,
early errors and unwind. The old explicit release function now calls ordinary
drop. Its internal owned-field match was the only production `Rc::try_unwrap`
call site requiring migration; the standalone owned-term consumer was also
migrated to borrow and clone the child.

The Rust API intentionally changes: callers can no longer move enum fields out
of an owned `Term` by matching on it, including after `Rc::try_unwrap`, because
the type implements `Drop`. A checked example borrows the owned term and clones
the needed child. This does not change Bend's language contract. The immutable
public graph cannot form strong cycles through safe Rust; unsafe external code
is not covered.

The destructor allocates one `Rfl` placeholder and a growable work list for a
nontrivial graph; pending storage is proportional to outstanding edges. Before
extracting a node's edges, it reserves room for all of them. If work-list
reservation fails, the process aborts before a partially detached graph can
unwind recursively. Placeholder allocation uses Rust's normal infallible
allocation path, which aborts under the standard runtime allocation handler.
This is fail-closed termination, not a recoverable Bend resource error or an
allocation-free guarantee. A global allocator is required not to unwind.

Four isolated Linux child processes injected real null returns from a custom
global allocator at the placeholder and work-list allocations, each in normal
cleanup and during an already-active unwind. All terminated with SIGABRT as
designed, on a requested 64 KiB host stack, with core dumps disabled. Ordinary
small-stack tests cover all child-bearing core variants, unique and shared
last-owner release, weak references, 50,000-node mixed chains, a 20,000-edge
shared graph, early errors and caught unwinds. A separate counting-allocator
experiment on the preliminary version observed two allocation/reallocation
calls during release of a 50,000-node chain; its largest request was 88 bytes
with one outer edge or 160,000 bytes with 20,000 shared outer edges. These are
shape-specific observations, not resource ceilings or whole-runtime memory
guarantees.

No compiler or kernel depth restriction was removed. The unchanged original
`"hi"` string-pattern source still fails port `check` and `run` at the
64-level pattern-compilation guard, while pinned upstream returns `True{}`.
Remaining surface/compiler recursion, bounded work accounting, input checking
and the separate kernel 128-level boundary must be addressed before that
example can pass without unsafe host-stack exposure. See the earlier
[path map](cpu-stack-path-map.md); its ownership and API-authorization
conclusions are superseded by this change. This destructor does not make
derived `Debug` or other recursive consumers of a deep `Term` stack-safe.

## Validation

The base is `366a39e85ceaded64e32ef15310dcbb041e57749`; pinned upstream is
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. Focused ownership and allocation
tests passed (five tests, including four subprocess failure modes). Ten nearby
string and nested-constructor controls matched upstream twice each. The
original upstream source returned `True{}` twice and the port still refused
checking at the same compiler guard twice; an additional port run also
refused at that guard. Strict production and new-test Clippy passed with
warnings denied. Comparison children had 30-second deadlines; bounded Cargo
commands used 1,200 seconds, an 11 GiB preflight and 10 GiB active workspace
disk floor, and a 1 GiB temporary disk floor. No extra OS memory cap was set.
GPU and other host platforms were unrun.

The complete offline test set was run in bounded batches: **1,417 passed,
zero failed, 49 ignored**, covering all 202 integration targets and the
library, binary and documentation tests. Afterward, a strict test-target lint
required a raw-pointer spelling adjustment in the allocation-failure test;
that changed target was rerun successfully. Workspace formatting, all-feature
production Clippy and build, strict Clippy for both new test targets, and the
Node socket provider check passed. A first new-test Clippy attempt failed on
the pointer spelling and was corrected; it was not a pass. The uninterrupted
standard `check-all.ps1` gate was not rerun for this slice. Its previous
1,200-second timeout was incomplete, not a passing gate, and the bounded test
batches do not change that status. See the
[validation receipt](cpu-term-ownership-validation.json).
