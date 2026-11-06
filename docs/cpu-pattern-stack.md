# Iterative nullary pattern fallbacks — 2026-11-06

On base `ddffe7f039c2b19b99d279270264815185746304`, the compiler compiled
nullary-constructor positive columns iteratively but recursively called the
same compiler to build each fallback. Nested alternatives therefore consumed
a Rust call frame for each fallback. This slice stores those suspended calls
in an explicit vector and reconstructs matches in the same order. A suspended
compiled arm now uses iterative owned-graph release when a later fallback
fails or unwinds. No dependency, toolchain, input budget or production guard
changed.

The logical guard still counts the same pending positive columns and fallback
call depth; field-bearing constructor arms, variable-only columns, nested
bodies and other compiler operations still have recursive paths. The
normalizer, checker, parser and ordinary public `Rc<Term>` destruction are
also not generally stack-safe. In particular, iterative release of suspended
arms does not make the other owned state, or returned terms, automatically safe
to drop at arbitrary depth. The original `"hi"` pattern still fails loading at
the 64-level compilation guard, and the kernel's 128-level guard remains a
separate obstacle. This change is an architectural step, not an end-to-end
string-pattern repair or justification for removing either guard.

## Evidence

On threads requesting 64 KiB stacks, direct compiler tests build 48 and 63
nested nullary alternatives, inspect every resulting arm and fallback, and
confirm that 64 alternatives still get the original guard error. A private
test allows this driver's bound to be 512 for 256 flat columns: all resulting
matches are inspected and explicitly released. Its remaining recursive calls
stay at depth one; this test does not change any public limit or prove the
remaining recursion safe. Tests exercise failure ordering when a positive arm
and a later fallback have different errors, and check that failure does not
retain a shared arm. Separate tests release a suspended 20,000-node arm during
panic unwinding and preserve a shared 20,000-node arm until its last owner is
explicitly released. These tests do not claim that public `Rc` drop is safe.

The unchanged original `tests/parse/string_pattern_sugar.bend` returned
`True{}` twice in the pinned upstream interpreter
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`; the port's check refused it twice
at the 64-level guard. Seven nearby empty, ASCII and Unicode string controls
matched upstream twice. Across 52 existing string, Nat and pattern-location
fixtures, 208 before/after check and run observations matched exactly in exit
status and both output streams. This includes errors and refusals; it is not
208 successful programs. The [comparison receipt](cpu-pattern-stack-evidence.json)
records hashes for each observation, source, binaries and pinned reference.

Each comparison process had a 30-second deadline; the bounded outer runner had
a 1,200-second deadline, required 11 GiB free workspace before starting and
10 GiB while active, and required 1 GiB free temporary storage. No additional
OS memory cap was applied. Rust 1.96.0, dependencies and the reference pin are
unchanged. GPU and other host-platform validation were not run.

The final-source offline test set completed in bounded batches: **1,394 passed,
zero failed and 49 ignored**. The two integration batches covered all 200
integration targets (1,009 passed, zero failed, 48 ignored); separate library,
binary and documentation tests contributed 385 passes and one ignored. The
seven focused surface unit tests include five new tests. An earlier partial
batch was interrupted before a test-only lint correction and is not included.

Production all-feature Clippy passed with warnings denied. A separate
supplementary all-target Clippy check failed on pre-existing diagnostics in
other files, including parser and kernel test code and
`tests/cli_checker_diagnostics.rs`; it did not report a diagnostic in the
changed pattern compiler. This supplementary check is not a pass.

The separate final-source `check-all.ps1` gate passed formatting, strict
production Clippy, the all-feature build and the real Node TCP/UDP provider
check. It then hit the 1,200-second deadline during the test stage, with **962
passed, zero failed and 48 ignored** across 135 completed groups. The active
group was interrupted and later groups were unrun in that attempt. This is an
incomplete gate, not a pass; the separate bounded test-set pass does not change
that result. The [validation receipt](cpu-pattern-stack-validation.json)
records commands, source identity, elapsed times and resource floors.
