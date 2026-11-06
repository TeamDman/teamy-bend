# Automatic cleanup for partial kernel results — 2026-11-06

On base `084046b0a6d6c809350d6e1865a7862463ee137e`, ordinary destruction of
a deeply nested `Rc<Term>` is still recursive. A disposable model of the
substitution result vector containing a 20,000-node chain aborted with stack
overflow while unwinding on a thread requesting a 64 KiB stack. A separate
model of the memo's map had the same outcome. Core dumps were disabled. These
were reduced models of the old ownership behavior, not claims that ordinary
checked source programs reached those depths.

The kernel now automatically releases intermediate owned results iteratively
in three specific places:

- The explicit result stack inside `substitute` cleans up remaining roots on
  unwind or an internal invariant failure. A successful returned `TermRef`
  retains its existing public ownership and ordinary drop behavior.
- Constructor and datatype child lists being freshened or strongly normalized
  release already-produced children if a later child returns an error or
  unwinds. The mapping retains source order and stops at the first failure.
- The private normalization memo releases cached values iteratively when it is
  dropped or when an entry is replaced. Its keys remain weak and shared values
  stay alive while another owner exists.

The existing explicit `release_term_iteratively` helper now avoids allocating
a work list when the root is shared; releasing that owner cannot destroy the
graph. The automatic paths do not alter term graphs, binder identities, demand,
fuel or guard thresholds. Temporary cleanup storage is proportional to pending
edges. The checked input and engine budgets remain in force; cleanup itself
must complete rather than abandon owned results at a new depth threshold.

This is **not automatic cleanup for all terms**. Ordinary public `Rc<Term>`
drops, scalar intermediate locals, other caches and parser/compiler data still
need review. In particular, making these paths safe does not justify removing
the pattern compiler or kernel guards yet.

## Evidence

Small-stack unit tests exercise the real substitution result stack with a
test-only injected panic after a 20,000-node partial result; a weak reference
confirms the result is released. Tests of the same child-map helper used by
freshening and normalization force a later `Err` and a panic, verify the first
failure and skipped later child, and check unique and shared ownership. Memo
tests exercise normal drop, replacement, a single result shared by two entries,
and panic unwinding with both unique and retained owners. All request a 64 KiB
thread stack and use 20,000-node test graphs.

Additional kernel tests exercise constructor and datatype child mapping at
every fuel value from zero through the exact successful boundary. Freshening
requires three steps and normalization six for the selected two-child terms;
failure unwinds depth, preserves fresh IDs and owners, and clears the
normalization memo. These are bounded controls, not a claim about every term.
The cleanup-filtered unit run passed seven tests, zero failed and zero ignored;
the exact-fuel test passed separately. Existing focused substitution, shared
input, normalization, String rendering and string-pattern targets also passed.

Against pinned upstream `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, seven
nearby empty, ASCII and Unicode string-pattern controls matched the upstream
interpreter and native port twice each. The unchanged original returned
`True{}` upstream twice and still failed port loading at the 64-level pattern
guard. The manual compact matcher continued to match the complete upstream
expanded and open alpha-trees twice; its whole-function normalization still
reached the 128-level engine guard. Its six closed inputs agreed through two
port APIs twice. Those observations do not qualify the original source as a
runtime pass. The [compact receipt](cpu-stack-auto-cleanup-evidence.json)
records source and artifact hashes and 36 separately labelled observations.

Production all-feature Clippy passed with warnings denied. A supplementary
all-target Clippy attempt on an earlier candidate remained unsuccessful on
unchanged historical test lint in `kernel_dead_residue.rs` (`map_err_ignore` and two
`assertions_on_result_states` diagnostics). An earlier attempt also exposed a
now-corrected unfulfilled lint expectation and a new test-style warning; those
failed attempts are not counted as passes.

Each comparison child had a 30-second deadline; the bounded outer runner used
1,200 seconds, at least 11 GiB workspace free before launch, 10 GiB while
active and 1 GiB temporary-storage headroom. No additional OS memory cap was
applied. Rust 1.96.0, dependencies and the pinned reference were unchanged.
GPU and other host platforms were unrun. The complete final-source offline
test set passed in bounded batches: **1,389 passed / 0 failed / 49 ignored**,
covering all 200 integration targets plus library, binary and documentation
tests. This is separate from the uninterrupted standard gate.

The final-source standard gate passed formatting, strict production Clippy,
the all-feature build and the real Node TCP/UDP provider check. It stopped at
the 1,200-second deadline during tests: 136 completed groups contained
**959 passed / 0 failed / 48 ignored**; the active group was interrupted and
later groups were unrun. This is not a gate pass. The separate bounded test-set
pass above does not change that result.
