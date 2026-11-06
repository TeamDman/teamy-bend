# CPU C array class-limit failure — 2026-11-06

Source `858e5169b92f38da742174b254784c8c3e037d74` corrects the generated CPU C
diagnostic for an upstream-invalid logical array class. Previously the port's
smaller allocation guard refused the depth-32 witness first, reporting its own
array-budget error. Pinned upstream C checks depth plus the base-two logarithm
of the element-layout stride before allocating and reports the class-31 language failure. The CPU path now
checks that class first and carries a distinct failure kind through worker
shutdown. The existing allocation-class ceiling remains 17; this does not
permit larger allocations. The guard macro is defined only for the CPU runtime;
GPU device execution has not been revalidated.

The focused tests exercise U32 depths 18 and 31, which still refuse at the
existing allocation guard, and depths 32 and 33, which now produce the reference
class-limit message. They check pending stdout, plus failure-kind propagation
and cleanup at one, two and four CPU workers. The three-field padded fixture
uses depth 30 and a four-word element stride (`lgs = 2`): the generated
upstream C and port C both refuse class 32 with stdout `before`, the same stderr and exit 1, twice each.
Its JS and native executions were not attempted: their depth checks differ from
C's layout-class check and executing a depth-30 array could request enormous
memory. This is a C-backend comparison, not a cross-backend allocation claim.

The four bounded array manifests completed **594 observations**, with zero
timeouts and no unwaived depth-diagnostic residual. These observations include
checking, source generation, C compilation, executable runs and expected
refusals; they are not 594 runtime passes. Each manifest retains the class-32
witness, and its five executable lanes report the same one-line language
failure. Other positive and negative controls also remain checked by the
runner. In the nested matrix, 118 path-independent observations match the paused baseline;
the only two differences are the intended port-C depth-error stderr repeats.
Twelve successful generation-output rows contain destination paths and were
checked by the runner rather than compared byte-for-byte across destinations.
No physical shared-view, general memory-safety, unlimited-allocation or
whole-CPU parity claim follows. The earlier discrepancy remains correctly
recorded as historical evidence in [the resumed progress report](cpu-progress-20261106.md).

The pinned toolchain, dependencies, upstream commit and resource limits are
unchanged. GPU/device execution, CUDA sanitizer checks, Windows/macOS hosts,
external consumers and the other outstanding CPU gaps remain unqualified by
this change. The observed upstream-C versus JS `nat_first` error-order conflict
and the three F32.read backend splits remain separately unresolved.

The full offline test set passed in bounded batches at the clean source:
**1,369 passed / 0 failed / 49 ignored**, covering all 197 integration targets,
library, binary and documentation tests. Formatting, standard strict Clippy,
all-feature build, the real Node TCP/UDP provider and targeted strict Clippy
also passed. The uninterrupted standard gate timed out at 1,200 seconds after
941 completed passes, zero failures and 48 ignores; its interrupted group and
remaining groups were not counted in that attempt. The separate complete test
run does not make that single-command gate a pass. Exact identities, hashes,
limits and test batches are in the [validation receipt](cpu-array-class-limit-validation.json).

## Reproduction boundary

The retained fixture is `tests/fixtures/cpu-array-depth/padded-class-limit.bend`.
The upstream C and generated port C were built with Clang 19.1.7, C11 and O3;
the port additionally used strict warnings. Both executables used one worker
and GPU off. Each execution was bounded to 60 seconds. The reference remains
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. Array manifests use the same
pinned reference and explicitly selected port binary; see
[scripts/array-runtime/README.md](../scripts/array-runtime/README.md).

One initial final-comparison launcher invocation failed before any fixture
execution because its output directory already existed. The corrected invocation
used a fresh directory and completed. Raw hashes and validation scope are
recorded in the accompanying receipt; machine-local raw logs remain outside Git.
