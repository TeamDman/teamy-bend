# Iterative field-bearing pattern branches — 2026-11-06

On base `986ed162f806d0fae6e025259126da6e0c60b474`, compiling a constructor
with fields recursively entered the pattern compiler for the positive arm and
again for its fallback. The compiler now schedules those two calls with
explicit heap frames, in the same positive-before-fallback order. It retains
constructor and occurrence origins and builds the same core match tree. A
compiled arm suspended while its fallback runs uses the existing iterative
owned-graph cleanup on error or unwind; shared values remain alive while other
owners exist. This adds fallback scrutinee and variable-vector copies for a
field-bearing branch; the existing source and depth limits still apply. No
work counter, dependency, guard or public term representation was changed.

This removes two Rust call paths, not the whole depth boundary. The parser's
pattern/body cloning and destruction, `Body::replace`, variable-only and
binder-skip compilation, nested body compilation, and public `Rc<Term>`
destruction still have recursive paths. The checker and normalizer likewise
have remaining recursion and a separate 128-level kernel guard. The compiler
still has its 64-level guard: removing it before those paths and automatic
cleanup are safe could expose host stack overflow, including on error. A
stack-safe ownership design compatible with the public `Rc<Term>` API and a
bounded accounting model for pending compiler states remain necessary. This
slice does not make the original `"hi"` source work or merely move its
refusal to a later phase: it still fails during pattern compilation.

## Evidence

A real compiler test compiles and inspects 40 nested field-bearing patterns on
a thread requesting a 64 KiB stack. Separate direct controls confirm 61 nested
fields succeed while 62 reach the existing 64-level limit, and exercise positive
arm versus later fallback error ordering and retained shared ownership. The
suspended-frame panic test releases a unique 20,000-node arm through the new
continuation enum on a requested 64 KiB stack; a shared 20,000-node arm remains
alive until its final owner explicitly releases it. These are focused controls;
they do not make arbitrary public `Rc<Term>` destruction stack-safe.

Three checked nested-constructor hit and fallback programs were added to the
existing native, generated C and generated JavaScript runtime regression.
Against pinned upstream `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, those
three and seven existing string controls each matched the upstream interpreter
twice. The unchanged original returned `True{}` upstream twice and the port
still refused it at the 64-level compilation guard twice. Across 55 existing and
new fixtures, 220 before/after check and run observations matched exactly in
exit status and both output streams; matching errors or refusals are not runtime
passes. The [comparison receipt](cpu-field-stack-evidence.json) records source,
binary and observation hashes.

Each comparison child had a 30-second deadline; the outer bounded runner used
1,200 seconds, with at least 11 GiB workspace free before launch, 10 GiB while
running and 1 GiB temporary-storage headroom. No additional OS memory cap was
applied. Rust 1.96.0, dependencies and the upstream pin are unchanged. GPU and
other host platforms were not tested.

The final-source offline test set completed in bounded batches: **1,397 passed,
zero failed and 49 ignored**. The 200 integration targets contributed 1,009
passes and 48 ignored; separate library, binary and documentation tests
contributed 388 passes and one ignored. The focused surface tests passed ten;
the six focused compiler, kernel and pattern integration targets also passed.
Production all-feature Clippy passed with warnings denied. A supplementary
all-target Clippy attempt failed on an existing formatting lint in
`tests/compiler_open_closures.rs`; it is not a pass.

The standard gate passed formatting, strict production Clippy, the all-feature
build and the real Node TCP/UDP provider check. Its first test run was stopped
at 606 seconds for the requested pause: 87 completed groups had 839 passed,
zero failed and 48 ignored. After work resumed, the restarted gate hit its
1,200-second limit during tests: 136 completed groups had **967 passed, zero
failed and 48 ignored**. In both attempts the active group was interrupted and
later groups were unrun. Neither attempt is a passing gate; the separate bounded
full test-set pass does not change that result. The
[validation receipt](cpu-field-stack-validation.json) records source identity,
commands, timings and resource floors.
