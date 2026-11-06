# Stack-safe qualification and term presentation — 2026-11-06

The parser's operator-qualification pass, core `Term` display and compact
structural debug formatting now traverse with explicit work stacks. Qualification
preserves unchanged constructor identity and visits children in the previous
order. Term display preserves the language-facing spellings, precedence,
binder scopes and literal forms in the tested cases. Compact `Debug` preserves
the tested structural spelling; alternate pretty-debug formatting now uses the
same compact layout, an intentional Rust API change. Ordinary `Rc<Term>`
destruction remains iterative. Formatter writer errors and ordinary unwinding
are covered; a process-wide allocator failure can still abort rather than
produce a recoverable Bend error.

On a requested 64 KiB thread stack, qualification handled a 50,000-node term;
display and debug handled a 20,000-node term, and display handled 10,000 nested
matches. Tests verify retained shared constructors, changed operators,
metadata/order, normal cleanup, unwind and formatter-error cleanup. A
200-term generated comparison matched the prior implementation's Display and
compact Debug bytes; a separate comparison covered every Term variant.

## Why the compiler guard remains

The 64-level pattern-compilation guard is a temporary **port** restriction,
not an upstream language rule. In an isolated no-guard experiment, the
unchanged upstream `"hi"` source lowered and then checking and execution
refused at the distinct 128-level kernel input guard. `nat65` executed as
`True{}`, matching upstream; `columns65` reached the missing-cases checker
diagnostic with the same status and streams as upstream. Temporarily bypassing
the kernel input guard as well moved `"hi"` to the engine's own 128-level
nesting refusal, not to successful execution.

That candidate cannot be published safely yet. On a requested 2 MiB thread
stack, loading and checking the existing `bool-122.bend` fixture overflowed
and aborted in the recursive checker; the same program happened to succeed on
the CLI's larger main stack. The experimentally lowered function has a direct
chain of 123 `Mat` arms, and `Engine::check_match` recursively calls `check`
for each arm. Its `enter` limit does not prevent this measured 2 MiB overflow.
A checked-in subprocess regression ensures this
fixture is rejected safely or checked successfully rather than aborting. The
current compiler guard remains because of that demonstrated host-stack hazard,
not as an upstream semantic rule or solely because a new budget is undecided.
The checking path needs explicit continuations that preserve arm-before-fallback
ordering, usage merging, diagnostics, fuel and cleanup; its logical depth
accounting can then be reconsidered against actual host recursion.
Separately, loading even a one-character control on a requested 64 KiB thread
stack overflowed in the full loader; private lowering and presentation tests
do not establish whole-loader safety at that stack size.

Before iterative display, formatting a term produced by the experimentally
admitted 64-character pattern also overflowed the default process stack. The
new formatter rendered it successfully. Removing the kernel input guard is
likewise not established safe for every downstream recursive checker and
compiler consumer; it was restored after the bounded experiment.

## Growth observations

An isolated no-guard build loaded repeated-ASCII string patterns. These are
observations for this particular family, not worst-case bounds. The probe
counted expanded occurrences and unique `Rc` nodes in the lowered function;
peak RSS includes the loader and Base imports. Each process had a 30-second
deadline, with no extra OS memory cap.

| Characters | Maximum term depth | Occurrences | Unique nodes | Peak RSS (KiB) |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 70 | 307 | 239 | 10,048 |
| 2 | 138 | 610 | 474 | 9,916 |
| 4 | 274 | 1,216 | 944 | 9,912 |
| 8 | 546 | 2,428 | 1,884 | 14,048 |
| 16 | 1,090 | 4,852 | 3,764 | 36,636 |
| 32 | 2,178 | 9,700 | 7,524 | 125,036 |
| 64 | 4,354 | 19,396 | 15,044 | 472,704 |

The measured graph size is linear for these samples, while observed loader
memory rises much faster. This does not justify an arbitrary language depth
limit, but it makes allocation behavior relevant. The parser still has its
own port-specific source, string and expression limits; the checker retains
its input and engine limits and a 2,000,000-occurrence/work budget.

## Validation

The base commit is `72beb81f7b5196a685b618e7bddea801ca69bbf7`; upstream is
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. The final guarded candidate
completed the entire offline test set in bounded batches: **1,425 passed, zero
failed and 49 ignored**, including all 203 integration targets, library, binary
and documentation targets. Workspace formatting, strict all-feature production
Clippy and build, and the Node TCP/UDP provider check passed. Ten neighboring
programs matched pinned upstream twice; 220 check/run observations across 55
pattern fixtures matched the comparison binary from `928bde`, which had already
matched the `72beb81` base on those observations, in status and both streams.
Matching refusals are not execution passes. Comparison processes had 30-second
deadlines, and Cargo batches had 1,200-second bounds with 11 GiB workspace
preflight, 10 GiB active workspace and 1 GiB temporary-storage floors. No extra
OS memory cap was imposed. The
no-guard candidate's initial broader test run aborted on `bool-122`, as described
above; it is not counted as a production test pass. Initial Clippy attempts
found issues that were fixed and rerun. The uninterrupted standard gate was not
rerun; its prior 1,200-second timeout remains incomplete. GPU and other hosts
were unrun. See the [validation receipt](cpu-iterative-presentation-validation.json).
