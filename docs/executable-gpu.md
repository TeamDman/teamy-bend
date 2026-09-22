# Generated CUDA execution

The executable C backend can run marked Bend calls on CUDA. The Rust compiler
uses the same typed function bodies and word layouts for CPU and device code.
Native `run`, proof evaluation and JavaScript preserve the value semantics of
the mark without offloading work.

The Windows/CUDA execution milestone is validated for the cases below. Full
upstream GPU parity remains in progress.

## Calling and selecting the GPU

Use a named call such as `walk!(depth, value)`. A partial ordinary call keeps its
closure until the remaining arguments arrive. Primitive operations that are
lowered directly can retain an inert mark, as in upstream. An explicit marked
call creates a task boundary; ordinary direct calls retain their direct-call
path. Existing task boundaries use the definition's global GPU eligibility.

Generate C with:

```text
teamy-bend compile --executable --target c --output program.c main.bend
```

Compile the resulting C program with the usual host compiler. CUDA is loaded
dynamically; startup loads a valid cache or uses NVRTC to compile the embedded
device source before program effects. Windows lookup uses the driver and CUDA toolkit libraries,
including `CUDA_PATH`. Unix uses the dynamic loader and additionally needs
`-ldl` where those functions are not provided by libc.

The generated executable also accepts `--gpu on|off|NMB|NGB`, `--threads N`,
`--gpu-build` and `--help`. Native builds and persistent caching have Windows
validation; see [native builds](native-builds.md) for their contracts.
An explicit CLI policy overrides `BEND_GPU` in the generated program's environment:

| Value | Behavior |
| --- | --- |
| `auto`, or unset | Use CUDA when available; otherwise execute on the CPU. |
| `off` | Execute on the CPU without initializing CUDA. |
| `on` | Fail at startup if a binary with an emitted device program cannot initialize CUDA. |

Only device unavailability permits automatic fallback. Compilation, allocation,
transfer or execution failures terminate the invocation. Consumed arguments are
never replayed on the CPU. Disabled or unavailable CUDA retains bounded CPU tail
execution rather than accumulating offload continuations.

## Storage and scheduling

The coordinator drains CPU workers before exporting a ready task. Suspended CPU
frames remain on the host. Corpus words, ownership metadata, allocation counts
and free lists transfer together. Successful completion imports that same state
before delivering the complete result words and ownership mask to the CPU.

Host values and ownership metadata have stable virtual reservations. CPU
allocation commits their used prefixes before publishing a new heap span. A
GPU result may grow beyond the previous host prefix: both host arrays become
writable before either readback, and allocator state is published only after
both downloads succeed. A failed commitment aborts the invocation without
replaying device work. Device buffers still allocate their full logical spans;
this host change does not establish upstream default-memory parity.

Device frames, queues and graph records store offsets into bounded device
storage. Generated functions return words, direct calls or task graphs to an
iterative dispatcher. Fork adoption and result delivery run between parallel
work launches; kernels do not wait for another block to finish a dependency.
Self-tail calls reuse frames. Boxed results borrow persistent device storage,
preserving the original argument across suspended calls.

The module, stream and allocations persist across marked calls in one
invocation. Intermediate graph work remains on the device. The current boundary
copies the used corpus and metadata prefix for each offload; it does not yet
track dirty regions across CPU execution.

The shared value code preserves CPU constructor, closure, array and reference
count representations. Foreign IO request construction is separate from host
effect execution. Device source contains no host foreign handlers. F32 text
conversion handlers remain unavailable to device dispatch. The emitted request
inspection guard covers the compiler's foreign-request inventory; arbitrary
runtime changes to foreign registrations need further qualification.

## Bounds and current evidence

CUDA currently requires compute capability 7.0 or newer. The storage allocator
uses short synchronized transactions and independent thread scheduling. Existing
step, task and continuation limits remain charged across host and device work.
Synchronous value-conversion helpers have a separate per-lane
`BEND_MAX_FRAMES` depth limit. Their depth storage counts against device scratch;
it is released when the root completes and does not consume continuation slots.
`BEND_MAX_GPU_ALLOC` caps device allocations, including temporary allocation
growth; its default is three times `BEND_MAX_ALLOC`. Device scratch defaults to
half `BEND_MAX_ALLOC` and can be configured with `BEND_MAX_GPU_SCRATCH`.

`BEND_GPU_BLOCKS`, `BEND_GPU_THREADS` and `BEND_GPU_QUANTUM` configure bounded
dispatch launches. Long individual value-helper traversals still need broader
watchdog and cancellation qualification; a dispatch quantum alone does not make
every helper preemptible. Expected device errors record their first diagnostic
and end the affected lanes; lock waiters observe the same cancellation flag.
The host discards the failed invocation without importing its partially changed
heap. Budget failures therefore preserve the specific Bend diagnostic and allow
a fresh invocation in the same process. Actual CUDA hardware or driver failures
remain separately reported; some invalidate subsequent CUDA work in the process.

Actual generated-program tests on Windows/CUDA cover recursive forks with
overlapping task lifetimes, repeated offloads with one compiled module and five
reused buffers, a 5,001-step tail loop returning owned multiword data,
host-created closures inside constructors, partial marked calls, boxed arguments
used after a suspended call, and array copy-on-write preserving a parked CPU
owner. Hardware tests are explicitly ignored by the portable gate and run
separately with an installed CUDA target. Retained receipts distinguish exact
generated-program execution from standalone adapter and shared-helper probes.

Four generated failure/recovery programs cover step, task and scratch budgets,
including concurrently active unsafe sibling computations. Each fails without
CPU replay, releases the CUDA context and host resources, then successfully
offloads a base-case input in a second invocation of the same generated program.
Two nested-conversion tests separately prove that helper depth can exceed a
one-continuation limit and that its own lower depth limit is enforced. The
first execution milestone has twelve passing hardware cases. The current suite
passes twenty-two, adding cache lifecycle, kernel-contract startup checks and actual
native CLI build/relocation coverage, plus host commitment growth and recovery. The portable gate skips all twenty-two explicitly.

Two host-storage tests return a GPU-created 4,096-element U32 array whose heap
span exceeds the host's existing committed prefix. Both host arrays grow before
either heap download, without changing their base addresses. A second test fails
the metadata commitment after the payload commitment succeeds: neither heap
download runs, no CPU replay occurs, all resources release and the identical
program then succeeds in the same process. The expected value also matches
freshly executed upstream JavaScript for that complete Bend source.

Each of the first twelve cases also has an exact generated-executable Compute
Sanitizer pass from the `e2ae6ed` milestone. These sanitizer results retain that
attribution. One recursive-fork invocation timed out under instrumentation; four
unchanged reruns completed in about three seconds with zero errors. The first
timeout is retained as an unexplained validation observation. No source,
launch dimensions or timeout was changed for those reruns.

## Remaining GPU work

- Broader upstream programs, numeric behavior, invalid outcomes, effect
  boundaries and additional allocation/transfer failure paths.
- Remaining upstream CLI/platform contracts, default device memory sizing and Metal support. Unix CUDA
  and Metal remain unexecuted on the current Windows validation host.
- Yielding within long helper operations, allocator contention, transfer
  reduction and scheduling optimization.
- Representative cold and warm performance measurements. Correct outputs and
  device overlap do not establish a speedup.

The [implementation plan](implementation-plan.md) retains these requirements.
The [Makepad and teamy-tts references](gpu-port-references.md) informed persistent
resources, explicit transfers and ordered work submission. Their reported
performance improvements are not Bend benchmark results.
